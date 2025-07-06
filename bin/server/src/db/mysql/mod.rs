// src/db/mod.rs
mod friend;
mod group;
mod manager;
mod message;
mod user;

use crate::db::InitDB;

use anyhow::Result;
use async_trait::async_trait;

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// 结构体 `MysqlDB` 用于封装 MySQL 连接池
pub struct MysqlDB {
    pool: MySqlPool,
}

#[async_trait]
impl InitDB for MysqlDB {
    /// 异步初始化数据库连接池
    async fn init(database_url: &str) -> Result<Self> {
        // 创建连接池，设置最大连接数为 20
        let pool = MySqlPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}
