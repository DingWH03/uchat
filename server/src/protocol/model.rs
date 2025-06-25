use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use sqlx::Type;
use utoipa::ToSchema;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSimpleInfo {
    pub user_id: u32,
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
    pub online: bool
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
    pub sender_id: u32,
    pub message: String,
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub timestamp: NaiveDateTime,
}

 #[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct GroupSessionMessage {
    pub group_id: u32,
    pub sender_id: u32,
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub timestamp: NaiveDateTime,
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
    pub message_type: MessageType,        // 可改为 enum 类型（如 MessageType 枚举）更安全
    pub message_preview: String,     // message 前 100 字符
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub timestamp: NaiveDateTime,
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
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub timestamp: chrono::NaiveDateTime,
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

impl ToString for RoleType {
    fn to_string(&self) -> String {
        match self {
            RoleType::Admin => "admin".to_string(),
            RoleType::User => "user".to_string(),
            RoleType::Invalid => "invalid".to_string(),
        }
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

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::Text => "text".to_string(),
            MessageType::Image => "image".to_string(),
            MessageType::File => "file".to_string(),
            MessageType::Video => "video".to_string(),
            MessageType::Audio => "audio".to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Empty;

#[derive(Serialize, ToSchema)]
pub struct UpdateTimestamps {
    pub friends_updated_at: i64,
    pub groups_updated_at: i64,
}
