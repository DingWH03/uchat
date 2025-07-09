// src/redis/mod.rs

// 引入所需的外部库
use bb8::{Pool, PooledConnection}; // bb8 连接池相关
use bb8_redis::RedisConnectionManager; // Redis 连接管理器
use redis::{AsyncCommands, AsyncIter, RedisResult}; // Redis 异步命令、异步迭代器和结果类型
use std::sync::Arc; // 线程安全的引用计数指针

/// Redis 客户端结构体，封装了 Redis 连接池
#[derive(Clone)]
pub struct RedisClient {
    pool: Pool<RedisConnectionManager>, // Redis 连接池
}

impl RedisClient {
    /// 创建新的 Redis 客户端，初始化连接池
    pub async fn new(redis_url: &str) -> RedisResult<Self> {
        let manager = RedisConnectionManager::new(redis_url)?; // 创建 Redis 连接管理器
        let pool = Pool::builder().build(manager).await?; // 构建连接池
        Ok(Self { pool })
    }

    /// 从连接池获取一个 Redis 连接
    async fn get_conn(&self) -> RedisResult<PooledConnection<'_, RedisConnectionManager>> {
        self.pool.get().await.map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::IoError, "pool error", e.to_string()))
        })
    }

    /// 设置指定 key 的值
    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?; // 获取连接
        conn.set(key, value).await // 执行 set 命令
    }

    /// 设置指定 key 的值，并设置过期时间（秒）
    pub async fn set_with_expire(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: i64,
    ) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.set::<&str, &str, ()>(key, value).await?; // 先设置值
        conn.expire(key, ttl_seconds).await // 再设置过期时间
    }

    /// 获取指定 key 的值，返回 Option<String>
    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.get_conn().await?;
        conn.get(key).await
    }

    /// 获取指定 key 的值，并刷新其过期时间
    pub async fn get_and_refresh(
        &self,
        key: &str,
        ttl_seconds: i64,
    ) -> RedisResult<Option<String>> {
        let mut conn = self.get_conn().await?;
        let result: Option<String> = conn.get(key).await?; // 获取值
        if result.is_some() {
            conn.expire::<_, ()>(key, ttl_seconds).await?; // 如果存在则刷新过期时间
        }
        Ok(result)
    }

    /// 获取 Redis 中指定集合键的元素数量
    pub async fn scard(&self, key: &str) -> RedisResult<usize> {
        let mut conn = self.get_conn().await?;
        conn.scard(key).await
    }

    /// 批量获取多个 key 的值，返回每个 key 对应的 Option<String>
    pub async fn mget(&self, keys: &[String]) -> RedisResult<Vec<Option<String>>> {
        let mut conn = self.get_conn().await?;
        conn.get(keys).await
    }

    /// 扫描 Redis 中匹配模式的所有 key，非阻塞，优于 KEYS 命令
    pub async fn scan_keys(&self, pattern: &str) -> RedisResult<Vec<String>> {
        let mut conn = self.get_conn().await?;
        let mut iter: AsyncIter<String> = conn.scan_match(pattern).await?; // 创建异步迭代器
        let mut keys = Vec::new();
        while let Some(res) = iter.next_item().await {
            match res {
                Ok(key) => keys.push(key), // 收集 key
                Err(e) => return Err(e),   // 出错则返回错误
            }
        }
        Ok(keys)
    }

    /// 删除指定 key
    pub async fn del(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.del(key).await
    }

    /// 向集合 key 添加一个成员
    pub async fn sadd(&self, key: &str, member: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.sadd(key, member).await
    }

    /// 向集合 key 批量添加多个成员
    pub async fn sadd_multiple(&self, key: &str, members: &[String]) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        if members.is_empty() {
            return Ok(());
        }
        conn.sadd(key, members).await
    }

    /// 从集合 key 移除一个成员
    pub async fn srem(&self, key: &str, member: &str) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.srem(key, member).await
    }

    /// 获取集合 key 的所有成员
    pub async fn smembers(&self, key: &str) -> RedisResult<Vec<String>> {
        let mut conn = self.get_conn().await?;
        conn.smembers(key).await
    }

    /// 设置指定 key 的过期时间（秒）
    pub async fn expire(&self, key: &str, seconds: i64) -> RedisResult<()> {
        let mut conn = self.get_conn().await?;
        conn.expire(key, seconds).await
    }
}

/// 共享的 Redis 客户端类型，便于多线程环境下使用
pub type SharedRedis = Arc<RedisClient>;
