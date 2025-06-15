// src/server.rs

mod route;

use axum::{Extension, Router};
use tokio::sync::{Mutex, RwLock};
use std::net::SocketAddr;
use crate::api::{request::Request, session_manager::SessionManager};
use crate::db::Database;
use crate::api::manager::Manager;
use route::router;
use log::{info, error};
use std::sync::Arc;

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
        let dbmysql = match Database::new().await {
            Ok(db) => {
                info!("数据库连接成功");
                Arc::new(db)
            },
            Err(e) => {
                error!("数据库连接失败: {}", e);
                std::process::exit(1);
            }
        };
        let sessions = Arc::new(RwLock::new(SessionManager::new()));
        let request = Arc::new(Mutex::new(Request::new(dbmysql.clone(), sessions.clone())));
        let manager = Arc::new(Mutex::new(Manager::new(dbmysql, sessions)));
        let state = AppState {
            request,
            manager,
        };
        // 构建路由
        let app = router()
            .layer(Extension(state));
        
        Server { addr, app }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("服务器已启动，监听地址 http://{}", self.addr);
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, self.app.into_make_service()).await?;
        Ok(())
    }
}
