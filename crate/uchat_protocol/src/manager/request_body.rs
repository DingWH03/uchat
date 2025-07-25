use crate::RoleType;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Debug, ToSchema)]
pub struct ChangeRoleRequest {
    pub user_id: u32,
    pub new_role: RoleType,
}

/// 获取用户详细信息
#[derive(Deserialize, Debug, IntoParams)]
pub struct CheckUserDetailRequest {
    pub user_id: u32,
}

/// 删除用户
#[derive(Deserialize, Debug, IntoParams)]
pub struct DeleteUserRequest {
    pub user_id: u32,
}

/// 删除登录会话
#[derive(Deserialize, Debug, IntoParams)]
pub struct DeleteSessionRequest {
    pub session_id: String,
}

/// 删除好友关系
#[derive(Deserialize, Debug, IntoParams)]
pub struct DeleteFriendshipRequest {
    pub user_id: u32,
    pub friend_id: u32,
}

/// 获取某用户的好友
#[derive(Deserialize, Debug, IntoParams)]
pub struct GetFriendsRequest {
    pub user_id: u32,
}

/// 获取近期聊天记录
#[derive(Deserialize, Debug, IntoParams)]
pub struct GetRecentMessageRequest {
    pub count: u32,
    pub offset: u32,
}

/// 获取用户近期聊天记录
#[derive(Deserialize, Debug, IntoParams)]
pub struct GetUserRecentMessageRequest {
    pub count: u32,
    pub offset: u32,
    pub user_id: u32,
}

/// 删除单条消息
#[derive(Deserialize, Debug, IntoParams)]
pub struct DeleteMessageRequest {
    pub message_id: u64,
}

/// 获取单条消息
#[derive(Deserialize, Debug, IntoParams)]
pub struct GetMessageRequest {
    pub message_id: u64,
}
