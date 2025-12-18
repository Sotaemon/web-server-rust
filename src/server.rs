use std::io::Result;
use tokio::net::TcpListener;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(address: &str) -> Result<Self> {
        let listener = TcpListener::bind(address).await?;
        let addr = listener.local_addr()?;
        println!("Server is starting, listening on {}", addr);
        Ok(Self { listener })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tokio::spawn(async move {
                        if let Err(e) = crate::http::handle_connection(stream, addr).await {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}
