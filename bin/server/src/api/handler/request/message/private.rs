use std::collections::HashMap;

use axum::{
    Extension,
    extract::{Path, Query},
    response::IntoResponse,
};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_model::{
    Empty, IdMessagePair, SessionMessage,
    request::{AfterTimestampQuery, MessageRequest, RequestResponse},
};

/// 获取私聊聊天记录
#[utoipa::path(
    get,
    path = "/message/user",
    params(
        MessageRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<SessionMessage>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 404, description = "找不到群组", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_session_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<MessageRequest>,
) -> impl IntoResponse {
    debug!("处理获取私聊聊天记录请求: {:?}", payload);

    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();

    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => {
            return RequestResponse::<()>::unauthorized().into_response();
        }
    };

    request_lock
        .get_messages(user_id, payload.id, payload.offset)
        .await
        .into_response()
}

/// 获取与某个用户的最后一条私聊消息时间戳
#[utoipa::path(
    get,
    path = "/message/user/{friend_id}/latest",
    params(
        ("friend_id" = u32, Path, description = "好友ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Option<i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamp_with_user(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Path(friend_id): Path<u32>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let my_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_latest_timestamp_with_user(my_id, friend_id)
        .await
        .into_response()
}

/// 获取当前用户所有私聊的最后一条消息时间戳（按好友ID映射）
#[utoipa::path(
    get,
    path = "/message/user/latest",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<HashMap<u32, i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamps_of_all_private_chats(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_latest_timestamps_of_all_private_chats(user_id)
        .await
        .into_response()
}

/// 获取当前用户所有私聊中全局最新的一条消息时间戳
#[utoipa::path(
    get,
    path = "/message/user/max",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Option<i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamp_of_all_private_messages(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_latest_timestamp_of_all_private_messages(user_id)
        .await
        .into_response()
}

/// 获取与某个用户某时间之后的聊天记录（时间递增）
#[utoipa::path(
    get,
    path = "/message/user/{friend_id}/after",
    params(
        ("friend_id" = u32, Path, description = "好友ID"),
        AfterTimestampQuery
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<SessionMessage>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_private_messages_after_timestamp(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Path(friend_id): Path<u32>,
    Query(AfterTimestampQuery { timestamp }): Query<AfterTimestampQuery>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let my_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_private_messages_after_timestamp(my_id, friend_id, timestamp)
        .await
        .into_response()
}

/// 获取所有私聊中某时间之后的所有聊天记录（带对方 ID）
#[utoipa::path(
    get,
    path = "/message/user/after",
    params(
        AfterTimestampQuery
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<IdMessagePair>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_all_private_messages_after_timestamp(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(AfterTimestampQuery { timestamp }): Query<AfterTimestampQuery>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_all_private_messages_after_timestamp(user_id, timestamp)
        .await
        .into_response()
}
