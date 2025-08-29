use axum::{Extension, Json, response::IntoResponse};
use log::debug;

use crate::server::AppState;
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use uchat_model::{
    Empty,
    request::{CreateGroupRequest, RequestResponse},
};

/// 创建群组
#[utoipa::path(
    post,
    path = "/group/new",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "创建成功", body = RequestResponse<u32>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器错误", body = RequestResponse<Empty>)
    ),
    tag = "request/group"
)]
pub async fn handle_creat_group(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    debug!("处理创建群聊请求: {:?}", payload);

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
        .create_group(user_id, &payload.group_name, payload.members)
        .await
        .into_response()
}
