use axum::extract::ws::Message;
use log::{debug};
use uchat_model::{event::content::public::PublicEvent, message::ServerMessage};
use crate::api::error::RequestError;

use super::Request;

impl Request {
    /// 获取群组成员ID列表
    /// 先查cache，如果未命中则查数据库并写入cache
    /// 返回成员ID列表
    pub async fn get_group_member_ids(&self, group_id: u32) -> Result<Vec<u32>, RequestError> {
        // 1. 先查cache
        if let Some(ids) = self.cache.get_group_members(group_id).await {
            debug!(
                "从缓存中获取群组 {} 成员列表: {:?}",
                group_id, ids
            );
            Ok(ids)
        } else {
            // 2. cache未命中，查数据库并写入cache
            let groups =  self.db.get_group_members(group_id).await?;
            let ids: Vec<u32> = groups.iter().map(|g| g.user_id).collect();
            self.cache.set_group_members(group_id, ids.clone()).await;
            Ok(ids)
        }
    }
    /// 获取用户好友ID列表
    /// 先查cache，如果未命中则查数据库并写入cache
    /// 返回好友ID列表
    pub async fn get_friends_ids(&self, user_id: u32) -> Result<Vec<u32>, RequestError> {
        // 1. 先查cache
        if let Some(ids) = self.cache.get_friends(user_id).await {
            debug!("从缓存中获取用户 {} 好友列表: {:?}", user_id, ids);
            Ok(ids)
        } else {
            // 2. cache未命中，查数据库并写入cache
            let friends = self.db.get_friends(user_id).await.map_err(RequestError::from)?;
            let ids: Vec<u32> = friends.iter().map(|f| f.user_id).collect();
            self.cache.set_friends(user_id, ids.clone()).await;
            Ok(ids)
        }
    }

    /// 向该用户好友广播事件
    pub async fn event_broadcast(&self, user_id: u32, event: PublicEvent) -> Result<(), RequestError> {
        let message = ServerMessage::Event(event);
        let json = serde_json::to_string(&message).map_err(RequestError::from)?;
        let friends = self.get_friends_ids(user_id).await?;
        for friend in friends {
                    self.send_to_user(
                        friend,
                        Message::Text(axum::extract::ws::Utf8Bytes::from(&json)),
                    )
                    .await;
                }
        Ok(())
    }
}