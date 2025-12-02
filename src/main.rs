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

    let response_bytes = if path == "/" {
        match fs::read("public/index.html") {
            Ok(bytes) => build_response("200 OK", &bytes, get_content_type(".html")),

            Err(_) => build_404_response(),
        }
    } else if path == "/about" {
        let body = b"This is the About page!";

        build_response("200 OK", body, "text/plain")
    } else if path.starts_with("/styles/")
        || path.starts_with("/scripts/")
        || path.starts_with("/assets/")
    {
        let file_path = format!("public{}", path);

        match fs::read(&file_path) {
            Ok(bytes) => build_response("200 OK", &bytes, get_content_type(path)),

            Err(_) => build_404_response(),
        }
    } else {
        build_404_response()
    };

    stream.write_all(&response_bytes)?;
    stream.flush()?;
    Ok(())
}
fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".html") || path == "/" {
        "text/html; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn build_response(status: &str, body: &[u8], content_type: &str) -> Vec<u8> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status,
        content_type,
        body.len()
    );

    let mut response = header.into_bytes();
    response.extend_from_slice(body);
    response
}

fn build_404_response() -> Vec<u8> {
    match fs::read("public/404.html") {
        Ok(bytes) => build_response("404 Not Found", &bytes, "text/html; charset=utf-8"),
        Err(_) => {
            let body = b"404 Not Found";
            build_response("404 Not Found", body, "text/plain")
        }
    }
}
