use axum::{
    extract::{Extension, ws::WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::{debug};
use uchat_protocol::request::RequestResponse;
use crate::{api::handler::ws::handle_socket};
use crate::server::AppState;
/// 处理WebSocket升级请求
/// 签名已调整为标准的WebSocket升级处理器，并从 Cookie 中提取 session_id
#[utoipa::path(
    get,
    path = "/auth/ws",
    responses(
        (status = 101, description = "WebSocket 协议升级成功"),
        (status = 401, description = "认证失败，缺少或非法 session_id Cookie")
    ),
    tag = "request/auth"
)]
pub async fn handle_connect(
    ws: WebSocketUpgrade,                      // Axum 提供的 WebSocket 升级器
    Extension(state): Extension<AppState>,     // 获取共享的应用程序状态
    TypedHeader(cookies): TypedHeader<Cookie>, // 提取 HTTP 请求中的所有 Cookie
) -> Response {
    debug!("收到WebSocket升级请求");

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
    drop(request_lock); // 及时释放 Mutex 锁
    // 使用 ws.on_upgrade 方法将 HTTP 连接升级为 WebSocket 连接
    // 然后将控制权交给 handle_socket 函数来处理 WebSocket 帧，并传递 session_id
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}
