use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path};
use std::time::SystemTime;
fn main() -> std::io::Result<()> {
    //设置监听端口 127.0.0.1:7878
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is starting, Listening to 127.0.0.1:7878");
    //从持续监听端口获得流
    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?; //处理流
    }
    Ok(())
}
fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(
            &mut stream,
            "400 Bad Request",
            b"Invalid request",
            "text/plain",
            Some("Cache-Control: no-store"),
        )?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        send_response(
            &mut stream,
            "405 Method Not Allowed",
            b"Method Not Allowed",
            "text/plain",
            Some("Cache-Control: no-store"),
        )?;
        return Ok(());
    }

    if path == "/about" {
        send_response(
            &mut stream,
            "200 OK",
            b"<h1>About Page</h1>",
            "text/html",
            Some("Cache-Control: no-store"),
        )?;
        return Ok(());
    }
    serve_static_file(&mut stream, path)?;
    Ok(())
}
fn get_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream", // 默认二进制
    }
}
fn send_response(
    stream: &mut TcpStream,
    status: &str,
    body: &[u8],
    content_type: &str,
    extra_headers: Option<&str>,
) -> std::io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        status,
        content_type,
        body.len()
    );

    if let Some(headers) = extra_headers {
        response.push_str(headers);
        response.push('\r');
        response.push('\n');
    }

    response.push_str("\r\n");

    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}
fn serve_static_file(stream: &mut TcpStream, request_path: &str) -> std::io::Result<()> {
    // 处理路径
    let file_path = resolve_safe_path("public", request_path)?;
    // 检查是否为目录
    if file_path.is_dir() {
        let index_path = file_path.join("index.html");
        if index_path.exists() {
            let content = fs::read(&index_path)?;
            let content_type = get_content_type("index.html");
            let cache_control = get_cache_control(&index_path);
            return send_response(
                stream,
                "200 OK",
                &content,
                content_type,
                Some(&format!("Cache-Control: {}", cache_control)),
            );
        } else {
            return serve_directory_listing(stream, &file_path, request_path);
        }
    }
    // 检查文件是否存在
    if !file_path.exists() {
        return serve_404(stream);
    }
    let content = match fs::read(&file_path) {
        Ok(data) => data,
        Err(_) => return serve_404(stream),
    };
    let content_type = match file_path.to_str() {
        Some(path_str) => get_content_type(path_str),
        None => "application/octet-stream", // 处理非UTF-8路径的情况
    };
    let cache_control = get_cache_control(&file_path);
    send_response(
        stream,
        "200 OK",
        &content,
        content_type,
        Some(&format!("Cache-Control: {}", cache_control)),
    )
}
fn resolve_safe_path(base: &str, request_path: &str) -> std::io::Result<std::path::PathBuf> {
    let base_path = Path::new(base).canonicalize()?;
    // 处理根路径 "/" → 应该返回 base/index.html
    let clean_path = if request_path == "/" {
        "index.html"
    } else {
        // 移除开头的 '/'（因为 public/ 已是根）
        request_path.trim_start_matches('/')
    };
    let mut final_path = base_path.clone();
    for component in Path::new(clean_path).components() {
        match component {
            Component::Normal(os_str) => {
                final_path.push(os_str);
            }
            Component::ParentDir | Component::CurDir => {
                continue;
            }
            Component::RootDir => {
                continue;
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid path component",
                ));
            }
        }
    }
    if !final_path.starts_with(&base_path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path traversal detected",
        ));
    }
    Ok(final_path)
}
fn serve_404(stream: &mut TcpStream) -> std::io::Result<()> {
    match fs::read("public/404.html") {
        Ok(content) => {
            let cache_control = "no-cache, no-store, must-revalidate";
            send_response(
                stream,
                "404 Not Found",
                &content,
                "text/html; charset=utf-8",
                Some(&format!("Cache-Control: {}", cache_control)),
            )
        }
        Err(_) => send_response(
            stream,
            "404 Not Found",
            b"404 Not Found",
            "text/plain",
            Some("Cache-Control: no-cache"),
        ),
    }
}
fn serve_directory_listing(
    stream: &mut TcpStream,
    dir_path: &Path,
    request_path: &str,
) -> std::io::Result<()> {
    let mut entries: Vec<(String, bool, Option<u64>, Option<SystemTime>)> = vec![];

    // 读取目录内容
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = match fs::metadata(&path) {
            Ok(meta) => meta,
            Err(_) => continue, // 跳过无法获取元数据的项
        };

        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };
        let modified = metadata.modified().ok();

        entries.push((name, is_dir, size, modified));
    }

    // 排序：目录在前，文件在后；同类型按名称排序
    entries.sort_by(|a, b| {
        match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less, // 目录优先
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0), // 同类型按名称
        }
    });

    // 构建 HTML
    let mut html = String::from("<!DOCTYPE html><html><head>");
    html.push_str(r#"<meta charset="utf-8"><title>Index of "#);
    html.push_str(request_path);
    html.push_str(r#"</title><style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; }
        h1 { color: #333; }
        table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
        th, td { padding: 0.5rem 1rem; text-align: left; border-bottom: 1px solid #eee; }
        th { font-weight: 600; color: #555; }
        tr:hover { background-color: #f9f9f9; }
        .dir::before { content: "📁 "; }
        .file::before { content: "📄 "; }
        .size { color: #888; font-size: 0.9em; }
        a { text-decoration: none; color: #0070f3; }
        a:hover { text-decoration: underline; }
    </style></head><body>"#);

    html.push_str("<h1>Index of ");
    html.push_str(request_path);
    html.push_str(
        "</h1><table><thead><tr><th>Name</th><th>Size</th><th>Modified</th></tr></thead><tbody>",
    );

    // 添加 ".." 返回上级（除非是根目录）
    if request_path != "/" && request_path != "" {
        let parent_path = Path::new(request_path)
            .parent()
            .map(|p| p.to_str().unwrap_or("/"))
            .unwrap_or("/");
        let display_parent = if parent_path.is_empty() {
            "/"
        } else {
            parent_path
        };
        html.push_str(&format!(
            r#"<tr><td><a href="{}" class="dir">../</a></td><td>-</td><td>-</td></tr>"#,
            display_parent
        ));
    }

    // 添加每个条目
    for (name, is_dir, size, _modified) in entries {
        let encoded_name = url_encode(&name);
        let full_url = if request_path.ends_with('/') {
            format!("{}{}", request_path, encoded_name)
        } else {
            format!("{}/{}", request_path, encoded_name)
        };

        let class = if is_dir { "dir" } else { "file" };
        let size_display = match size {
            Some(s) if s == 0 => "-".to_string(),
            Some(s) => format!("{} B", s),
            None => "-".to_string(),
        };

        let mod_display = "-";

        html.push_str(&format!(
            r#"<tr><td><a href="{}" class="{}">{}</a></td><td class="size">{}</td><td>{}</td></tr>"#,
            full_url,
            class,
            name,
            size_display,
            mod_display
        ));
    }

    html.push_str("</tbody></table></body></html>");

    send_response(
        stream,
        "200 OK",
        html.as_bytes(),
        "text/html; charset=utf-8",
        Some("Cache-Control: no-cache"),
    )
}
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        })
        .collect()
}
fn get_cache_control(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        // 不缓存 HTML（确保用户看到最新内容）
        Some("html") | Some("htm") => "no-cache, no-store, must-revalidate",

        // 长期缓存静态资源（1 小时 = 3600 秒）
        Some("css") | Some("js") | Some("png") | Some("jpg") | Some("jpeg") | Some("gif")
        | Some("webp") | Some("ico") | Some("svg") | Some("woff") | Some("woff2") | Some("ttf") => {
            "public, max-age=3600"
        }

        // 其他文件（如 txt、json）—— 按需调整
        _ => "no-cache",
    }
}
