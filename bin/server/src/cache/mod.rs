// src/cache/mod.rs
#[cfg(not(feature = "redis-support"))]
pub mod memory;
#[cfg(not(feature = "redis-support"))]
pub use crate::cache::memory::CacheConfig;
#[cfg(feature = "redis-support")]
pub mod redis;
#[cfg(feature = "redis-support")]
pub use crate::cache::redis::CacheConfig;

use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait CacheManagerTrait: Send + Sync {
    type Config: Send + Sync;

    async fn new_with_config(config: Self::Config) -> Arc<Self>
    where
        Self: Sized;

    // 群成员缓存
    async fn get_group_members(&self, group_id: u32) -> Option<Vec<u32>>;
    async fn set_group_members(&self, group_id: u32, members: Vec<u32>);
    async fn invalidate_group_members(&self, group_id: u32);

    // 好友缓存
    async fn get_friends(&self, user_id: u32) -> Option<Vec<u32>>;
    async fn set_friends(&self, user_id: u32, friends: Vec<u32>);
    async fn invalidate_friends(&self, user_id: u32);
}

/// 工厂函数，根据 feature 选择 CacheManager 实现
pub async fn create_cache_manager(
    config: CacheConfig,
) -> Arc<dyn CacheManagerTrait<Config = CacheConfig>> {
    #[cfg(not(feature = "redis-support"))]
    {
        let manager = memory::MemoryCacheManager::new_with_config(config).await;
        manager
    }
    #[cfg(feature = "redis-support")]
    {
        redis::RedisCacheManager::new_with_config(config).await
    }
}