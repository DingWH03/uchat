// src/protocol.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub sender: User,
    pub receiver: User,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ClientRequest {
    #[serde(rename = "request")]
    Request {
        request: String,
    },
    #[serde(rename = "userinfo")]
    CheckUserInfo {
        user_id: u32,
    },
    #[serde(rename = "register")]
    Register {
        username: String,
        password: String,
    },
    #[serde(rename = "login")]
    Login {
        user_id: u32,
        password: String,
    },
    #[serde(rename = "send_message")]
    SendMessage {
        receiver: u32,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ServerResponse {
    #[serde(rename = "generic_response")]
    GenericResponse {
        status: String,
        message: String,
    },
    #[serde(rename = "receive_message")]
    ReceiveMessage {
        sender: u32,
        message: String,
        timestamp: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "online_users")]
    OnlineUsers {
        flag: String,
        user_ids: Vec<u32>,
    },
    #[serde(rename = "username")]
    UserName {
        user_id: u32,
        username: String,
    },
}