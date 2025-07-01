use axum::{Extension, Json, response::IntoResponse};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty,
    request::{FriendRequest, RequestResponse},
};

/// 添加好友
#[utoipa::path(
    post,
    path = "/friend/add",
    request_body = FriendRequest,
    responses(
        (status = 200, description = "添加成功", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 400, description = "目标用户不存在", body = RequestResponse<Empty>)
    ),
    tag = "request/friend"
)]
pub async fn handle_add_friend(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<FriendRequest>,
) -> impl IntoResponse {
    debug!("处理添加好友请求: {:?}", payload);

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
        .add_friend(user_id, payload.id)
        .await
        .into_response()
}
