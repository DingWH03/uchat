use serde::{Deserialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Debug, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct PasswordRequest {
    pub user_id: u32,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct LoginRequest {
    pub userid: u32,
    pub password: String,
}

#[derive(Deserialize, Debug, IntoParams, ToSchema)]
pub struct FriendRequest {
    pub id: u32,
}

#[derive(Deserialize, Debug, IntoParams, ToSchema)]
pub struct GroupRequest {
    pub id: u32,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub members: Vec<u32>, // 成员ID列表
}

/// 获取聊天记录的请求
#[derive(Deserialize, Debug, IntoParams)]
pub struct MessageRequest {
    pub id: u32, // 用户ID或群组ID
    pub offset: u32, // 偏移量，用于分页
}



#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub username: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchUserRequest {
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckStatusRequest {
    pub user_ids: Vec<u32>, // 用户ID列表
}