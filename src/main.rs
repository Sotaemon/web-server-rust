mod utils;
mod server;
mod http;
mod handlers;


#[tokio::main]
// 这是一个异步主函数，返回一个 I/O 结果类型，可能包含成功或错误信息
async fn main() -> std::io::Result<()> {
    // 创建一个新的服务器实例，绑定到本地地址 127.0.0.1:10106
    // 使用 ? 操作符如果创建失败则提前返回错误
   let server = crate::server::Server::new("127.0.0.1:10106").await?;
    // 异步运行服务器，等待其完成
    // 使用 await 等待异步操作完成
   server.run().await
}