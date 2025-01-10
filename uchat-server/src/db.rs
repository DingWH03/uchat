// src/db.rs

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use anyhow::Result;
use dotenv::dotenv;
use std::env;

/// 结构体 `Database` 用于封装 MySQL 连接池
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// 异步初始化数据库连接池
    pub async fn new() -> Result<Self> {
        // 加载 .env 文件中的环境变量
        dotenv().ok();

        // 从环境变量中获取数据库连接字符串
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL 环境变量未设置");

        // 创建连接池，设置最大连接数为 20
        let pool = MySqlPoolOptions::new()
            .max_connections(20)
            .connect(&database_url).await?;

        Ok(Self { pool })
    }

    /// 查询用户密码哈希
    pub async fn get_password_hash(&self, id: u32) -> Result<Option<String>> {
        let row = sqlx::query!(
            "SELECT password_hash FROM users WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.password_hash))
    }

    pub async fn new_user(&self, username: &str, password_hash: &str) -> Result<Option<u32>> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            username,
            password_hash
        )
        .execute(&self.pool)
        .await?;
    
        // 获取插入的自增ID
        let last_insert_id = result.last_insert_id() as u32;
    
        Ok(Some(last_insert_id))
    }

    pub async fn get_username(&self, id: u32) -> Result<Option<String>> {
        let row = sqlx::query!(
            "SELECT username FROM users WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
    // println!("{:?}",row);
    
        Ok(row.map(|r| r.username))
    }

    pub async fn get_friends(&self, id: u32) -> Result<Vec<u32>> {
        let rows = sqlx::query!(
            "SELECT friend_id FROM friendships WHERE user_id = ?",
            id
        )
        .fetch_all(&self.pool)
        .await?;
    
        Ok(rows.iter().map(|r| r.friend_id).collect())
    }

    pub async fn get_groups(&self, id: u32) -> Result<Vec<u32>> {
        let rows = sqlx::query!(
            "SELECT group_id FROM group_members WHERE user_id = ?",
            id
        )
        .fetch_all(&self.pool)
        .await?;
    
        Ok(rows.iter().map(|r| r.group_id).collect())
    }
    pub async fn get_group_members(&self, group_id: u32) -> Result<Vec<u32>> {
        let rows = sqlx::query!(
            "SELECT user_id FROM group_members WHERE group_id = ?",
            group_id
        )
        .fetch_all(&self.pool)
        .await?;
    
        Ok(rows.iter().map(|r| r.user_id).collect())
    }
    pub async fn add_friend(&self, user_id: u32, friend_id: u32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO friendships (user_id, friend_id) VALUES (?, ?)",
            user_id,
            friend_id
        )
        .execute(&self.pool)
        .await?;
    
        Ok(())
    }
    pub async fn add_group(&self, user_id: u32, group_id: u32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_members (user_id, group_id) VALUES (?, ?)",
            user_id,
            group_id
        )
        .execute(&self.pool)
        .await?;
    
        Ok(())
    }

}
