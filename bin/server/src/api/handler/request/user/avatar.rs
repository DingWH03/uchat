use axum::response::IntoResponse;
use axum::Extension;
use axum_extra::TypedHeader;
use axum::extract::multipart::Multipart;
use headers::Cookie;
use uchat_protocol::{request::RequestResponse, Empty};
use crate::server::AppState;

/// 上传头像（multipart 文件）
/// 返回上传后的头像 URL
#[utoipa::path(
    post,
    path = "/user/avatar",
    request_body(
        content_type = "multipart/form-data",
        description = "头像文件"
    ),
    responses(
        (status = 200, description = "头像上传成功", body = RequestResponse<String>),
        (status = 401, description = "未认证", body = RequestResponse<Empty>),
        (status = 400, description = "上传失败", body = RequestResponse<Empty>)
    ),
    tag = "request/user"
)]
pub async fn handle_upload_avatar(
    Extension(state): Extension<AppState>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    use axum::body::Bytes;

    // 提取 session_id
    let session_id = cookies.get("session_id").map(str::to_string);
    if session_id.is_none() {
        return RequestResponse::<()>::unauthorized().into_response();
    }
    let session_id = session_id.unwrap();

    // 获取 user_id
    let request_lock = state.request.lock().await;
    let user_id = match request_lock.check_session(&session_id).await {
        Some(uid) => uid,
        None => return RequestResponse::<()>::unauthorized().into_response(),
    };

    // 提取文件字段
    let mut file_bytes: Option<Bytes> = None;
    let mut file_name = "avatar.png".to_string();
    let mut content_type = "application/octet-stream".to_string();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("file") {
            file_name = field.file_name().unwrap_or("avatar.png").to_string();
            content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            file_bytes = Some(field.bytes().await.unwrap_or_default());
            break;
        }
    }

    let allowed_types = ["image/png", "image/jpeg", "image/jpg", "image/webp"];
    if !allowed_types.contains(&content_type.as_str()) {
        return RequestResponse::<()>::err("仅支持上传 PNG/JPEG/WebP 格式图片").into_response();
    }

    if let Some(bytes) = file_bytes {
        let response = request_lock
            .update_avatar(user_id, &bytes, &file_name, &content_type)
            .await;

        response.into_response()
    } else {
        RequestResponse::<()>::err("未找到有效的文件字段").into_response()
    }
}

