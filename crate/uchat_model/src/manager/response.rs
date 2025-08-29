use axum::{Json, http::StatusCode, response::IntoResponse};
use chrono::DateTime;
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

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

    /// 发生错误
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            status: false,
            code: 500,
            message: message.into(),
            data: None,
        }
    }

    /// 未认证
    pub fn unauthorized() -> Self {
        Self {
            status: false,
            code: 401,
            message: "认证失败".to_string(),
            data: None,
        }
    }

    /// 权限不足
    pub fn forbidden() -> Self {
        Self {
            status: false,
            code: 403,
            message: "权限不足".to_string(),
            data: None,
        }
    }
}

impl<T: serde::Serialize> IntoResponse for ManagerResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK);
        (status, Json(self)).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct UserSessionInfo {
    pub session_id: String,
    pub user_id: u32,
    #[schema(example = "2025-06-20T15:30:00", value_type = String)]
    pub created_at: DateTime<chrono::Utc>,
    pub ip: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct OnlineUserTree {
    pub users: HashMap<u32, Vec<UserSessionInfo>>,
}
