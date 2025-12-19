use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use std::collections::HashMap;
use std::net::SocketAddr;

pub async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    // 解析请求行
    let (method, path) = match parse_request_line(&mut reader).await {
        Ok(result) => result,
        Err(_) => return handle_invalid_request(&mut stream, addr).await,
    };
    // 解析请求头
    let headers = parse_headers(&mut reader).await?;
    // 读取请求体
    let content_length = headers
        .get("content-length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = read_body(&mut reader, content_length).await?;
    // 创建日志条目并路由请求
    let log = crate::utils::LogEntry::new(method.clone(), path.clone(), Some(addr));
    route_request(&mut stream, &method, &path, &body, &log).await
}
async fn parse_request_line(reader: &mut BufReader<&mut TcpStream>) -> std::io::Result<(String, String)> {
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;
    
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() != 3 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid request line"));
    }
    
    Ok((parts[0].to_string(), parts[1].to_string()))
}
async fn parse_headers(reader: &mut BufReader<&mut TcpStream>) -> std::io::Result<HashMap<String, String>> {
    let mut headers = HashMap::new();
    let mut line = String::new();
    
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 || matches!(line.as_str(), "\r\n" | "\n") {
            break;
        }
        
        if let Some((key, value)) = line.trim_end_matches(&['\r', '\n']).split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    
    Ok(headers)
}
async fn read_body(reader: &mut BufReader<&mut TcpStream>, content_length: usize) -> std::io::Result<String> {
    const MAX_BODY_SIZE: usize = 1024 * 1024;
    let body_size = content_length.min(MAX_BODY_SIZE);
    
    if body_size == 0 {
        return Ok(String::new());
    }
    
    let mut body = vec![0; body_size];
    reader.read_exact(&mut body).await?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}
async fn handle_invalid_request(stream: &mut TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    let log = crate::utils::LogEntry::new("UNKNOWN".to_string(), "INVALID".to_string(), Some(addr));
    log.log("400");
    crate::utils::send_400_response(stream, b"Invalid request").await
}
async fn route_request(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    log: &crate::utils::LogEntry,
) -> std::io::Result<()> {
    match method {
        "GET" => crate::handlers::handle_get_request(stream, path, log).await,
        "POST" => crate::handlers::handle_post_request(stream, path, body, log).await,
        _ => {
            crate::utils::send_405_response(stream, b"Method Not Allowed").await?;
            log.log("405");
            Ok(())
        }
    }
}