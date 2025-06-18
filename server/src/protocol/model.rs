use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use sqlx::Type;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSimpleInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailedInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSimpleInfoWithStatus {
    pub base: UserSimpleInfo,
    pub online: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSimpleInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDetailedInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMessage {
    pub sender_id: u32,
    pub message: String,
    pub timestamp: NaiveDateTime,
}

 #[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupSessionMessage {
    pub group_id: u32,
    pub sender_id: u32,
    pub timestamp: NaiveDateTime,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
pub enum RoleType {
    User,
    Admin,
}

impl FromStr for RoleType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(RoleType::User),
            "admin" => Ok(RoleType::Admin),
            _ => Err(()),
        }
    }
}

impl ToString for RoleType {
    fn to_string(&self) -> String {
        match self {
            RoleType::Admin => "admin".to_string(),
            RoleType::User => "user".to_string(),
        }
    }
}

impl RoleType {
    pub fn is_admin(&self) -> bool {
        matches!(self, RoleType::Admin)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "ENUM('text', 'image', 'file', 'video', 'audio')")]
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
