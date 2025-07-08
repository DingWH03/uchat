use crate::{server::AppState, utils::detect_image_type};
use axum::Extension;
use axum::extract::multipart::Multipart;
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use axum::body::Bytes;
use headers::Cookie;
use log::debug;
use uchat_protocol::{Empty, request::RequestResponse};

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
    debug!("处理更新用户头像请求");

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

    debug!("上传文件 content_type: {}", content_type);

    if let Some(bytes) = &file_bytes {
        // 检查 MIME 类型前缀（更宽松）
        let allowed_types = ["image/png", "image/jpeg", "image/jpg", "image/webp"];
        if !allowed_types.iter().any(|t| content_type.starts_with(t)) {
            return RequestResponse::<()>::err("仅支持上传 PNG/JPEG/WebP 格式图片").into_response();
        }

        // 魔数校验
        let valid = match detect_image_type(bytes) {
            Some(kind) => {
                debug!("识别的文件类型: {}", kind);
                allowed_types.contains(&kind)
            }
            None => false,
        };

        if !valid {
            return RequestResponse::<()>::err("文件内容格式不合法").into_response();
        }

        let response = request_lock
            .update_avatar(user_id, bytes, &file_name, &content_type)
            .await;

        response.into_response()
    } else {
        RequestResponse::<()>::err("未找到有效的文件字段").into_response()
    }
}
