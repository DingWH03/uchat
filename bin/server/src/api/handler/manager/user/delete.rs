use crate::server::AppState;
use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;
use uchat_protocol::{Empty, ManagerResponse, manager::DeleteUserRequest};

/// 指定删除某用户
#[utoipa::path(
    delete,
    path = "/manager/user",
    params(
        DeleteUserRequest
    ),
    responses(
        (status = 200, description = "删除成功", body = ManagerResponse<Empty>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/user"
)]
pub async fn handle_delete_user(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteUserRequest>,
) -> impl IntoResponse {
    debug!("manager请求：删除用户 {} ", payload.user_id,);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .delete_user(payload.user_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}
