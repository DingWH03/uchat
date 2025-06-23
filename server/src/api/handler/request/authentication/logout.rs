use axum::{Extension, response::IntoResponse};
use log::debug;

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{
    protocol::{Empty, request::RequestResponse},
    server::AppState,
};

/// 处理退出登录
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "退出登录成功", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器内部错误", body = RequestResponse<Empty>),
    ),
    tag = "request/auth"
)]
pub async fn handle_logout(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理退出登录请求");

    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();

    let request_lock = state.request.lock().await;
    let _user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => {
            return RequestResponse::<()>::unauthorized().into_response();
        }
    };

    request_lock.logout(&session_id).await.into_response()
}
