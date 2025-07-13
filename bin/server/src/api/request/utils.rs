use log::{error, debug};
use super::Request;

impl Request {
    /// 获取群组成员ID列表
    /// 先查cache，如果未命中则查数据库并写入cache
    /// 返回成员ID列表
    pub async fn get_group_member_ids(&self, group_id: u32) -> Vec<u32> {
        // 1. 先查cache
        if let Some(ids) = self.cache.get_group_members(group_id).await {
            debug!(
                "从缓存中获取群组 {} 成员列表: {:?}",
                group_id, ids
            );
            ids
        } else {
            // 2. cache未命中，查数据库并写入cache
            match self.db.get_group_members(group_id).await {
                Err(e) => {
                    error!("获取群组 {} 成员失败: {:?}", group_id, e);
                    return Vec::new();
                }
                Ok(members) => {
                    let ids: Vec<u32> = members.iter().map(|m| m.user_id).collect();
                    self.cache.set_group_members(group_id, ids.clone()).await;
                    ids
                }
            }
        }
    }
    /// 获取用户好友ID列表
    /// 先查cache，如果未命中则查数据库并写入cache
    /// 返回好友ID列表
    pub async fn get_friends_ids(&self, user_id: u32) -> Vec<u32> {
        // 1. 先查cache
        if let Some(ids) = self.cache.get_friends(user_id).await {
            debug!("从缓存中获取用户 {} 好友列表: {:?}", user_id, ids);
            ids
        } else {
            // 2. cache未命中，查数据库并写入cache
            match self.db.get_friends(user_id).await {
                Err(e) => {
                    error!("获取用户 {} 好友失败: {:?}", user_id, e);
                    return Vec::new();
                }
                Ok(friends) => {
                    let ids: Vec<u32> = friends.iter().map(|f| f.user_id).collect();
                    self.cache.set_friends(user_id, ids.clone()).await;
                    ids
                }
            }
        }
    }
}