use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path};
use std::time::{SystemTime, Instant, UNIX_EPOCH};
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
/// 处理TCP连接请求
/// 
/// 该函数负责解析HTTP请求，根据请求路径和方法返回相应的响应。
/// 支持GET方法访问/about路径和其他静态文件。
/// 
/// # 参数
/// * `stream` - TCP流连接，用于读取请求和发送响应
/// 
/// # 返回值
/// * `std::io::Result<()>` - 操作结果，成功时返回Ok，失败时返回相应的错误信息
fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    // 解析HTTP请求的第一行，获取请求方法和路径
    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let client_addr = stream.peer_addr().ok();
    // 检查请求格式是否有效（至少包含方法和路径）
    if parts.len() < 2 {
        let log = LogEntry::new("UNKNOWN".to_string(), "INVALID".to_string(), client_addr);
        log.log("400");
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
    let log = LogEntry::new(method.to_string(), path.to_string(), client_addr);
    // 只支持GET方法，其他方法返回405错误
    if method != "GET" {
        log.log("405");
        send_response(
            &mut stream,
            "405 Method Not Allowed",
            b"Method Not Allowed",
            "text/plain",
            Some("Cache-Control: no-store"),
        )?;
        return Ok(());
    }
    // 处理/about路径的特殊响应
    if path == "/about" {
        log.log("200");
        send_response(
            &mut stream,
            "200 OK",
            b"<h1>About Page</h1>",
            "text/html",
            Some("Cache-Control: no-store"),
        )?;
        return Ok(());
    }
    let status = match serve_static_file(&mut stream, &path) {
        Ok(code) => code,
        Err(_) => {
            // 发生内部错误（如文件读取失败）
            serve_500(&mut stream)?;
            "500".to_string()
        }
    };
    log.log(&status);
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
fn serve_static_file(stream: &mut TcpStream, request_path: &str) -> std::io::Result<String> {
    let file_path = resolve_safe_path("public", request_path)?;

    if file_path.is_dir() {
        let index_path = file_path.join("index.html");
        if index_path.exists() {
            let content = fs::read(&index_path)?;
            let content_type = get_content_type("index.html");
            let cache_control = get_cache_control(&index_path);
            send_response(
                stream,
                "200 OK",
                &content,
                content_type,
                Some(&format!("Cache-Control: {}", cache_control)),
            )?;
            return Ok("200".to_string());
        } else {
            serve_directory_listing(stream, &file_path, request_path)?;
            return Ok("200".to_string());
        }
    }
    if !file_path.exists() {
        serve_404(stream)?;
        return Ok("404".to_string());
    }
    let content = fs::read(&file_path)?;
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
    )?;
    Ok("200".to_string())
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
struct LogEntry {
    method: String,
    path: String,
    client_addr: Option<SocketAddr>,
    start_time: Instant,
}
impl LogEntry {
    fn new(method: String, path: String, client_addr: Option<SocketAddr>) -> Self {
        Self {
            method,
            path,
            client_addr,
            start_time: std::time::Instant::now(),
        }
    }
    fn log(&self, status_code: &str) {
        let elapsed = self.start_time.elapsed().as_millis();
        let timestamp = format_timestamp();
        let client_info = self.client_addr.map(|addr| addr.to_string()).unwrap_or_else(|| "unknown".to_string());
        let message = format!(
            "[{}] \"{} {}\" {} {}ms - {}",
            timestamp, self.method, self.path, status_code, elapsed, client_info
        );
        eprintln!("{}",message);
        // 追加到日志文件
        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("access.log")
            .and_then(|mut file| writeln!(file, "{}", message)){
            eprintln!("Failed to write log to file: {}", e);
        }
    }
}
fn format_timestamp() -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // 将 Unix 时间戳转换为 UTC 日期时间（手动计算）
    let datetime = timestamp_to_datetime(secs);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        datetime.year,
        datetime.month,
        datetime.day,
        datetime.hour,
        datetime.minute,
        datetime.second
    )
}
// 简单的 Unix 时间戳 → UTC 日期时间转换（不考虑闰秒）
struct DateTime {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}
fn timestamp_to_datetime(mut t: u64) -> DateTime {
    // 秒转为分钟、小时等
    let second = (t % 60) as u32;
    t /= 60;
    let minute = (t % 60) as u32;
    t /= 60;
    let hour = (t % 24) as u32;
    t /= 24;

    // 处理年月日（从 1970 年开始）
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if t < days_in_year as u64 {
            break;
        }
        t -= days_in_year as u64;
        year += 1;
    }
    let mut month = 1;
    let mut days_left = t as u32;
    let days_in_month = [
        31, // Jan
        if is_leap_year(year) { 29 } else { 28 }, // Feb
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    for (i, &dim) in days_in_month.iter().enumerate() {
        if days_left < dim {
            month = i as u32 + 1;
            break;
        }
        days_left -= dim;
    }

    DateTime {
        year,
        month,
        day: days_left + 1, // 日从 1 开始
        hour,
        minute,
        second,
    }
}
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn serve_500(stream: &mut TcpStream) -> std::io::Result<()> {
    send_response(
        stream,
        "500 Internal Server Error",
        b"Internal Server Error",
        "text/plain",
        Some("Cache-Control: no-cache"),
    )
}