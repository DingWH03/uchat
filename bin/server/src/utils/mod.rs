pub mod build_info;
use bytes::Bytes;


/// 检查魔数，返回识别出的 MIME 类型
pub fn detect_image_type(bytes: &Bytes) -> Option<&'static str> {
    let slice = bytes.as_ref();

    if slice.len() >= 8 && slice.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }

    if slice.len() >= 3 && slice.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }

    if slice.len() >= 12 && &slice[..4] == b"RIFF" && &slice[8..12] == b"WEBP" {
        return Some("image/webp");
    }

    None
}