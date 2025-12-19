use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use std::collections::HashMap;
use std::net::SocketAddr;

pub async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    // 解析请求方法、路径和HTTP版本
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() != 3 {
        let log = crate::utils::LogEntry::new("UNKNOWN".to_string(), "INVALID".to_string(), Some(addr));
        log.log("400");
        return crate::utils::send_400_response(&mut stream, b"Invalid request").await;
    }

    let method = parts[0];
    let path = parts[1];

    // 读取并解析请求头
    let mut headers = HashMap::new();
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await? {
            0 => return Ok(()), // 客户端关闭连接
            _ => {
                match line.as_str() {
                    "\r\n" | "\n" => break,
                    _ => {
                        if let Some((key, value)) = line.trim_end_matches(&['\r', '\n']).split_once(':') {
                            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    // 读取请求体
    let content_length: usize = headers
        .get("content-length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // 限制最大请求体大小为1MB
    const MAX_BODY_SIZE: usize = 1024 * 1024;
    let body_size = content_length.min(MAX_BODY_SIZE);
    let mut body = Vec::with_capacity(body_size);
    
    if body_size > 0 {
        body.resize(body_size, 0);
        reader.read_exact(&mut body).await?;
    }
    let body_str = String::from_utf8_lossy(&body).into_owned();

    // 创建日志条目
    let log = crate::utils::LogEntry::new(method.to_string(), path.to_string(), Some(addr));

    // 路由处理
    match method {
        "GET" => crate::handlers::handle_get_request(&mut stream, path, &log).await,
        "POST" => crate::handlers::handle_post_request(&mut stream, path, &body_str, &log).await,
        _ => {
            crate::utils::send_405_response(&mut stream, b"Method Not Allowed");
            log.log("405");
            Ok(())
        }
    }
}
