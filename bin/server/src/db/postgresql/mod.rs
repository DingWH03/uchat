// src/db/postgresql/mod.rs
mod friend;
mod group;
mod message;
mod user;
mod manager;

use crate::db::InitDB;

use anyhow::Result;
use async_trait::async_trait;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// 结构体 `PgSqlDB` 用于封装 Postgresql 连接池
pub struct PgSqlDB {
    pool: PgPool,
}

#[async_trait]
impl InitDB for PgSqlDB {
    /// 异步初始化数据库连接池
    async fn init(database_url: &str) -> Result<Self> {
        // 创建连接池，设置最大连接数为 20
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }
}
