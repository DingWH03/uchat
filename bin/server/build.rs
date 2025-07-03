use std::{fs, path::Path};
use toml::Value;

fn main() {
    println!("cargo:rustc-env=SQLX_OFFLINE=true");  // sqlx本地化编译
    // 读取 Cargo.toml
    let manifest_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(manifest_path).expect("无法读取 Cargo.toml");

    // 解析 toml
    let parsed: Value = content.parse().expect("解析 Cargo.toml 失败");

    let package = &parsed["package"];
    let name = package["name"].as_str().unwrap_or("Unknown");
    let version = package["version"].as_str().unwrap_or("Unknown");
    let authors = package["authors"]
        .as_array()
        .and_then(|arr| Some(arr.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", ")))
        .unwrap_or("Unknown".to_string());

    let build_time = chrono::Utc::now().to_rfc3339();

    // 设置环境变量，供代码中使用
    println!("cargo:rustc-env=PKG_NAME={}", name);
    println!("cargo:rustc-env=PKG_VERSION={}", version);
    println!("cargo:rustc-env=PKG_AUTHORS={}", authors);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

}