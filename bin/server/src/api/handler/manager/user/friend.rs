use crate::server::AppState;
use axum::{
    extract::{Extension, Query},
    response::IntoResponse,
};
use axum_extra::extract::TypedHeader;
use headers::Cookie;
use log::debug;
use uchat_protocol::{
    Empty, ManagerResponse, UserSimpleInfo,
    manager::{DeleteFriendshipRequest, GetFriendsRequest},
};

/// 指定删除某好友关系
#[utoipa::path(
    delete,
    path = "/manager/user/friend",
    params(
        DeleteFriendshipRequest
    ),
    responses(
        (status = 200, description = "删除成功", body = ManagerResponse<Empty>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/user"
)]
pub async fn handle_delete_friendship(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<DeleteFriendshipRequest>,
) -> impl IntoResponse {
    debug!(
        "manager请求：删除用户好友关系 {} 和 {} ",
        payload.user_id, payload.friend_id
    );

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .delete_friendship(payload.user_id, payload.friend_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}

/// 查看某用户所有好友
#[utoipa::path(
    get,
    path = "/manager/user/friend",
    params(
        GetFriendsRequest
    ),
    responses(
        (status = 200, description = "获取成功", body = ManagerResponse<Vec<UserSimpleInfo>>),
        (status = 401, description = "认证失败", body = ManagerResponse<Empty>),
        (status = 403, description = "权限不足", body = ManagerResponse<Empty>),
        (status = 500, description = "服务器错误", body = ManagerResponse<Empty>)
    ),
    tag = "manager/user"
)]
pub async fn handle_get_friends(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Query(payload): Query<GetFriendsRequest>,
) -> impl IntoResponse {
    debug!("manager请求：获取用户 {} 好友列表 ", payload.user_id);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return ManagerResponse::<()>::unauthorized().into_response();
    };

    let manager_lock = state.manager.lock().await;

    // 验证权限
    match manager_lock.check_session_role(&session_id).await {
        Some(role) if role.is_admin() => manager_lock
            .get_friends(payload.user_id)
            .await
            .into_response(),
        Some(_) => ManagerResponse::<()>::forbidden().into_response(),
        None => ManagerResponse::<()>::unauthorized().into_response(),
    }
}
