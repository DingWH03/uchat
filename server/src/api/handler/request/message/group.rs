use std::collections::HashMap;

use axum::{extract::{Path, Query}, response::IntoResponse, Extension};
use log::debug;

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{
    protocol::{
        Empty, IdMessagePair, SessionMessage,
        request::{AfterTimestampQuery, MessageRequest, RequestResponse},
    },
    server::AppState,
};

/// 获取群聊聊天记录
#[utoipa::path(
    get,
    path = "/message/group",
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
pub async fn handle_get_group_message(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<MessageRequest>,
) -> impl IntoResponse {
    debug!("处理获取群聊聊天记录请求: {:?}", payload);

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

    request_lock
        .get_group_messages(payload.id, payload.offset)
        .await
        .into_response()
}

/// 获取某个群聊的最后一条消息时间戳
#[utoipa::path(
    get,
    path = "/message/group/{group_id}/latest",
    params(
        ("group_id" = u32, Path, description = "群组ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Option<i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 404, description = "群组不存在", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamp_of_group(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Path(group_id): Path<u32>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let _user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_latest_timestamp_of_group(group_id)
        .await
        .into_response()
}

/// 获取当前用户所在所有群的最后一条消息时间戳（Map）
#[utoipa::path(
    get,
    path = "/message/group/latest",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<HashMap<u32, i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamps_of_all_groups(
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
        .get_latest_timestamps_of_all_groups(user_id)
        .await
        .into_response()
}

/// 获取当前用户所有群聊中最新的一条消息时间戳（全局最大）
#[utoipa::path(
    get,
    path = "/message/group/max",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Option<i64>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_latest_timestamp_of_all_group_messages(
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
        .get_latest_timestamp_of_all_group_messages(user_id)
        .await
        .into_response()
}

/// 获取某个群某时间之后的消息
#[utoipa::path(
    get,
    path = "/message/group/{group_id}/after",
    params(
        ("group_id" = u32, Path, description = "群组ID"),
        AfterTimestampQuery
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<SessionMessage>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 404, description = "群组不存在", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/message"
)]
pub async fn handle_get_group_messages_after_timestamp(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Path(group_id): Path<u32>,
    Query(AfterTimestampQuery { timestamp }): Query<AfterTimestampQuery>,
) -> impl IntoResponse {
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }

    let session_id = session_id.unwrap();
    let request_lock = state.request.lock().await;
    let _user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    request_lock
        .get_group_messages_after_timestamp(group_id, timestamp)
        .await
        .into_response()
}

/// 获取当前用户所有群中某时间之后的消息（带群 ID）
#[utoipa::path(
    get,
    path = "/message/group/after",
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
pub async fn handle_get_all_group_messages_after_timestamp(
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
        .get_all_group_messages_after_timestamp(user_id, timestamp)
        .await
        .into_response()
}
