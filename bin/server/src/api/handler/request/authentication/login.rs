use crate::server::AppState;
use axum::{Extension, extract::Json, response::IntoResponse};
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
    Extension(state): Extension<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    debug!("处理登录请求: {:?}", payload);
    let mut request = state.request.lock().await;
    request
        .login(payload.userid, &payload.password)
        .await
        .into_response()
}
