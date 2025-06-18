use axum::{
    extract::{Extension, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::{debug, warn};

use crate::api::handler::ws::handle_socket;
use crate::server::AppState;
/// 处理WebSocket升级请求
/// 签名已调整为标准的WebSocket升级处理器，并从 Cookie 中提取 session_id
pub async fn handle_connect(
    ws: WebSocketUpgrade,                      // Axum 提供的 WebSocket 升级器
    Extension(state): Extension<AppState>,     // 获取共享的应用程序状态
    TypedHeader(cookies): TypedHeader<Cookie>, // 提取 HTTP 请求中的所有 Cookie
) -> Response {
    debug!("收到WebSocket升级请求");

    // 尝试从 Cookie 中获取 "session_id"
    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        warn!("未找到 session_id Cookie，拒绝 WebSocket 连接");
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie，拒绝 WebSocket 连接",
        )
            .into_response();
    };

    let request_lock = state.request.lock().await; // 获取 Request 的 Mutex 锁 (tokio::sync::Mutex)
    if let Some(user_id) = request_lock.check_session(&session_id).await {
        debug!("登陆用户id: {}", user_id);
    } else {
        warn!("会话ID {} 不存在或已过期", session_id);
        return (axum::http::StatusCode::UNAUTHORIZED, "会话ID不存在或已过期").into_response();
    }
    drop(request_lock); // 及时释放 Mutex 锁
    // 使用 ws.on_upgrade 方法将 HTTP 连接升级为 WebSocket 连接
    // 然后将控制权交给 handle_socket 函数来处理 WebSocket 帧，并传递 session_id
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}
