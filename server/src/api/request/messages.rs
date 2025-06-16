// api/request/messages.rs

use crate::protocol::SessionMessage;
use super::Request;

impl Request {
    /// 获取群聊聊天记录
    pub async fn get_group_messages(
        &self,
        group_id: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, sqlx::Error> {
        self.db
            .get_group_messages(group_id, offset)
            .await
            .map_err(|e| sqlx::Error::Decode(e.into()))
    }
    /// 获取私聊聊天记录
    pub async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, sqlx::Error> {
        self.db
            .get_messages(sender, receiver, offset)
            .await
            .map_err(|e| sqlx::Error::Decode(e.into()))
    }
}