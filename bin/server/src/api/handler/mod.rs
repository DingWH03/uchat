pub mod manager;
pub mod request;
pub mod ws;

use crate::server::AppState;
use axum::Extension;
use axum::response::IntoResponse;
use log::debug;
use uchat_model::{Empty, request::RequestResponse};

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
        (status = 200, description = "成功响应", body = RequestResponse<Empty>)
    )
)]
pub async fn ping(Extension(state): Extension<AppState>) -> impl IntoResponse {
    // 接受http发出的ping请求
    // 测试request接口与客户端之间通信
    debug!("响应ping请求");
    let request = state.request.lock().await;
    request.ping().await.into_response()
}
