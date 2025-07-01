use crate::server::AppState;
use axum::{Extension, extract::Query, response::IntoResponse};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::{debug, warn};
use uchat_protocol::{Empty, ManagerResponse, manager::DeleteSessionRequest};

/// 删除某session_id
#[utoipa::path(
    delete,
    path = "/manager/online/session",
    params(
        DeleteSessionRequest
    ),
    responses(
        (status = 200, description = "删除成功", body = ManagerResponse<Empty>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/online"
)]
pub async fn handle_delete_session(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteSessionRequest>,
) -> impl IntoResponse {
    debug!("manager请求：删除在线session{}", payload.session_id);

    // 从 Cookie 中提取 session_id
    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        warn!("未找到 session_id Cookie，拒绝操作");
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证 session 是否存在，并且拥有权限（管理员）
    match manager_lock.check_session_role(&session_id).await {
        Some(role) => {
            if !role.is_admin() {
                warn!("用户权限不足: {:?}", role);
                return ManagerResponse::<()>::forbidden().into_response();
            }
        }
        None => {
            warn!("无效或过期的 session_id: {}", session_id);
            return ManagerResponse::<()>::unauthorized().into_response();
        }
    };

    // 删除session
    manager_lock
        .remove_session(&payload.session_id)
        .await
        .into_response()
}
