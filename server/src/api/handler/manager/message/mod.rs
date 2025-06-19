use axum::{
    extract::{Extension, Json, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;

use crate::{
    protocol::{manager::{DeleteMessageRequest, GetMessageRequest, GetRecentMessageRequest, GetUserRecentMessageRequest}, ManagerResponse},
    server::AppState,
};

pub async fn handle_get_recent_messages(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetRecentMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：查看服务器近期消息");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return Json(ManagerResponse::<()>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            Json(manager_lock.get_recent_messages(payload.count, payload.offset).await).into_response()
        }
        Some(_) => Json(ManagerResponse::<()>::err("无管理员权限")).into_response(),
        None => Json(ManagerResponse::<()>::err("会话无效或已过期")).into_response(),
    }
}

pub async fn handle_get_user_recent_messages(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetUserRecentMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：查看服务器近期消息");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return Json(ManagerResponse::<()>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            Json(manager_lock.get_user_recent_messages(payload.count, payload.offset, payload.user_id).await).into_response()
        }
        Some(_) => Json(ManagerResponse::<()>::err("无管理员权限")).into_response(),
        None => Json(ManagerResponse::<()>::err("会话无效或已过期")).into_response(),
    }
}

pub async fn handle_delete_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：删除消息 {} ", payload.message_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return Json(ManagerResponse::<()>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            Json(manager_lock.delete_message(payload.message_id).await).into_response()
        }
        Some(_) => Json(ManagerResponse::<()>::err("无管理员权限")).into_response(),
        None => Json(ManagerResponse::<()>::err("会话无效或已过期")).into_response(),
    }
}

pub async fn handle_get_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：获取消息 {} ", payload.message_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return Json(ManagerResponse::<()>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            Json(manager_lock.get_message(payload.message_id).await).into_response()
        }
        Some(_) => Json(ManagerResponse::<()>::err("无管理员权限")).into_response(),
        None => Json(ManagerResponse::<()>::err("会话无效或已过期")).into_response(),
    }
}