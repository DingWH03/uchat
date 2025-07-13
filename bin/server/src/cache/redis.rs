use async_trait::async_trait;
use std::sync::Arc;
use crate::redis::SharedRedis;
use super::CacheManagerTrait;

pub struct CacheConfig {
    pub redis: SharedRedis,
    pub expire_secs: Option<i64>, // 可选过期时间
}

pub struct RedisCacheManager {
    redis: SharedRedis,
    expire_secs: Option<i64>,
}

#[async_trait]
impl CacheManagerTrait for RedisCacheManager {
    type Config = CacheConfig;

    async fn new_with_config(config: Self::Config) -> Arc<Self> {
        Arc::new(Self {
            redis: config.redis,
            expire_secs: config.expire_secs,
        })
    }

    async fn get_group_members(&self, group_id: u32) -> Option<Vec<u32>> {
        let key = format!("group:{}:members", group_id);
        match self.redis.smembers(&key).await {
            Ok(values) if !values.is_empty() => {
                if let Some(expire) = self.expire_secs {
                    let _ = self.redis.expire(&key, expire).await;
                }
                Some(values.into_iter().filter_map(|s| s.parse().ok()).collect())
            }
            _ => None,
        }
    }


    async fn set_group_members(&self, group_id: u32, members: Vec<u32>) {
        let key = format!("group:{}:members", group_id);
        let _ = self.redis.del(&key).await;
        if !members.is_empty() {
            let members_str: Vec<String> = members.iter().map(|id| id.to_string()).collect();
            let _ = self.redis.sadd_multiple(&key, &members_str).await;
            if let Some(expire) = self.expire_secs {
                let _ = self.redis.expire(&key, expire).await;
            }
        }
    }

    async fn invalidate_group_members(&self, group_id: u32) {
        let key = format!("group:{}:members", group_id);
        let _ = self.redis.del(&key).await;
    }

    async fn get_friends(&self, user_id: u32) -> Option<Vec<u32>> {
        let key = format!("user:{}:friends", user_id);
        match self.redis.smembers(&key).await {
            Ok(values) if !values.is_empty() => {
                if let Some(expire) = self.expire_secs {
                    let _ = self.redis.expire(&key, expire).await;
                }
                Some(values.into_iter().filter_map(|s| s.parse().ok()).collect())
            }
            _ => None,
        }
    }


    async fn set_friends(&self, user_id: u32, friends: Vec<u32>) {
        let key = format!("user:{}:friends", user_id);
        let _ = self.redis.del(&key).await;
        if !friends.is_empty() {
            let friends_str: Vec<String> = friends.iter().map(|id| id.to_string()).collect();
            let _ = self.redis.sadd_multiple(&key, &friends_str).await;
            if let Some(expire) = self.expire_secs {
                let _ = self.redis.expire(&key, expire).await;
            }
        }
    }

    async fn invalidate_friends(&self, user_id: u32) {
        let key = format!("user:{}:friends", user_id);
        let _ = self.redis.del(&key).await;
    }
}