use axum::{Extension, Json, response::IntoResponse};
use log::{debug, warn};

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{protocol::request::ServerResponse, server::AppState};

pub async fn handle_check_session(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("检查会话是否有效");

    // 尝试从 Cookie 中获取 "session_id"
    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        debug!("未找到 session_id Cookie，无效操作");
        return Json(ServerResponse::CheckSessionResponse { status: false, role: crate::protocol::RoleType::Invalid })
        .into_response();
    };

    let request_lock = state.request.lock().await;
    // 通过会话id获取用户id
    match request_lock.check_session_role(&session_id).await {
        Some(role) => {
            debug!("登陆用户role: {:?}", role);
            // request_lock.
            Json(ServerResponse::CheckSessionResponse { status: true, role: role })
            .into_response()
        }
        None => {
            warn!("会话ID {} 不存在或已过期", session_id);
            Json(ServerResponse::CheckSessionResponse { status: false, role: crate::protocol::RoleType::Invalid })
            .into_response()
            .into_response()
        }
    }
}
