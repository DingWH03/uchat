use axum::{response::IntoResponse, Extension, Json};
use log::debug;

use crate::{protocol::{request::{PasswordRequest, RequestResponse}, Empty}, server::AppState};

/// 修改密码处理函数
#[utoipa::path(
    post,
    path = "/auth/password",
    request_body = PasswordRequest,
    responses(
        (status = 200, description = "修改密码成功", body = RequestResponse<Empty>),
        (status = 400, description = "密码格式错误", body = RequestResponse<Empty>),
        (status = 401, description = "原密码错误", body = RequestResponse<Empty>),
        (status = 500, description = "服务器内部错误", body = RequestResponse<Empty>),
    ),
    tag = "request/auth"
)]
pub async fn handle_passwd(Extension(state): Extension<AppState>, Json(payload): Json<PasswordRequest>) -> impl IntoResponse {
    debug!("处理更改密码请求: {:?}", payload);
    let request_lock = state.request.lock().await;
    request_lock.change_user_password(payload.user_id, &payload.old_password, &payload.new_password).await.into_response()
}