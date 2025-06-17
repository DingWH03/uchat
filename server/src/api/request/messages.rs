// api/request/messages.rs

use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::Request;
use crate::{db::error::DBError, protocol::SessionMessage};

impl Request {
    /// 获取群聊聊天记录
    pub async fn get_group_messages(
        &self,
        group_id: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError> {
        self.db
            .get_group_messages(group_id, offset)
            .await
    }
    /// 获取私聊聊天记录
    pub async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError> {
        self.db
            .get_messages(sender, receiver, offset)
            .await
    }
    /// 获取某群聊最新一条消息时间戳
    pub async fn get_latest_timestamp_of_group(
        &self,
        group_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamp_of_group(group_id)
            .await
    }
    /// 获取用户所有群聊最新一条消息时间戳
    pub async fn get_latest_timestamps_of_all_groups(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamps_of_all_groups(user_id)
            .await
    }
    /// 当前用户所有群聊中最新的一条消息的时间戳（全局最大）
    pub async fn get_latest_timestamp_of_all_group_messages(
        &self,
        group_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamp_of_all_group_messages(group_id)
            .await
    }
    /// 某个群某时间之后的消息
    pub async fn get_group_messages_after_timestamp(
        &self,
        group_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>, DBError> {
        self.db
            .get_group_messages_after_timestamp(group_id, after)
            .await
    }
    /// 当前用户所有群某时间之后的消息
    pub async fn get_all_group_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError> {
        self.db
            .get_all_group_messages_after_timestamp(user_id, after)
            .await
    }
    /// 获取与某个用户的最后一条私聊消息时间戳
    pub async fn get_latest_timestamp_with_user(
        &self,
        my_id: u32,
        friend_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamp_with_user(my_id, friend_id)
            .await
    }
    /// 获取当前用户所有私聊会话的最后时间戳（按对方用户 ID 映射）
    pub async fn get_latest_timestamps_of_all_private_chats(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamps_of_all_private_chats(user_id)
            .await
    }
    /// 获取当前用户所有私聊中最新的一条消息时间戳（全局最大）
    pub async fn get_latest_timestamp_of_all_private_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        self.db
            .get_latest_timestamp_of_all_private_messages(user_id)
            .await
    }
    /// 获取与某个用户某时间之后的聊天记录（时间递增）
    pub async fn get_private_messages_after_timestamp(
        &self,
        my_id: u32,
        friend_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>, DBError> {
        self.db
            .get_private_messages_after_timestamp(my_id, friend_id, after)
            .await
    }
    /// 获取所有私聊消息中某时间之后的所有聊天记录（带对方 ID）
    pub async fn get_all_private_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError> {
        self.db
            .get_all_private_messages_after_timestamp(user_id, after)
            .await
    }
}
