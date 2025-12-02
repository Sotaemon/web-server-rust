use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
fn main() -> std::io::Result<()> {
    //设置监听端口 127.0.0.1:7878
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is starting, Listening to 127.0.0.1:7878");

    //从持续监听端口获得流
    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?;         //处理流
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
        let body = "Invalid request";
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        stream.write(response.as_bytes())?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        let body = "Method not allowed";
        let response = format!(
            "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write(response.as_bytes())?;
        return Ok(());
    }

    let (status, body, content_type) = if path == "/" {
        match fs::read_to_string("public/index.html") {
            Ok(content) => ("200 OK", content, "text/html; charset=utf-8"),
            Err(_) => read_404_page(), // 复用404逻辑
        }
    } else if path == "/about" {
        ("200 OK", "This is the About page!".to_string(), "text/plain")
    } else if path.starts_with("/styles/") {
        let file_path = format!("public{}", path); // 转为 public/css/style.css
        match fs::read_to_string(&file_path) {
            Ok(content) => ("200 OK", content, "text/css; charset=utf-8"),
            Err(_) => read_404_page(),
        }
    } else {
        // 其他路径 → 404
        read_404_page()
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );

    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
fn read_404_page() -> (&'static str, String, &'static str) {
    match fs::read_to_string("public/404.html") {
        Ok(content) => ("404 Not Found", content, "text/html; charset=utf-8"),
        Err(_) => ("404 Not Found", "404 Not Found".to_string(), "text/plain"),
    }
}