use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// 完整请求
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    /// 入口：从 TcpStream 里解析出完整 Request
    pub fn parse(reader: &mut BufReader<&TcpStream>) -> Result<Self, Box<dyn std::error::Error>> {
        // 注意：head_str 应该是从 reader 中读取的头部字符串（此处原逻辑不完整）
        // 示例补全伪代码（实际需按协议逐行读取）：
        let mut head_str = String::new();
        reader.read_line(&mut head_str)?; // 至少读第一行用于测试

        // 2. 按行拆分
        let mut lines = head_str.lines();

        // 3. 请求行
        let req_line = lines.next().ok_or("empty request line")?;
        let mut parts = req_line.split_whitespace();
        let method = parts.next().ok_or("missing method")?.to_string();
        let path = parts.next().ok_or("missing path")?.to_string();
        let version = parts.next().unwrap_or("HTTP/1.0").to_string();

        // 4. 头部
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let colon_pos = line.find(':').ok_or("invalid header line")?;
            let name = line[..colon_pos].trim().to_lowercase(); // 忽略大小写
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(name, value);
        }

        // 5. 读 Body（仅支持 Content-Length）
        let body = if let Some(cl_str) = headers.get("content-length") {
            let cl: usize = cl_str.parse().map_err(|_| "invalid Content-Length")?;
            let mut buf = vec![0u8; cl];
            reader.read_exact(&mut buf)?;
            buf
        } else {
            Vec::new()
        };

        Ok(Request {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

/// 响应结构保持不变，仅加 400 快捷构造
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

    pub fn bad_request() -> Self {
        Self::build(400, "text/plain", b"400 Bad Request\n")
    }

    pub fn build(code: u16, content_type: &str, body: &[u8]) -> Self {
        let reason = match code {
            200 => "OK",
            400 => "Bad Request",
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
