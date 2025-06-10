pub mod authentication;
pub mod friend;
pub mod group;
pub mod ws;
pub mod message;

use axum::response::IntoResponse;
use axum::Extension;
use crate::server::AppState;
use crate::protocol::ServerResponse::GenericResponse;
use log::debug;

pub async fn handle_request() -> &'static str {
    "Hello, world!"
}

pub async fn ping(Extension(state): Extension<AppState>) -> impl IntoResponse {
    // 接受http发出的ping请求
    // 测试request接口与客户端之间通信
    debug!("响应ping请求");
    let request = state.request.lock().await;
    let result = request.ping().await;
    let response = GenericResponse {
        status: "success".to_string(),
        message: result,
    };
    axum::Json(response).into_response()
}