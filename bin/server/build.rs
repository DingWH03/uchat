use std::{env, fs, path::{Path, PathBuf}};
use toml::Value;

// 只取主/次/补丁三段；忽略 -pre/+build
fn parse_semver_three_parts(s: &str) -> (u32, u32, u32) {
    let core = s.split(['-', '+']).next().unwrap();
    let mut it = core.split('.');
    let maj = it.next().unwrap_or("0").parse().unwrap();
    let min = it.next().unwrap_or("0").parse().unwrap();
    let pat = it.next().unwrap_or("0").parse().unwrap();
    (maj, min, pat)
}

/// 以 (maj<<24)|(min<<16)|(pat<<8) 编码，低 8 位预留
fn make_version_code(version: &str) -> u32 {
    let (maj, min, pat) = parse_semver_three_parts(version);
    for (name, v) in [("major", maj), ("minor", min), ("patch", pat)] {
        assert!(v <= 255, "version {}={} exceeds 255; adjust packing scheme", name, v);
    }
    (maj << 24) | (min << 16) | (pat << 8)
}

fn main() {
    println!("cargo:rustc-env=SQLX_OFFLINE=true"); // sqlx本地化编译
    // 读取 Cargo.toml
    let manifest_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(manifest_path).expect("无法读取 Cargo.toml");

    // 解析 toml
    let parsed: Value = content.parse().expect("解析 Cargo.toml 失败");

    let package = &parsed["package"];
    let name = package["name"].as_str().unwrap_or("Unknown");
    let version = package["version"].as_str().expect("version not set");
    let version_code: u32 = make_version_code(version);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("version.rs");
    // 生成一个带常量的 Rust 源文件
    fs::write(
        dest,
        format!(
            "/// Auto-generated in build.rs\npub const VERSION_CODE: u32 = {};\n",
            version_code
        ),
    )
    .unwrap();
    let authors = package["authors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|v| v.as_str().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or("Unknown".to_string());

    let build_time = chrono::Utc::now().to_rfc3339();

    // 设置环境变量，供代码中使用
    println!("cargo:rustc-env=PKG_NAME={}", name);
    println!("cargo:rustc-env=PKG_VERSION={}", version);
    println!("cargo:rustc-env=PKG_VERSION_CODE={}", version_code);
    println!("cargo:rustc-env=PKG_AUTHORS={}", authors);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
}
