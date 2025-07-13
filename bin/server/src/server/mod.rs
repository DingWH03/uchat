// src/server.rs

mod route;

use crate::api::manager::Manager;
use crate::api::request::Request;
use crate::config::get_config;
use crate::db::factory::{DbType, create_database};
#[cfg(feature = "redis-support")]
use crate::redis::RedisClient;
use crate::session::SessionConfig;
use crate::session::create_session_manager;

use crate::cache::{create_cache_manager, CacheConfig};
use crate::storage::{StorageBackend, StorageConfig, init_storage};
use axum::{Extension, Router};
use log::{error, info};
use route::router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub request: Arc<Mutex<Request>>,
    pub manager: Arc<Mutex<Manager>>,
}

pub struct Server {
    addr: SocketAddr,
    app: Router,
}

impl Server {
    pub async fn new(addr: SocketAddr) -> Self {
        // 从全局配置文件中获取数据库连接字符串，完成初始化数据库操作
        let config= get_config();
        let database_url = config.database.url.clone();
        let db_type = match config.database.db_type
            .parse::<DbType>()
        {
            Ok(db_type) => db_type,
            Err(e) => {
                error!("未识别的数据库类型: {}", e);
                panic!("未识别的数据库类型: {}", e);
            }
        };
        let db = match create_database(db_type, &database_url).await {
            Ok(db) => db,
            Err(e) => {
                error!("数据库连接失败: {}", e);
                panic!("数据库连接失败: {}", e);
            }
        };

        // 初始化redis连接池
        #[cfg(feature = "redis-support")]
        let redis_client_session = {
            let redis_url = config.redis.sessions.url.clone();
            let redis_client = RedisClient::new(&redis_url).await;
            match redis_client {
                Err(e) => {
                    error!("[Session] Redis连接失败: {}", e);
                    panic!("[Session] Redis连接失败: {}", e);
                }
                Ok(client) => {
                    info!("[Session] Redis连接成功");
                    client
                }
            }
        };
        #[cfg(feature = "redis-support")]
        let redis_client_cache = {
            let redis_url = config.redis.cache.url.clone();
            let redis_client = RedisClient::new(&redis_url).await;
            match redis_client {
                Err(e) => {
                    error!("[Cache] Redis连接失败: {}", e);
                    panic!("[Cache] Redis连接失败: {}", e);
                }
                Ok(client) => {
                    info!("[Cache] Redis连接成功");
                    client
                }
            }
        };
        // 选择会话存储配置
        #[cfg(not(feature = "redis-support"))]
        let session_config = { SessionConfig {} };
        #[cfg(feature = "redis-support")]
        let session_config = {
            SessionConfig {
                redis: Arc::new(redis_client_session),
                session_expire_secs: 7200, // 默认会话过期时间为2小时
            }
        };
        // 选择缓存存储配置
        #[cfg(not(feature = "redis-support"))]
        let cache_config = { CacheConfig {}};
        #[cfg(feature = "redis-support")]
        let cache_config = {
            CacheConfig {
                redis: Arc::new(redis_client_cache),
                expire_secs: Some(7200), // 默认会话过期时间为2小时
            }
        };
        // 从环境变量读取 MinIO（或其他存储）配置
        let storage_config = StorageConfig {
            endpoint: config.minio.endpoint.clone(),
            access_key: config.minio.access_key.clone(),
            secret_key: config.minio.secret_key.clone(),
            bucket: config.minio.bucket.clone(),
            base_url: config.minio.base_url.clone(),
            local_dir: config.local.storage_dir.clone(), // 这个你可以选用默认
        };
        // 选用 MinIO 作为存储后端（以后可用环境变量或配置动态切换）
        let storage_backend = StorageBackend::Minio;
        // 初始化存储
        let storage = init_storage(storage_backend, &storage_config).await;
        let sessions = create_session_manager(session_config).await;
        let cache = create_cache_manager(cache_config).await;
        let request = Arc::new(Mutex::new(Request::new(
            db.clone(),
            sessions.clone(),
            storage.clone(),
            cache
        )));
        let manager = Arc::new(Mutex::new(Manager::new(db, sessions, storage)));
        let state = AppState { request, manager };
        // 构建路由
        let mut app = router().layer(Extension(state));

        // 条件编译：仅在启用 swagger 特性时添加
        #[cfg(feature = "swagger")]
        {
            use crate::api::doc;
            use utoipa_swagger_ui::SwaggerUi;
            app =
                app.merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", doc::openapi()));
        }

        Server { addr, app }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let banner = r#"
            _    _          _               _
            | |  | |        | |             | |
            | |  | |   ___  | |__     __ _  | |_
            | |  | |  / __| | '_ \   / _` | | __|
            | |__| | | (__  | | | | | (_| | | |_
             \____/   \___| |_| |_|  \__,_|  \__|

        "#;

        println!("{}", banner);
        println!("项目名称     : {}", env!("PKG_NAME"));
        println!("版本号       : {}", env!("PKG_VERSION"));
        println!("作者         : {}", env!("PKG_AUTHORS"));
        println!("构建时间     : {}", env!("BUILD_TIME"));
        println!("监听地址     : http://{}", self.addr);
        let now = chrono::Local::now();
        println!("启动时间     : {}\n", now.format("%Y-%m-%d %H:%M:%S"));

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, self.app.into_make_service_with_connect_info::<SocketAddr>()).await?;
        Ok(())
    }
}
