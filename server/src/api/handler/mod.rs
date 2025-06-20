pub mod request;
pub mod ws;
pub mod manager;

use axum::response::IntoResponse;
use axum::Extension;
use crate::server::AppState;
use crate::protocol::request::ServerResponse::{self, GenericResponse};
use log::debug;

#[utoipa::path(
    get,
    path = "/",
    tag = "测试接口",
    responses(
        (status = 200, description = "成功响应", body = String)
    )
)]

pub async fn handle_request() -> &'static str {
    "Hello, world!"
}

#[utoipa::path(
    get,
    path = "/ping",
    tag = "测试接口",
    responses(
        (status = 200, description = "成功响应", body = ServerResponse)
    )
)]
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