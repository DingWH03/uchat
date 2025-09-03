// 内有u32类型的 VERSION_CODE
include!(concat!(env!("OUT_DIR"), "/version.rs"));

// 其他构建期信息也顺便集中在此处
pub const PKG_NAME: &str    = env!("CARGO_PKG_NAME");
pub const PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIME: &str   = env!("BUILD_TIME");
