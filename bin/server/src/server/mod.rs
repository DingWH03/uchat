// src/server.rs

mod route;

use crate::api::manager::Manager;
use crate::api::request::Request;
use crate::db::factory::{DbType, create_database};
#[cfg(feature = "redis-support")]
use crate::redis::RedisClient;
use crate::session::SessionConfig;
use crate::session::create_session_manager;
use crate::storage::{StorageBackend, StorageConfig, init_storage};
use axum::{Extension, Router};
use log::{error, info};
use route::router;
use std::env;
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
        // 从环境变量中获取数据库连接字符串，完成初始化数据库操作
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL 环境变量未设置");
        let db_type = match env::var("DB_TYPE")
            .unwrap_or_else(|_| "mysql".to_string())
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
        let redis_client = {
            let redis_url = env::var("REDIS_URL").expect("REDIS_URL 环境变量未设置");
            let redis_client = RedisClient::new(&redis_url).await;
            match redis_client {
                Err(e) => {
                    error!("Redis连接失败: {}", e);
                    panic!("Redis连接失败: {}", e);
                }
                Ok(client) => {
                    info!("Redis连接成功");
                    client
                }
            }
        };
        /// 选择会话存储配置
        #[cfg(not(feature = "redis-support"))]
        let config = { SessionConfig {} };
        #[cfg(feature = "redis-support")]
        let config = {
            SessionConfig {
                redis: Arc::new(redis_client),
                session_expire_secs: 7200, // 默认会话过期时间为2小时
            }
        };
        // 从环境变量读取 MinIO（或其他存储）配置
        let storage_config = StorageConfig {
            endpoint: env::var("MINIO_ENDPOINT").expect("环境变量 MINIO_ENDPOINT 未设置"),
            access_key: env::var("MINIO_ACCESS_KEY").expect("环境变量 MINIO_ACCESS_KEY 未设置"),
            secret_key: env::var("MINIO_SECRET_KEY").expect("环境变量 MINIO_SECRET_KEY 未设置"),
            bucket: env::var("MINIO_BUCKET").expect("环境变量 MINIO_BUCKET 未设置"),
            base_url: env::var("MINIO_BASE_URL").expect("环境变量 MINIO_BASE_URL 未设置"),
            local_dir: env::var("LOCAL_STORAGE_DIR").unwrap_or_else(|_| "./data".to_string()), // 这个你可以选用默认
        };
        // 选用 MinIO 作为存储后端（以后可用环境变量或配置动态切换）
        let storage_backend = StorageBackend::Minio;
        // 初始化存储
        let storage = init_storage(storage_backend, &storage_config).await;
        let sessions = create_session_manager(config).await;
        let request = Arc::new(Mutex::new(Request::new(
            db.clone(),
            sessions.clone(),
            storage.clone(),
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
        axum::serve(listener, self.app.into_make_service()).await?;
        Ok(())
    }
}
