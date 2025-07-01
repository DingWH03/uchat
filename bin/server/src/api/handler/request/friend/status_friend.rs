use axum::{Extension, Json, response::IntoResponse};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty, UserStatus,
    request::{CheckStatusRequest, RequestResponse},
};

/// 使用好友id批量查询好友在线情况
#[utoipa::path(
    post,
    path = "/friend/status",
    request_body = CheckStatusRequest,
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<Vec<UserStatus>>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/friend"
)]
pub async fn handle_get_status_by_userid(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(user_ids): Json<CheckStatusRequest>,
) -> impl IntoResponse {
    debug!("处理获取好友在线状态请求: {:?}", user_ids);

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
        .get_status_by_userids(&user_ids.user_ids)
        .await
        .into_response()
}
