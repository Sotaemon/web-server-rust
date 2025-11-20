use std::io::Read;
use std::{net::TcpStream, path::PathBuf};

mod get;
mod post;

pub fn handle_connection(mut stream: TcpStream) {
    let request = get_request_str(&mut stream);
    let (request_line, headers, body) = parse_request(&request);
    make_response(&stream, &request_line);
}

fn make_response(mut stream: &TcpStream, request_line: &str) {
    let (method, path, version) = parse_request_line(request_line);
    match method.as_str() {
        "GET" => get::make_get_response(&mut stream, &path, &version), // 添加引用操作符
        "POST" => post::post_make_response(&mut stream, request_line.to_string()),
        _ => {}
    }
}

fn get_request_str(mut stream: &TcpStream) -> String {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    request_str.to_string()
}

fn parse_request(request: &str) -> (String, Vec<String>, String) {
    let lines: Vec<&str> = request.lines().collect();
    let request_line = lines[0].to_string();

    let mut headers = vec![];
    let mut i = 1;
    while i < lines.len() && !lines[i].is_empty() {
        headers.push(lines[i].to_string());
        i += 1;
    }

    let body = if i + 1 < lines.len() {
        lines[i + 1..].join("\n")
    } else {
        String::new()
    };

    (request_line, headers, body)
}

fn parse_request_line(request_line: &str) -> (String, String, String) {
    let parts: Vec<&str> = request_line.split(' ').collect();
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();
    (method, path, version)
}

fn get_path_corrected(path: &str) -> String {
    if path == "/" {
        return "./sources/index.html".to_string();
    }

    let clean_path = if path.starts_with("/") {
        &path[1..]
    } else {
        path
    };

    let mut full_path = PathBuf::from("./sources");
    full_path.push(clean_path);

    match full_path.canonicalize() {
        Ok(canonical_path) => {
            let base_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("sources")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("./sources"));

            if canonical_path.starts_with(&base_dir) {
                canonical_path.to_string_lossy().to_string()
            } else {
                "./sources/index.html".to_string()
            }
        }
        Err(_) => "./sources/index.html".to_string(),
    }
}

fn get_content_type(path: &str) -> String {
    let content_type = match path {
        p if p.ends_with(".html") || p.ends_with(".htm") => "text/html; charset=UTF-8",
        p if p.ends_with(".css") => "text/css; charset=UTF-8",
        p if p.ends_with(".js") => "application/javascript; charset=UTF-8",
        p if p.ends_with(".json") => "application/json; charset=UTF-8",
        p if p.ends_with(".png") => "image/png",
        p if p.ends_with(".jpg") || p.ends_with(".jpeg") => "image/jpeg",
        p if p.ends_with(".gif") => "image/gif",
        p if p.ends_with(".svg") => "image/svg+xml",
        p if p.ends_with(".ico") => "image/x-icon",
        p if p.ends_with(".txt") => "text/plain; charset=UTF-8",
        p if p.ends_with(".xml") => "application/xml; charset=UTF-8",
        p if p.ends_with(".pdf") => "application/pdf",
        _ => "application/octet-stream",
    };
    content_type.to_string()
}
