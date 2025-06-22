use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;

use crate::{
    protocol::{manager::ChangeRoleRequest, request::RequestResponse, Empty, ManagerResponse},
    server::AppState,
};

/// 更改用户身份
#[utoipa::path(
    post,
    path = "/manager/user/role",
    request_body = ChangeRoleRequest,
    responses(
        (status = 200, description = "更改身份成功", body = RequestResponse<Empty>),
        (status = 400, description = "目标身份不存在", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>),
        (status = 500, description = "服务器内部错误", body = RequestResponse<Empty>),
    ),
    tag = "manager/user"
)]
pub async fn handle_change_role(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<ChangeRoleRequest>,
) -> impl IntoResponse {
    debug!(
        "manager请求：修改用户 {} 的角色为 {}",
        payload.user_id,
        payload.new_role.to_string()
    );

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => 
            manager_lock
                .set_user_role(payload.user_id, payload.new_role)
                .await
        .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}
