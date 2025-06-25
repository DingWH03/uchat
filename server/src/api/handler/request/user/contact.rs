use crate::protocol::Empty;
use crate::protocol::UpdateTimestamps;
use crate::protocol::request::RequestResponse;
use crate::server::AppState;
use axum::Extension;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use headers::Cookie;

/// 查询用户的好友和群组更新时间戳(单位：秒)
#[utoipa::path(
    get,
    path = "/user/contact/timestamps",
    responses(
        (status = 200, description = "获取个人信息", body = RequestResponse<UpdateTimestamps>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_get_contact_timestamps(
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
        None => {
            return RequestResponse::<()>::unauthorized().into_response();
        }
    };

    request_lock
        .get_update_timestamps(user_id)
        .await
        .into_response()
}
