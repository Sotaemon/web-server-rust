use crate::http::{Request, Response};
use std::io::BufReader;
use std::net::TcpStream;

pub fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 BufReader
    let mut reader = BufReader::new(&stream);

    // 2. 完整解析
    let req = match Request::parse(&mut reader) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("parse error: {}", e);
            Response::bad_request().write_to(&mut stream)?;
            return Ok(());
        }
    };

    // 3. 简单路由
    let resp = match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => Response::ok_html("<h1>Hello from scratch!</h1>"),
        ("GET", "/ping") => Response::ok_plain("pong\n"),
        ("POST", "/echo") => {
            // 把 body 原样返回
            Response::build(200, "application/octet-stream", &req.body)
        }
        _ => Response::not_found(),
    };

    // 4. 回写
    resp.write_to(&mut stream)?;
    Ok(())
}
