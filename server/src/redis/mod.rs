// src/redis/mod.rs
use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisClient {
    pool: Pool<RedisConnectionManager>,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> RedisResult<Self> {
        let manager = RedisConnectionManager::new(redis_url)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(Self { pool })
    }

    async fn get_conn(&self) -> RedisResult<PooledConnection<'_, RedisConnectionManager>> {
        self.pool.get().await.map_err(|e| redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string())))
    }

    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.set(key, value).await
    }

    pub async fn set_with_expire(&self, key: &str, value: &str, ttl_seconds: i64) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.set::<&str, &str, ()>(key, value).await?;
        conn.expire(key, ttl_seconds).await
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.get_conn().await?;
        conn.get(key).await
    }

    pub async fn get_and_refresh(&self, key: &str, ttl_seconds: i64) -> RedisResult<Option<String>> {
        let mut conn = self.get_conn().await?;
        let result: Option<String> = conn.get(key).await?;
        if result.is_some() {
            conn.expire::<_, ()>(key, ttl_seconds).await?;
        }
        Ok(result)
    }

    pub async fn del(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.del(key).await
    }

    pub async fn sadd(&self, key: &str, member: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.sadd(key, member).await
    }

    pub async fn srem(&self, key: &str, member: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.srem(key, member).await
    }

    pub async fn smembers(&self, key: &str) -> RedisResult<Vec<String>> {
        let mut conn = self.get_conn().await?;
        conn.smembers(key).await
    }

    pub async fn expire(&self, key: &str, seconds: i64) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.expire(key, seconds).await
    }
}

pub type SharedRedis = Arc<RedisClient>;
