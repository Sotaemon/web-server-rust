/*
use tokio::fs;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, AsyncReadExt};
use std::io::Result;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::path::{Path, Component};

pub struct LogEntry {
    method: String,
    path: String,
    client_addr: Option<SocketAddr>,
    start_time: Instant,
}
impl LogEntry {
    pub fn new(method: String, path: String, client_addr: Option<SocketAddr>) -> Self {
        Self {
            method,
            path,
            client_addr,
            start_time: Instant::now(),
        }
    }

    pub fn log(&self, status_code: &str) {
        let elapsed = self.start_time.elapsed().as_millis();
        let timestamp = format_timestamp();
        let client_info = self
            .client_addr
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let message = format!(
            "[{}] \"{} {}\" {} {}ms - {}",
            timestamp, self.method, self.path, status_code, elapsed, client_info
        );
        eprintln!("{}", message);
        
        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("access.log")
            .and_then(|mut file| writeln!(file, "{}", message))
        {
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
        31,                                       // Jan
        if is_leap_year(year) { 29 } else { 28 }, // Feb
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
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
pub async fn send_response(
    stream: &mut TcpStream,
    status: &str,
    body: &[u8],
    content_type: &str,
    extra_headers: Option<&str>,
) -> Result<()> {
    let mut response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        status,
        content_type,
        body.len()
    );

    if let Some(headers) = extra_headers {
        response.push_str(headers);
        response.push_str("\r\n");
    }

    response.push_str("\r\n");

    stream.write_all(response.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}

pub fn send_json_response(
    stream: &mut TcpStream,
    status_code: u16,
    json_value: serde_json::Value,
) -> Result<()> {
    let body = serde_json::to_vec(&json_value)
        .unwrap_or_else(|_| b"{\"error\":\"Serialization failed\"}".to_vec());
    
    let status_text = match status_code {
        200 => "200 OK",
        201 => "201 Created",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        409 => "409 Conflict",
        500 => "500 Internal Server Error",
        _ => "500 Internal Server Error",
    };
    
    send_response(
        stream,
        status_text,
        &body,
        "application/json; charset=utf-8",
        Some("Cache-Control: no-store"),
    )
}
pub fn get_content_type(path: &str) -> &'static str {
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
        _ => "application/octet-stream",
    }
}
pub fn resolve_safe_path(base: &str, request_path: &str) -> Result<std::path::PathBuf> {
    let base_path = Path::new(base).canonicalize()?;
    let clean_path = if request_path == "/" {
        "index.html"
    } else {
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
pub fn get_cache_control(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|s| s.to_str());
    match ext {
        Some("html") | Some("htm") => "no-cache, no-store, must-revalidate",
        Some("css") | Some("js") | Some("png") | Some("jpg") | Some("jpeg") | Some("gif")
        | Some("webp") | Some("ico") | Some("svg") | Some("woff") | Some("woff2") | Some("ttf") => {
            "public, max-age=3600"
        }
        _ => "no-cache",
    }
}
pub async fn serve_static_file(stream: &mut TcpStream, request_path: &str) -> Result<String> {
    let file_path = resolve_safe_path("public", request_path)?;

    if file_path.is_dir() {
        let index_path = file_path.join("index.html");
        if index_path.exists() {
            let content = fs::read(&index_path).await?;
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
    let content = fs::read(&file_path).await?;
    let content_type = match file_path.to_str() {
        Some(path_str) => get_content_type(path_str),
        None => "application/octet-stream",
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
pub fn serve_404(stream: &mut TcpStream) -> Result<()> {
    match std::fs::read("public/404.html") {
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
pub fn serve_500(stream: &mut TcpStream) -> Result<()> {
    send_response(
        stream,
        "500 Internal Server Error",
        b"Internal Server Error",
        "text/plain",
        Some("Cache-Control: no-cache"),
    )
}
fn serve_directory_listing(
    stream: &mut TcpStream,
    dir_path: &Path,
    request_path: &str,
) -> Result<()> {
    let mut entries: Vec<(String, bool, Option<u64>, Option<SystemTime>)> = vec![];
    // 读取目录内容
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = match std::fs::metadata(&path) {
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
*/