use axum::{Extension, response::IntoResponse};
use log::debug;

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{
    protocol::{Empty, ManagerResponse, RoleType, request::RequestResponse},
    server::AppState,
};

/// 会话有效性检查接口
///
/// 检查客户端当前 Cookie 中的 `session_id` 是否有效，并返回用户的身份角色。
/// 通常用于客户端启动后自动判断是否已登录或会话是否过期。
///
/// ### 用法示例
/// - 请求方式：`GET /check_session`
/// - 请求头中需携带 Cookie：`session_id=...`
/// - 返回：
///   - `status: true` 表示登录状态有效
///   - `data`: 用户的角色（如 User、Admin、Invalid）
///
/// 若未携带 Cookie 或会话已失效，将返回 `status: false` 和 `data: null`。
#[utoipa::path(
    get,
    path = "/auth/check_session",
    responses(
        (status = 200, description = "会话有效", body = RequestResponse<RoleType>),
        (status = 401, description = "会话无效", body = RequestResponse<Empty>)
    ),
    tag = "request/auth"
)]
pub async fn handle_check_session(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("检查会话是否有效");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let request_lock = state.request.lock().await;
    // 通过会话id获取用户id
    request_lock
        .check_session_role(&session_id)
        .await
        .into_response()
}
