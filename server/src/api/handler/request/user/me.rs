use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use axum_extra::TypedHeader;
use headers::Cookie;
use log::debug;
use log::warn;
use crate::protocol::request::ServerResponse;
use crate::protocol::request::UpdateUserRequest;

/// 获取个人信息
pub async fn handle_get_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理查看自己信息请求");

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        debug!("从 Cookie 中找到 session_id: {}", session_id_cookie);
        session_id_cookie.to_string()
    } else {
        warn!("未找到 session_id Cookie，拒绝查看自己信息操作");
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie",
        )
        .into_response();
    };

    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => {
            debug!("登录用户 id: {}", uid);
            uid
        }
        None => {
            warn!("会话 ID 不存在或已过期: {}", session_id);
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "会话 ID 不存在或已过期",
            )
                .into_response();
        }
    };

    let result = request_lock.get_userinfo(user_id).await;

    let response = match result {
        Ok(Some(info)) => ServerResponse::UserInfo { user_id, userinfo: info },
        Ok(None) => ServerResponse::GenericResponse {
            status: "NotFound".to_string(),
            message: "用户不存在".to_string(),
        },
        Err(e) => {
            warn!("用户 {} 查询自己信息时错误: {}", user_id, e);
            ServerResponse::GenericResponse {
                status: "Err".to_string(),
                message: "服务器错误".to_string(),
            }
        }
    };

    debug!("查看自己信息请求响应: {:?}", response);
    Json(response).into_response()
}

/// 完整更新个人资料
pub async fn handle_put_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    debug!("处理完整更新自己信息请求: {:?}", payload);

    let session_id = if let Some(session_id_cookie) = cookies.get("session_id") {
        session_id_cookie.to_string()
    } else {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie",
        )
        .into_response();
    };

    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "会话 ID 不存在或已过期",
            )
                .into_response();
        }
    };

    let result = request_lock.update_user_info_full(user_id, payload).await;

    let response = match result {
        Ok(_) => ServerResponse::GenericResponse {
            status: "OK".to_string(),
            message: "用户信息已更新".to_string(),
        },
        Err(e) => {
            warn!("用户 {} 更新信息失败: {}", user_id, e);
            ServerResponse::GenericResponse {
                status: "Err".to_string(),
                message: "服务器错误".to_string(),
            }
        }
    };

    Json(response).into_response()
}

/// 部分修改个人资料
use crate::protocol::request::PatchUserRequest;
use crate::server::AppState;

pub async fn handle_patch_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    Json(payload): Json<PatchUserRequest>,
) -> impl IntoResponse {
    debug!("处理部分更新自己信息请求: {:?}", payload);

    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie",
        )
            .into_response();
    }

    let session_id = session_id.unwrap();

    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "会话 ID 不存在或已过期",
            )
                .into_response();
        }
    };

    let result = request_lock.update_user_info_partial(user_id, payload).await;

    let response = match result {
        Ok(_) => ServerResponse::GenericResponse {
            status: "OK".to_string(),
            message: "部分信息已更新".to_string(),
        },
        Err(e) => {
            warn!("用户 {} 部分更新失败: {}", user_id, e);
            ServerResponse::GenericResponse {
                status: "Err".to_string(),
                message: "服务器错误".to_string(),
            }
        }
    };

    Json(response).into_response()
}

/// 删除自己账号
pub async fn handle_delete_me(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> impl IntoResponse {
    debug!("处理用户注销请求");

    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "未找到 session_id Cookie",
        )
            .into_response();
    }

    let session_id = session_id.unwrap();

    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "会话 ID 不存在或已过期",
            )
                .into_response();
        }
    };

    let result = request_lock.delete_user(user_id).await;

    let response = match result {
        Ok(_) => ServerResponse::GenericResponse {
            status: "OK".to_string(),
            message: "账号已注销".to_string(),
        },
        Err(e) => {
            warn!("用户 {} 注销失败: {}", user_id, e);
            ServerResponse::GenericResponse {
                status: "Err".to_string(),
                message: "服务器错误".to_string(),
            }
        }
    };

    Json(response).into_response()
}
