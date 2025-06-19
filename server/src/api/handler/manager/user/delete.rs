use axum::{
    extract::{Extension, Json, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;

use crate::{
    protocol::{manager::{DeleteUserRequest}, ManagerResponse},
    server::AppState,
};

pub async fn handle_delete_user(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteUserRequest>,
) -> impl IntoResponse {
    debug!("manager请求：删除用户 {} ", payload.user_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return Json(ManagerResponse::<()>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => {
            Json(manager_lock.delete_user(payload.user_id).await).into_response()
        }
        Some(_) => Json(ManagerResponse::<()>::err("无管理员权限")).into_response(),
        None => Json(ManagerResponse::<()>::err("会话无效或已过期")).into_response(),
    }
}
