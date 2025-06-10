use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

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