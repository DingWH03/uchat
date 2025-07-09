mod api;
mod config;
mod db;
mod error;
#[cfg(feature = "redis-support")]
mod redis;
mod server;
mod session;
mod storage;
mod utils;
// mod event;

use crate::config::init_config;
use dotenv::dotenv;
use log::{error, info};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    // 读取环境变量
    dotenv().ok();
    match init_config() {
        Ok(config) => {
            info!("配置加载成功：{:?}", config);
        }
        Err(e) => {
            error!("加载配置出现错误：{}", e);
            return;
        }
    }

    // 启动服务
    if let Err(e) = start().await {
        error!("服务启动失败: {}", e);
    }
}

async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量中获取服务端监听字符串
    let server_url = config::get_config().server.address.clone();
    let addr = server_url.parse::<SocketAddr>()?;

    let server = server::Server::new(addr).await;

    server.run().await?;

    Ok(())
}
