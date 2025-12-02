use std::io::{Read, Write};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is starting, Listening to 127.0.0.1:7878");
    for stream in listener.incoming() {
        let stream = stream?;
        handle_connection(stream)?;
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


    // ✅ 关键修改：从 public/ 文件夹读取文件
    let (status, body, content_type) = match path {
        "/" => {
            let path = Path::new("/home/swang/Projects/web-server-rust/public/").join("index.html");
            match fs::read_to_string(&path) {
                Ok(content) => ("200 OK", content, "text/html"),
                Err(e) => {
                    eprintln!("ERROR: Failed to read {:?}: {}", path, e);
                    ("404 Not Found", "404 Not Found".to_string(), "text/plain")
                }
            }
        }
        "/about" => ("200 OK", "This is the About page!".to_string(), "text/plain"),
        _ => ("404 Not Found", "404 Not Found".to_string(), "text/plain"),
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
