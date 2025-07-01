use axum::{Extension, extract::Query, response::IntoResponse};

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty, UserDetailedInfo,
    request::{FriendRequest, RequestResponse},
};

/// 查看friend详细信息
#[utoipa::path(
    get,
    path = "/friend/info",
    params(
        FriendRequest
    ),
    responses(
        (status = 200, description = "获取个人信息", body = RequestResponse<UserDetailedInfo>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/friend"
)]
pub async fn handle_info_friend(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<FriendRequest>,
) -> impl IntoResponse {
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

    request_lock.get_userinfo(payload.id).await.into_response()
}
