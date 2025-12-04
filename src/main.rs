use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path};
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
        )?;
        return Ok(());
    }

    if path == "/about" {
        send_response(&mut stream, "200 OK", b"<h1>About Page</h1>", "text/html")?;
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
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status,
        content_type,
        body.len(),
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}
fn serve_static_file(stream: &mut TcpStream, request_path: &str) -> std::io::Result<()> {
    let file_path = resolve_safe_path("public", request_path)?;
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
    send_response(stream, "200 OK", &content, content_type)
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
        Ok(content) => send_response(
            stream,
            "404 Not Found",
            &content,
            "text/html; charset=utf-8",
        ),
        Err(_) => send_response(stream, "404 Not Found", b"404 Not Found", "text/plain"),
    }
}
