mod db;
mod protocol;
mod server;
mod handler;
mod api;
mod error;

use std::net::SocketAddr;
use log::error;


#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();

    // 启动服务
    if let Err(e) = start().await {
        error!("服务启动失败: {}", e);
    }
}


async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:25597".parse::<SocketAddr>()?;

    let server = server::Server::new(addr).await;

    server.run().await?;

    Ok(())
}
