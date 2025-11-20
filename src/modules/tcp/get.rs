use std::io::Write;
use std::net::TcpStream;

pub fn make_get_response(stream: &TcpStream, path: &str, version: &str) {
    let corrected_path = super::get_path_corrected(path);
    let content_type = super::get_content_type(&corrected_path);

    let is_text_content = matches!(
        content_type.as_str(),
        "text/html; charset=UTF-8"
            | "text/css; charset=UTF-8"
            | "application/javascript; charset=UTF-8"
            | "application/json; charset=UTF-8"
            | "text/plain; charset=UTF-8"
            | "application/xml; charset=UTF-8"
    );

    if is_text_content {
        make_get_text_response(stream, &corrected_path, version, &content_type);
    } else {
        make_get_binary_response(stream, &corrected_path, version, &content_type);
    }
}

fn make_get_text_response(stream: &TcpStream, path: &str, version: &str, content_type: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            make_error_response(stream, "404 Not Found");
            return;
        }
    };

    let response = format!(
        "{} 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        version,
        content_type,
        content.len(),
        content
    );

    let mut mutable_stream = stream;
    if let Err(e) = mutable_stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response: {}", e);
    }

    if let Err(e) = mutable_stream.flush() {
        eprintln!("Failed to flush stream: {}", e);
    }
}

fn make_get_binary_response(stream: &TcpStream, path: &str, version: &str, content_type: &str) {
    let content = match std::fs::read(path) {
        Ok(content) => content,
        Err(_) => {
            make_error_response(stream, "404 Not Found");
            return;
        }
    };

    let headers = format!(
        "{} 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        version,
        content_type,
        content.len()
    );

    let mut mutable_stream = stream;
    if let Err(e) = mutable_stream.write_all(headers.as_bytes()) {
        eprintln!("Failed to write headers: {}", e);
        return;
    }

    if let Err(e) = mutable_stream.write_all(&content) {
        eprintln!("Failed to write content: {}", e);
    }
}

fn make_error_response(stream: &TcpStream, message: &str) {
    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\n\
         Content-Type: text/plain; charset=UTF-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        message.len(),
        message
    );

    let mut mutable_stream = stream;
    if let Err(e) = mutable_stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write error response: {}", e);
    }
}
