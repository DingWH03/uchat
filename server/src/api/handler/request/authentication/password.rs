use axum::{response::IntoResponse, Extension, Json};
use log::debug;

use crate::{protocol::request::{PasswordRequest, ServerResponse}, server::AppState};

pub async fn handle_passwd(Extension(state): Extension<AppState>, Json(payload): Json<PasswordRequest>) -> impl IntoResponse {
    debug!("处理更改密码请求: {:?}", payload);
    let request_lock = state.request.lock().await;
    let response = match request_lock.change_user_password(payload.user_id, &payload.old_password, &payload.new_password).await {
        Ok(_) => {
            debug!("密码更改成功");
            ServerResponse::GenericResponse { status: "Ok".to_string(), message: "密码更改成功".to_string() }
        }
        Err(e) => {
            debug!("密码更改失败: {}", e);
            ServerResponse::GenericResponse { status: "Err".to_string(), message: format!("密码更新失败，错误原因：{}", e) }
        }
    };
    axum::Json(response)
}