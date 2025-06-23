use axum::{response::IntoResponse, Extension, Json};
use log::{debug};

use crate::{protocol::{request::{RegisterRequest, RequestResponse}, Empty}, server::AppState};

/// 注册处理函数
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册成功", body = RequestResponse<u32>),
        (status = 400, description = "用户名或密码格式错误", body = RequestResponse<Empty>),
        (status = 500, description = "服务器内部错误", body = RequestResponse<Empty>),
    ),
    tag = "request/auth"
)]
pub async fn handle_register(Extension(state): Extension<AppState>, Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    debug!("处理注册请求: {:?}", payload);
    let request = state.request.lock().await;
    request.register(&payload.username, &payload.password).await.into_response()
}