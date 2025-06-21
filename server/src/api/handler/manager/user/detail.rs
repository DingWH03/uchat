use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;

use crate::{
    protocol::{ManagerResponse, manager::CheckUserDetailRequest},
    server::AppState,
};

pub async fn handle_get_userinfo(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<CheckUserDetailRequest>,
) -> impl IntoResponse {
    debug!("manager请求：查看用户 {} 详细信息", payload.user_id);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            manager_lock.get_user_detail(payload.user_id).await.into_response()
        }
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}
