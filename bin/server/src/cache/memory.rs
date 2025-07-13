use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use super::CacheManagerTrait;

pub struct CacheConfig; // 可扩展参数

pub struct MemoryCacheManager {
    group_members: DashMap<u32, Vec<u32>>, // group_id -> member_ids
    friends: DashMap<u32, Vec<u32>>,       // user_id -> friend_ids
}

#[async_trait]
impl CacheManagerTrait for MemoryCacheManager {
    type Config = CacheConfig;

    async fn new_with_config(_: Self::Config) -> Arc<Self> {
        Arc::new(Self {
            group_members: DashMap::new(),
            friends: DashMap::new(),
        })
    }

    async fn get_group_members(&self, group_id: u32) -> Option<Vec<u32>> {
        self.group_members.get(&group_id).map(|v| v.clone())
    }

    async fn set_group_members(&self, group_id: u32, members: Vec<u32>) {
        self.group_members.insert(group_id, members);
    }

    async fn invalidate_group_members(&self, group_id: u32) {
        self.group_members.remove(&group_id);
    }

    async fn get_friends(&self, user_id: u32) -> Option<Vec<u32>> {
        self.friends.get(&user_id).map(|v| v.clone())
    }

    async fn set_friends(&self, user_id: u32, friends: Vec<u32>) {
        self.friends.insert(user_id, friends);
    }

    async fn invalidate_friends(&self, user_id: u32) {
        self.friends.remove(&user_id);
    }
}