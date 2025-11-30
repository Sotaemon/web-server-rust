use crate::http::{Request, Response};
use std::io::{BufReader, Write};
use std::net::TcpStream;

/// 业务入口：读出 Request → 构造 Response → 写回
pub fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 解析请求行 + 头部（暂时不解析 body）
    let mut reader = BufReader::new(&stream); // 修改这里
    let req = Request::parse(&mut reader)?;
    // 2. 路由
    let resp = match req.path.as_str() {
        "/" => Response::ok_html("<h1>123</h1>"),
        "/ping" => Response::ok_plain("pong\n"),
        _ => Response::not_found(),
    };
    // 3. 写回
    resp.write_to(&mut stream)?;
    stream.flush()?;
    Ok(())
}
