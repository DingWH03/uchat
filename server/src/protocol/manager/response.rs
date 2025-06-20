use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::session_manager::SessionInfo;

#[derive(Serialize, Debug, ToSchema)]
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
#[derive(Serialize, ToSchema)]
pub struct UserSessionInfo {
    pub session_id: String,
    pub user_id: u32,
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub created_at: NaiveDateTime,
    pub ip: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct OnlineUserTree {
    pub users: HashMap<u32, Vec<UserSessionInfo>>,
}
impl From<HashMap<u32, Vec<(String, SessionInfo)>>> for OnlineUserTree {
    fn from(source: HashMap<u32, Vec<(String, SessionInfo)>>) -> Self {
        let mut users = HashMap::with_capacity(source.len());

        for (user_id, sessions) in source {
            let mut session_infos = Vec::with_capacity(sessions.len());
            for (session_id, info) in sessions {
                session_infos.push(UserSessionInfo {
                    session_id,
                    user_id,
                    created_at: info.created_at,
                    ip: info.ip,
                });
            }
            users.insert(user_id, session_infos);
        }

        OnlineUserTree { users }
    }
}


