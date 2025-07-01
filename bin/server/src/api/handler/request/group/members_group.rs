use axum::{Extension, extract::Query, response::IntoResponse};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty, UserSimpleInfo,
    request::{GroupRequest, RequestResponse},
};

/// 获取群组成员
#[utoipa::path(
    get,
    path = "/group/members",
    params(
        GroupRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = RequestResponse<UserSimpleInfo>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 404, description = "找不到群组", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/group"
)]
pub async fn handle_members_group(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GroupRequest>,
) -> impl IntoResponse {
    debug!("处理查询群聊成员请求: {:?}", payload);

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
        .get_group_members(payload.id)
        .await
        .into_response()
}
