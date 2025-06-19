
use log::info;

use crate::{protocol::{manager::OnlineUserTree, ManagerResponse, RoleType}};

use super::Manager;

impl Manager {
     /// 移除一个指定 session
    pub async fn remove_session(&self, session_id: &str) -> ManagerResponse<()> {
        info!("响应manager移除session: {}", session_id);
        let mut sessions_write = self.sessions.write().await;
        sessions_write.delete_session(session_id);
        ManagerResponse::ok("移除成功",())
    }

    /// 移除某个用户的所有 session
    pub async fn remove_user_sessions(&self, user_id: u32) -> ManagerResponse<()> {
        info!("响应manager移除user: {}", user_id);
        let mut sessions_write = self.sessions.write().await;
        if let Some(session_ids) = sessions_write.get_sessions_by_user(user_id).cloned() {
            for session_id in session_ids {
                sessions_write.delete_session(&session_id);
            }
        }
        ManagerResponse::ok("移除成功",())
    }

    /// 清除所有 session（慎用）
    pub async fn remove_all_sessions(&self) -> ManagerResponse<()> {
        info!("响应manager移除所有的session");
        let mut sessions = self.sessions.write().await;
        sessions.clear_all_sessions();
        ManagerResponse::ok("移除成功",())
    }

    /// 获取session_id权限
    pub async fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        let sessions = self.sessions.read().await;
        sessions.check_session_role(session_id)
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user(&self) -> ManagerResponse<OnlineUserTree> {
        info!("响应manager获取在线用户");
        let sessions = self.sessions.read().await;
        ManagerResponse::ok(
            "获取成功",
            OnlineUserTree::from(sessions.get_all_online_users_tree()),
        ) // 返回一个克隆，可能比较影响效率
    }
}