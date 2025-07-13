use std::net::SocketAddr;
use crate::server::AppState;
use axum::{extract::{ConnectInfo, Json}, response::IntoResponse, Extension};
use log::debug;
use uchat_protocol::{
    Empty,
    request::{LoginRequest, RequestResponse},
};

/// 登陆处理函数
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登陆成功", body = RequestResponse<String>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器内部错误", body = RequestResponse<Empty>),
    ),
    tag = "request/auth"
)]
pub async fn handle_login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    debug!("处理登录请求: {:?} 来自 IP: {}", payload, addr.ip());

    let mut request = state.request.lock().await;
    request
        .login(payload.userid, &payload.password, addr.ip())
        .await
        .into_response()
}
