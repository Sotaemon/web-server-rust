use crate::handler::handle_connection;
use std::net::TcpListener;
use std::thread;
pub struct Server {
    listener: TcpListener,
}
impl Server {
    pub fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).expect("bind failed");
        println!("Listening on http://{}", addr);
        Server { listener }
    }
    /// 主循环：accept → spawn
    pub fn run(self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    // 最简单的“每连接一线程”模型
                    thread::spawn(|| {
                        if let Err(e) = handle_connection(stream) {
                            eprintln!("connection error: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("accept error: {}", e),
            }
        }
    }
}
