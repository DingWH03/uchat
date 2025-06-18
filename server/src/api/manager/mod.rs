pub mod session;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api::session_manager::{SessionInfo, SessionManager};
use crate::db::DB;
use crate::protocol::{ManagerResponse, RoleType};
use log::{info,error};

pub struct Manager {
    db: Arc<dyn DB>,
    sessions: Arc<RwLock<SessionManager>>,
}

impl Manager {
    pub fn new(db: Arc<dyn DB>, sessions: Arc<RwLock<SessionManager>>) -> Self {
        Self { db, sessions }
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        info!("响应manager获取在线用户");
        let sessions = self.sessions.read().await;
        sessions.get_all_online_users_tree() // 返回一个克隆，可能比较影响效率
    }

    /// 获取总用户人数
    pub async fn get_users_count(&self) -> ManagerResponse<u32> {
        info!("响应获取总用户人数");
        match self.db.get_user_count().await {
            Ok(num) => ManagerResponse::ok("获取成功", num),
            Err(e) => {
                error!("获取总用户人数失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }

    /// 修改用户身份
    pub async fn set_user_role(&self, user_id: u32, role: RoleType) -> ManagerResponse<()> {
        info!("修改用户{}身份为{}", user_id, role.to_string());
        match self.db.change_user_role(user_id, role).await {
            Ok(_) => {ManagerResponse::ok("修改成功", ())},
            Err(e) => {
                error!("修改用户身份失败失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }

}
