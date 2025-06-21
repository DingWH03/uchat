use serde::{Deserialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordRequest {
    pub user_id: u32,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub userid: u32,
    pub password: String,
}

/// 好友请求具体枚举
#[derive(Deserialize, Debug)]
pub enum FriendRequestType {
    Add = 1,
    Info = 2,
}

#[derive(Deserialize, Debug, IntoParams)]
pub struct FriendRequest {
    pub id: u32,
}

/// 群聊请求具体枚举
#[derive(Deserialize, Debug)]
pub enum GroupRequestType {
    Join = 1,
    Info = 2,
    Creat = 3,
    Leave = 4,
    Member = 5
}

#[derive(Deserialize, Debug)]
pub struct GroupRequest {
    pub request_type: GroupRequestType,
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub members: Vec<u32>, // 成员ID列表
}

/// 获取聊天记录的请求
#[derive(Deserialize, Debug)]
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
