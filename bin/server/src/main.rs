mod db;
mod server;
mod api;
mod error;
mod session;
mod storage;
#[cfg(feature = "redis-support")]
mod redis;

use std::{env, net::SocketAddr};
use log::error;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    // 读取环境变量
    dotenv().ok();

    // 启动服务
    if let Err(e) = start().await {
        error!("服务启动失败: {}", e);
    }
}


async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量中获取服务端监听字符串
    let server_url = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS 环境变量未设置");
    let addr = server_url.parse::<SocketAddr>()?;

    let server = server::Server::new(addr).await;

    server.run().await?;

    Ok(())
}
