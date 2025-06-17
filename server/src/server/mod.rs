// src/server.rs

mod route;

use crate::api::manager::Manager;
use crate::api::{request::Request, session_manager::SessionManager};
use crate::db::factory::{DbType, create_database};
use axum::{Extension, Router};
use log::{error, info};
use route::router;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

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
        // 从环境变量中获取数据库连接字符串
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

        let sessions = Arc::new(RwLock::new(SessionManager::new()));
        let request = Arc::new(Mutex::new(Request::new(db.clone(), sessions.clone())));
        let manager = Arc::new(Mutex::new(Manager::new(db, sessions)));
        let state = AppState { request, manager };
        // 构建路由
        let app = router().layer(Extension(state));

        Server { addr, app }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("服务器已启动，监听地址 http://{}", self.addr);
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, self.app.into_make_service()).await?;
        Ok(())
    }
}
