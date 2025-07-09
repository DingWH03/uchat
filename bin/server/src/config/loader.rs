use super::model::AppConfig;
use once_cell::sync::OnceCell;
use config::{Config, File};
use anyhow::Result;
use std::env;

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

pub fn init_config() -> Result<AppConfig> {
    // 从环境变量获取配置文件路径，默认 config.toml
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

    // 构建配置（不加载 Environment）
    let builder = Config::builder()
        .add_source(File::with_name(&config_path));

    let config = builder.build()?.try_deserialize::<AppConfig>()?;
    CONFIG.set(config.clone()).map_err(|_| anyhow::anyhow!("Config already initialized"))?;

    Ok(config)
}


pub fn get_config() -> &'static AppConfig {
    CONFIG.get().expect("Configuration not initialized")
}
