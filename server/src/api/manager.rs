use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api::session_manager::{SessionInfo, SessionManager};
use crate::db::Database as DB;
use log::info;

pub struct Manager {
    db: Arc<DB>,
    sessions: Arc<RwLock<SessionManager>>,
}

impl Manager {
    pub fn new(db: Arc<DB>, sessions: Arc<RwLock<SessionManager>>) -> Self {
        Self { db, sessions }
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        info!("响应manager获取在线用户");
        let sessions = self.sessions.read().await;
        sessions.get_all_online_users_tree() // 返回一个克隆，可能比较影响效率
    }

    /// 移除一个指定 session
    pub async fn remove_session(&self, session_id: &str) {
        info!("响应manager移除session: {}", session_id);
        let mut sessions_write = self.sessions.write().await;
        sessions_write.delete_session(session_id);
    }

    /// 移除某个用户的所有 session
    pub async fn remove_user_sessions(&self, user_id: u32) {
        info!("响应manager移除user: {}", user_id);
        let mut sessions_write = self.sessions.write().await;
        if let Some(session_ids) = sessions_write.get_sessions_by_user(user_id).cloned() {
            for session_id in session_ids {
                sessions_write.delete_session(&session_id);
            }
        }
    }

    /// 清除所有 session（慎用）
    pub async fn remove_all_sessions(&self) {
        info!("响应manager移除所有的session");
        let mut sessions = self.sessions.write().await;
        sessions.clear_all_sessions();
    }

}
