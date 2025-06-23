
use log::info;

use crate::{protocol::{manager::OnlineUserTree, ManagerResponse, RoleType}};

use super::Manager;

impl Manager {
     /// 移除一个指定 session
    pub async fn remove_session(&self, session_id: &str) -> ManagerResponse<()> {
        info!("响应manager移除session: {}", session_id);
        self.sessions.delete_session(session_id).await;
        ManagerResponse::ok("移除成功",())
    }

    /// 移除某个用户的所有 session
    pub async fn remove_user_sessions(&self, user_id: u32) -> ManagerResponse<()> {
        info!("响应manager移除user: {}", user_id);
        if let Some(session_ids) = self.sessions.get_sessions_by_user(user_id).await {
            for session_id in session_ids {
                self.sessions.delete_session(&session_id).await;
            }
        }
        ManagerResponse::ok("移除成功",())
    }

    /// 清除所有 session（慎用）
    pub async fn remove_all_sessions(&self) -> ManagerResponse<()> {
        info!("响应manager移除所有的session");
        self.sessions.clear_all_sessions().await;
        ManagerResponse::ok("移除成功",())
    }

    /// 获取session_id权限
    pub async fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        self.sessions.check_session_role(session_id).await
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user(&self) -> ManagerResponse<OnlineUserTree> {
        info!("响应manager获取在线用户");
        ManagerResponse::ok(
            "获取成功",
            OnlineUserTree::from(self.sessions.get_all_online_users_tree().await),
        ) // 返回一个克隆，可能比较影响效率
    }
}