use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ManagerResponse<T> {
    pub status: bool,
    pub code: u16,
    pub message: String, // 提示信息统一为 String
    pub data: Option<T>, // 仅在成功时存在数据
}

impl<T> ManagerResponse<T> {
    pub fn ok(message: impl Into<String>, data: T) -> Self {
        Self {
            status: true,
            code: 200,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            status: false,
            code: 500,
            message: message.into(),
            data: None,
        }
    }
}




#[derive(Serialize)]
pub struct OnlineUserTree {
    pub users: HashMap<u32, Vec<UserSessionInfo>>,
}

#[derive(Serialize)]
pub struct UserSessionInfo {
    pub session_id: String,
    pub user_id: u32,
    pub created_at: NaiveDateTime,
    pub ip: Option<String>,
}
