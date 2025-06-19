use axum::{extract::Query, response::IntoResponse, Extension, Json};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::{debug, warn};

use crate::{
    protocol::{manager::DeleteSessionRequest, ManagerResponse}, server::AppState // 你的 ManagerResponse 结构体
};

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
    Json(manager_lock.remove_session(&payload.session_id).await).into_response()
}
