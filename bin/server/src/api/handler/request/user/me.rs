use crate::server::AppState;
use axum::Extension;
use axum::Json;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use headers::Cookie;
use uchat_protocol::{
    Empty, UserDetailedInfo,
    request::{PatchUserRequest, RequestResponse, UpdateUserRequest},
};

/// 获取个人信息
#[utoipa::path(
    get,
    path = "/user/me",
    responses(
        (status = 200, description = "获取个人信息", body = RequestResponse<UserDetailedInfo>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_get_me(
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

    request_lock.get_userinfo(user_id).await.into_response()
}

/// 完整更新个人资料
#[utoipa::path(
    put,
    path = "/user/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "完整更新个人资料", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_put_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<UpdateUserRequest>,
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
        .update_user_info_full(user_id, payload)
        .await
        .into_response()
}

/// 部分修改个人资料
#[utoipa::path(
    patch,
    path = "/user/me",
    request_body = PatchUserRequest,
    responses(
        (status = 200, description = "部分修改个人资料", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_patch_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<PatchUserRequest>,
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
        .update_user_info_partial(user_id, payload)
        .await
        .into_response()
}

/// 删除自己账号
#[utoipa::path(
    delete,
    path = "/user/me",
    responses(
        (status = 200, description = "删除自己的账号(依据session_id)", body = RequestResponse<Empty>),
        (status = 401, description = "认证失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_delete_me(
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
    request_lock.delete_user(user_id).await.into_response()
}
