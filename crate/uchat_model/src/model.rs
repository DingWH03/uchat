use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::{
    fmt::{self},
    str::FromStr,
};
use utoipa::ToSchema;

pub type UserId = u32;
pub type GroupId = u32;
pub type MessageId = u64;
pub type Ver = u32;
pub type Timestamp = i64;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSimpleInfo {
    pub user_id: UserId,
    pub username: String,
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ManagerUserSimpleInfo {
    pub user_id: u32,
    pub username: String,
    pub role: RoleType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserDetailedInfo {
    pub user_id: u32,
    pub username: String,
    pub role: RoleType,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSimpleInfoWithStatus {
    pub base: UserSimpleInfo,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupSimpleInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupDetailedInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionMessage {
    pub message_id: u32,
    pub message_type: MessageType, // enum 类型更安全
    pub sender_id: u32,
    pub message: String,
    pub timestamp: i64, // 使用 i64 存储时间戳，单位为秒
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct GroupSessionMessage {
    pub message_id: u32,
    pub message_type: MessageType, // enum 类型更安全
    pub group_id: u32,
    pub sender_id: u32,
    pub timestamp: i64, // 使用 i64 存储时间戳，单位为秒
    pub message: String,
}

/// 用于manager后台获取消息
#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PreviewPrivateMessage {
    pub id: u32,
    pub sender_id: u32,
    pub sender_username: String,
    pub receiver_id: u32,
    pub receiver_username: String,
    pub message_type: MessageType, // 可改为 enum 类型（如 MessageType 枚举）更安全
    pub message_preview: String,   // message 前 100 字符
    pub timestamp: i64,            // 使用 i64 存储时间戳，单位为秒
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct FullPrivateMessage {
    pub id: u32,
    pub sender_id: u32,
    pub sender_username: String,
    pub receiver_id: u32,
    pub receiver_username: String,
    pub message_type: MessageType,
    pub message: String, // 完整消息内容
    pub timestamp: i64,  // 使用 i64 存储时间戳，单位为秒
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize, ToSchema)]
#[sqlx(type_name = "ENUM", rename_all = "lowercase")]
#[serde(rename_all = "PascalCase")]
pub enum RoleType {
    #[sqlx(rename = "user")]
    User,
    #[sqlx(rename = "admin")]
    Admin,
    #[sqlx(rename = "invalid")]
    Invalid,
}

impl FromStr for RoleType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(RoleType::User),
            "admin" => Ok(RoleType::Admin),
            "invalid" => Ok(RoleType::Invalid),
            _ => Err(()),
        }
    }
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RoleType::Admin => "admin",
            RoleType::User => "user",
            RoleType::Invalid => "invalid",
        };
        write!(f, "{}", s)
    }
}

impl RoleType {
    pub fn is_admin(&self) -> bool {
        matches!(self, RoleType::Admin)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "mysql", sqlx(type_name = "text"))]
#[cfg_attr(feature = "postgres", sqlx(type_name = "message_type"))]
#[sqlx(rename_all = "lowercase")]
pub enum MessageType {
    #[serde(rename = "text")]
    #[sqlx(rename = "text")]
    Text,
    #[serde(rename = "image")]
    #[sqlx(rename = "image")]
    Image,
    #[serde(rename = "file")]
    #[sqlx(rename = "file")]
    File,
    #[serde(rename = "video")]
    #[sqlx(rename = "video")]
    Video,
    #[serde(rename = "audio")]
    #[sqlx(rename = "audio")]
    Audio,
}

impl FromStr for MessageType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "text" => Ok(MessageType::Text),
            "image" => Ok(MessageType::Image),
            "file" => Ok(MessageType::File),
            "video" => Ok(MessageType::Video),
            "audio" => Ok(MessageType::Audio),
            _ => Err(()),
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
            MessageType::Video => "video",
            MessageType::Audio => "audio",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, ToSchema)]
pub struct Empty;

#[derive(Serialize, ToSchema)]
pub struct UpdateTimestamps {
    pub friends_updated_at: i64,
    pub groups_updated_at: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ContactList {
    pub friends: Vec<UserSimpleInfo>,
    pub groups: Vec<GroupSimpleInfo>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserStatus {
    pub user_id: u32,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IdMessagePair {
    /// 对方用户ID 或 群聊ID
    pub id: u32,
    pub message: SessionMessage,
}
