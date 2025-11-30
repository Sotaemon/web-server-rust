use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
/// 极简 Request：只存 method / path / version
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
}

impl Request {
    /// 阻塞读到第一行 "GET / HTTP/1.1\r\n" 就返回
    pub fn parse(reader: &mut BufReader<&TcpStream>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut line = String::new();
        reader.read_line(&mut line)?; // 读取第一行请求行

        let mut parts = line.trim().split_whitespace();
        let method = parts.next().ok_or("missing method")?.to_string();
        let path = parts.next().ok_or("missing path")?.to_string();
        let version = parts.next().unwrap_or("HTTP/1.0").to_string();

        // 暂时把剩余头部全部读完丢弃，后续再实现 Header map、body
        loop {
            let mut hdr = String::new();
            reader.read_line(&mut hdr)?;
            if hdr == "\r\n" || hdr.is_empty() {
                break;
            }
        }

        Ok(Request {
            method,
            path,
            version,
        })
    }
}
pub struct Response {
    status_line: String,
    headers: String,
    body: Vec<u8>,
}
impl Response {
    pub fn ok_html(body: &str) -> Self {
        Self::build(200, "text/html", body.as_bytes())
    }
    pub fn ok_plain(body: &str) -> Self {
        Self::build(200, "text/plain", body.as_bytes())
    }
    pub fn not_found() -> Self {
        Self::build(404, "text/plain", b"404 Not Found\n")
    }

    fn build(code: u16, content_type: &str, body: &[u8]) -> Self {
        let reason = match code {
            200 => "OK",
            404 => "Not Found",
            _ => "Unknown",
        };
        let status_line = format!("HTTP/1.1 {} {}\r\n", code, reason);
        let headers = format!(
            "Content-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            content_type,
            body.len()
        );
        Response {
            status_line,
            headers,
            body: body.to_vec(),
        }
    }
    pub fn write_to(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        stream.write_all(self.status_line.as_bytes())?;
        stream.write_all(self.headers.as_bytes())?;
        stream.write_all(&self.body)?;
        Ok(())
    }
}
