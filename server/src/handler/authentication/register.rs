use axum::{response::IntoResponse, Extension, Json};
use log::{debug, warn};

use crate::{protocol::{RegisterRequest, ServerResponse}, server::AppState};


pub async fn handle_register(Extension(state): Extension<AppState>, Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    debug!("处理注册请求: {:?}", payload);
    let request = state.request.lock().await;
    let register_result = request.register(&payload.username, &payload.password).await;
    let response = match register_result {
        Ok(user_id) => ServerResponse::RegisterResponse {
            status: true,
            message: format!(
                "注册成功，你的id为{}",
                user_id.map_or("出错".to_string(), |id| id.to_string())
            ),
        },
        Err(e) => {
            warn!("注册失败: {}", e);
            ServerResponse::RegisterResponse {
                status: false,
                message: format!("注册失败: {}", e),
            }
        }
    };
    debug!("注册响应: {:?}", response);
    axum::Json(response)
}