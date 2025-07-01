pub mod session;
pub mod user;
pub mod message;
use std::collections::HashMap;
use std::sync::Arc;
use crate::session::{SessionConfig, SessionInfo, SessionManagerTrait};
use crate::db::DB;
use crate::storage::{ObjectStorage};
use log::{info};

pub struct Manager {
    db: Arc<dyn DB>,
    sessions: Arc<dyn SessionManagerTrait<Config = SessionConfig>>,
    storage: Arc<dyn ObjectStorage + Send + Sync>,
}

impl Manager {
    pub fn new(db: Arc<dyn DB>, sessions: Arc<dyn SessionManagerTrait<Config = SessionConfig>>, storage: Arc<dyn ObjectStorage + Send + Sync>,) -> Self {
        Self { db, sessions, storage }
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        info!("响应manager获取在线用户");
        self.sessions.get_all_online_users_tree().await // 返回一个克隆，可能比较影响效率
    }

    

    

}
