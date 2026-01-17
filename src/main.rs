use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let socket = "127.0.0.1:50000";
    let source = "~/Projects/web-client-node";

    let listener = TcpListener::bind(socket).unwrap();
    println!("正在监听 Tcp 套接字 {}", socket);

    for stream in listener.incoming() {}
}
