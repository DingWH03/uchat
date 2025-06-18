use axum::{response::IntoResponse, Extension, Json};
use log::{debug, warn};

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{protocol::request::{CreateGroupRequest, ServerResponse}, server::AppState};

pub async fn handle_creat_group(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    debug!("处理创建群聊请求: {:?}", payload);
    
    // 尝试从 Cookie 中获取 "session_id"
    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        warn!("未找到 session_id Cookie，拒绝创建群聊操作");
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie，拒绝创建群聊操作",
        )
            .into_response();
    };

    let request_lock = state.request.lock().await;
    // 通过会话id获取用户id
    let user_id = match request_lock.check_session(&session_id).await {
    Some(uid) => {
        debug!("登陆用户id: {}", uid);
        uid
    }
    None => {
        warn!("会话ID {} 不存在或已过期", session_id);
        return (axum::http::StatusCode::UNAUTHORIZED, "会话ID不存在或已过期").into_response();
    }
};

    let result = request_lock.create_group(user_id,&payload.group_name, payload.members).await;
    let response = match result {
        Ok(group_id) => ServerResponse::GenericResponse { status: "Ok".to_string(), message: format!("创建群聊成功，群聊ID为{}", group_id) }, 
        Err(e) => {
            warn!("用户{}创建群聊出现错误: {}", user_id, e);
            ServerResponse::GenericResponse { status: "Err".to_string(), message: "服务器错误".to_string() }
        }
    };
    debug!("创建群聊请求响应: {:?}", response);
    axum::Json(response).into_response()
}