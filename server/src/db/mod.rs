pub mod error;
pub mod factory;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgresql;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    db::error::DBError,
    protocol::{
        request::{PatchUserRequest, UpdateUserRequest}, FullPrivateMessage, GroupDetailedInfo, GroupSimpleInfo, ManagerUserSimpleInfo, MessageType, PreviewPrivateMessage, RoleType, SessionMessage, UpdateTimestamps, UserDetailedInfo, UserSimpleInfo
    },
};

#[async_trait]
pub trait InitDB: Send + Sync {
    /// 初始化（构造函数一般在实现结构体中写，这里可选）
    async fn init(database_url: &str) -> Result<Self>
    where
        Self: Sized;
}

#[async_trait]
pub trait UserDB: Send + Sync {
    /// 查询用户密码哈希
    async fn get_password_hash(&self, id: u32) -> Result<String, DBError>;
    /// 查询用户密码哈希以及role
    async fn get_user_password_and_role(&self, user_id: u32)
    -> Result<(String, RoleType), DBError>;
    /// 更新用户密码
    async fn update_password(&self, id: u32, new_password_hash: &str) -> Result<(), DBError>;
    /// 获取用户的好友和群组更新时间（返回时间戳，单位：秒）
    async fn get_update_timestamps(&self, id: u32) -> Result<UpdateTimestamps, DBError>;
    /// 创建新用户
    async fn new_user(&self, username: &str, password_hash: &str) -> Result<u32, DBError>;
    /// 删除用户
    async fn delete_user(&self, id: u32) -> Result<(), DBError>;
    /// 完整更新用户信息
    async fn update_user_info_full(
        &self,
        id: u32,
        update: UpdateUserRequest,
    ) -> Result<(), DBError>;
    /// 部分更新用户信息
    async fn update_user_info_partial(
        &self,
        id: u32,
        patch: PatchUserRequest,
    ) -> Result<(), DBError>;
    async fn update_user_avatar(
        &self,
        id: u32,
        avatar_url: &str,
    ) -> Result<(), DBError>;
    /// 根据id查找用户详细信息
    async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>, DBError>;
    // 设置UserDetailedInfo用户信息，当前用户信息较少，以后会考虑单独设置某一部分，例如个性签名，头像等
    // async fn set_userinfo(&self, id: u32, userinfo: UserDetailedInfo) -> Result<()>;
}

#[async_trait]
pub trait FriendDB: Send + Sync {
    /// 根据user_id🔍好友列表，一般是自己查找自己的好友列表
    async fn get_friends(&self, user_id: u32) -> Result<Vec<UserSimpleInfo>, DBError>;

    /// 添加好友，user_id是发送者的id，friend_id是接收者的id
    /// 直接双向成为好友，暂不支持请求与同意机制
    async fn add_friend(&self, user_id: u32, friend_id: u32) -> Result<(), DBError>;
    /// 删除好友
    async fn delete_friendship(&self, user_id: u32, friend_id: u32) -> Result<(), DBError>;
}

#[async_trait]
pub trait GroupDB: Send + Sync {
    /// 根据user_id🔍群组列表，一般是自己查找自己的群组列表
    async fn get_groups(&self, user_id: u32) -> Result<Vec<GroupSimpleInfo>, DBError>;
    /// 根据group_id获取群聊详细信息
    async fn get_groupinfo(&self, group_id: u32) -> Result<Option<GroupDetailedInfo>, DBError>;
    /// 根据group_id🔍群组成员列表
    async fn get_group_members(&self, group_id: u32) -> Result<Vec<UserSimpleInfo>, DBError>;
    /// 创建群组
    async fn create_group(
        &self,
        user_id: u32,
        group_name: &str,
        members: Vec<u32>,
    ) -> Result<u32, DBError>;
    /// 添加群组成员，user_id是发送者的id，group_id是接收者的id
    async fn join_group(&self, user_id: u32, group_id: u32) -> Result<(), DBError>;
    /// 退出群聊
    async fn leave_group(&self, user_id: u32, group_id: u32) -> Result<(), DBError>;
}

#[async_trait]
pub trait MessageDB: Send + Sync {
    /// 添加私聊信息聊天记录，返回消息的timestamp
    async fn add_message(
        &self,
        sender: u32,
        receiver: u32,
        message_type: MessageType,
        message: &str,
    ) -> Result<i64, DBError>;
    /// 添加离线消息记录
    async fn add_offline_message(
        &self,
        receiver_id: u32,
        is_group: bool,
        message_id: Option<u64>,
        group_message_id: Option<u64>,
    ) -> Result<(), DBError>;
    /// 添加群聊信息聊天记录，返回消息的timestamp
    async fn add_group_message(
        &self,
        group_id: u32,
        sender: u32,
        message: &str,
    ) -> Result<i64, DBError>;
    /// 获取私聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError>;
    /// 获取群聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    async fn get_group_messages(
        &self,
        group_id: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError>;
    /// 获取某群聊最新一条消息时间戳
    async fn get_latest_timestamp_of_group(
        &self,
        group_id: u32,
    ) -> Result<Option<i64>, DBError>;
    /// 用户加入群聊的所有的群消息最后的时间戳
    async fn get_latest_timestamps_of_all_groups(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, i64>, DBError>;
    /// 当前用户所有群聊中最新的一条消息的时间戳（全局最大）
    async fn get_latest_timestamp_of_all_group_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<i64>, DBError>;
    /// 某个群某时间之后的消息
    async fn get_group_messages_after_timestamp(
        &self,
        group_id: u32,
        after: i64,
    ) -> Result<Vec<SessionMessage>, DBError>;
    // 当前用户所有群某时间之后的消息
    async fn get_all_group_messages_after_timestamp(
        &self,
        user_id: u32,
        after: i64,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError>;
    /// 获取与某个用户的最后一条私聊消息时间戳
    async fn get_latest_timestamp_with_user(
        &self,
        user1_id: u32,
        user2_id: u32,
    ) -> Result<Option<i64>, DBError>;
    /// 获取当前用户所有私聊会话的最后时间戳（按对方用户 ID 映射）
    async fn get_latest_timestamps_of_all_private_chats(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, i64>, DBError>;
    /// 获取当前用户所有私聊中最新的一条消息时间戳（全局最大）
    async fn get_latest_timestamp_of_all_private_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<i64>, DBError>;
    /// 获取与某个用户某时间之后的聊天记录（时间递增）
    async fn get_private_messages_after_timestamp(
        &self,
        user1_id: u32,
        user2_id: u32,
        after: i64,
    ) -> Result<Vec<SessionMessage>, DBError>;
    /// 获取所有私聊消息中某时间之后的所有聊天记录（带对方 ID）
    async fn get_all_private_messages_after_timestamp(
        &self,
        user_id: u32,
        after: i64,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError>;
}

#[async_trait]
pub trait ManagerDB: Send + Sync {
    /// 获取所有用户数量(包括管理员和普通用户)
    async fn get_user_count(&self) -> Result<u32, DBError>;
    /// 获取所有的用户
    async fn get_all_user(&self) -> Result<Vec<ManagerUserSimpleInfo>, DBError>;
    /// 改变用户身份
    async fn change_user_role(&self, userid: u32, role: RoleType) -> Result<(), DBError>;
    /// 获取全服务器近N条聊天记录
    async fn get_recent_messages(
        &self,
        count: u32,
        offset: u32,
    ) -> Result<Vec<PreviewPrivateMessage>, DBError>;
    /// 获取某用户近N条聊天记录
    async fn get_user_recent_messages(
        &self,
        count: u32,
        offset: u32,
        user_id: u32,
    ) -> Result<Vec<PreviewPrivateMessage>, DBError>;
    /// 删除某条聊天记录
    async fn delete_private_message(&self, message_id: u64) -> Result<u64, DBError>;
    /// 获取一个私聊聊天记录
    async fn get_private_message(&self, message_id: u64) -> Result<FullPrivateMessage, DBError>;
}

// 综合 trait，将所有子 trait 组合起来
#[async_trait]
pub trait DB: InitDB + UserDB + FriendDB + GroupDB + MessageDB + ManagerDB {}

impl<T> DB for T where T: InitDB + UserDB + FriendDB + GroupDB + MessageDB + ManagerDB {}
