use axum::{Extension, Json, response::IntoResponse};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty,
    request::{GroupRequest, RequestResponse},
};

/// 离开群组
#[utoipa::path(
    post,
    path = "/group/leave",
    request_body = GroupRequest,
    responses(
        (status = 200, description = "离开成功", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 400, description = "错误请求", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/group"
)]
pub async fn handle_leave_group(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<GroupRequest>,
) -> impl IntoResponse {
    debug!("处理退出群聊请求: {:?}", payload);

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
        .leave_group(user_id, payload.id)
        .await
        .into_response()
}
