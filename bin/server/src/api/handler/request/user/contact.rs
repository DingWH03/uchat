use crate::server::AppState;
use axum::Extension;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use headers::Cookie;
use log::debug;
use uchat_protocol::{ContactList, Empty, UpdateTimestamps, request::RequestResponse};

/// 查询用户的好友和群组更新时间戳(单位：秒)
#[utoipa::path(
    get,
    path = "/user/contact/timestamps",
    responses(
        (status = 200, description = "获取好友和群组列表最新时间戳", body = RequestResponse<UpdateTimestamps>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_get_contact_timestamps(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理查询用户的好友和群组更新时间戳请求");
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
        .get_update_timestamps(user_id)
        .await
        .into_response()
}

/// 查询用户的好友和群组列表，相当于同时调用list_friend和list_group
#[utoipa::path(
    get,
    path = "/user/contact/list",
    responses(
        (status = 200, description = "获取完整好友列表，不支持状态信息", body = RequestResponse<ContactList>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_get_contact_list(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理查询用户的好友和群组列表");
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

    request_lock.get_contact_list(user_id).await.into_response()
}
