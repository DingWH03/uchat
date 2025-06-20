use axum::{response::IntoResponse, Extension, Json};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::{debug, warn};

use crate::{
    protocol::{manager::OnlineUserTree, ManagerResponse}, server::AppState 
};

#[utoipa::path(
    get,
    path = "/manager/online/tree",
    responses(
        (status = 200, description = "返回在线用户树", body = ManagerResponse<OnlineUserTree>)
    ),
    tag = "manager"
)]
pub async fn handle_tree_online(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("manager请求：获取在线用户树");

    // 从 Cookie 中提取 session_id
    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        warn!("未找到 session_id Cookie，拒绝操作");
        return Json(ManagerResponse::<u32>::err("未找到 session_id Cookie")).into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证 session 是否存在，并且拥有权限（管理员）
    match manager_lock.check_session_role(&session_id).await {
        Some(role) => {
            if !role.is_admin() {
                warn!("用户权限不足: {:?}", role);
                return Json(ManagerResponse::<u32>::err("无管理员权限")).into_response();
            }
        }
        None => {
            warn!("无效或过期的 session_id: {}", session_id);
            return Json(ManagerResponse::<u32>::err("会话无效或已过期")).into_response();
        }
    };

    // 获取用户列表
    Json(manager_lock.get_online_user().await).into_response()
}
