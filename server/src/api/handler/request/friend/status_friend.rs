use axum::{extract::Query, response::IntoResponse, Extension};
use log::{debug};

use axum_extra::extract::TypedHeader;
use headers::Cookie;

use crate::{protocol::{request::RequestResponse, Empty, UserStatus}, server::AppState};


/// 使用好友id批量查询好友在线情况
#[utoipa::path(
    get,
    path = "/friend/status",
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
    Query(user_ids): Query<Vec<u32>>,
) -> impl IntoResponse {
    debug!("处理获取好友列表请求");
    
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

    request_lock.get_status_by_userids(&user_ids).await.into_response()
}