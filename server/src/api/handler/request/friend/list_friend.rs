use axum::{response::IntoResponse, Extension};
use log::{debug};

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{protocol::{request::{RequestResponse}, Empty, UserSimpleInfo, UserSimpleInfoWithStatus}, server::AppState};

/// 获取好友列表
#[utoipa::path(
    get,
    path = "/friend/list",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<UserSimpleInfo>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/friend"
)]
pub async fn handle_list_friend(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理获取好友列表请求");
    
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

    request_lock.get_friends(user_id).await.into_response()
}

/// 获取好友列表(带状态信息)
#[utoipa::path(
    get,
    path = "/friend/listv2",
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<UserSimpleInfoWithStatus>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/friend"
)]
pub async fn handle_list_friend_with_status(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理获取好友列表请求");
    
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

    request_lock.get_friends_with_status(user_id).await.into_response()
}