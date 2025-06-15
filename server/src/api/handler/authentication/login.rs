use axum::{extract::Json, response::IntoResponse, Extension};
use crate::protocol::{LoginRequest, ServerResponse};
use log::debug;
use crate::server::AppState;

pub async fn handle_login(Extension(state): Extension<AppState>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    debug!("处理登录请求: {:?}", payload);
    let mut request = state.request.lock().await;
    let login_result = request.login(payload.userid, &payload.password).await;
    let response = match login_result {
        Ok(session_id) => ServerResponse::LoginResponse {
            status: true,
            message: format!(
                "{}",
                session_id
            ),
        },
        Err(e) => {
            debug!("登录失败: {}", e);
            ServerResponse::LoginResponse {
                status: false,
                message: format!("登录失败: {}", e),
            }
        }
    };
    debug!("登录响应: {:?}", response);
    axum::Json(response)
}