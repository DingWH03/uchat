use crate::server::AppState;
use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;
use uchat_protocol::{
    Empty, FullPrivateMessage, ManagerResponse, PreviewPrivateMessage,
    manager::{
        DeleteMessageRequest, GetMessageRequest, GetRecentMessageRequest,
        GetUserRecentMessageRequest,
    },
};

/// 查看服务器近期消息
#[utoipa::path(
    get,
    path = "/manager/message/privite/recent",
    params(
        GetRecentMessageRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = ManagerResponse<Vec<PreviewPrivateMessage>>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/message"
)]
pub async fn handle_get_recent_messages(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetRecentMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：查看服务器近期消息");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .get_recent_messages(payload.count, payload.offset)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}

/// 查看某用户近期消息
#[utoipa::path(
    get,
    path = "/manager/message/privite/user",
    params(
        GetUserRecentMessageRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = ManagerResponse<Vec<PreviewPrivateMessage>>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/message"
)]
pub async fn handle_get_user_recent_messages(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetUserRecentMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：查看服务器近期消息");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .get_user_recent_messages(payload.count, payload.offset, payload.user_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}

/// 依据message_id删除某消息
#[utoipa::path(
    delete,
    path = "/manager/message/privite",
    params(
        DeleteMessageRequest
    ),
    responses(
        (status = 200, description = "删除成功", body = ManagerResponse<u64>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/message"
)]
pub async fn handle_delete_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：删除消息 {} ", payload.message_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .delete_message(payload.message_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}

/// 依据message_id获取某消息
#[utoipa::path(
    get,
    path = "/manager/message/privite",
    params(
        GetMessageRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = ManagerResponse<FullPrivateMessage>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/message"
)]
pub async fn handle_get_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetMessageRequest>,
) -> impl IntoResponse {
    debug!("manager请求：获取消息 {} ", payload.message_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .get_message(payload.message_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}
