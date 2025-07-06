use std::collections::HashMap;

use log::info;

use uchat_protocol::{ManagerResponse, RoleType, manager::OnlineUserTree};

use super::Manager;

impl Manager {
    /// 移除一个指定 session
    pub async fn remove_session(&self, session_id: &str) -> ManagerResponse<()> {
        info!("响应manager移除session: {}", session_id);
        self.sessions.delete_session(session_id).await;
        ManagerResponse::ok("移除成功", ())
    }

    /// 移除某个用户的所有 session
    pub async fn remove_user_sessions(&self, user_id: u32) -> ManagerResponse<()> {
        info!("响应manager移除user: {}", user_id);
        if let Some(session_ids) = self.sessions.get_sessions_by_user(user_id).await {
            for session_id in session_ids {
                self.sessions.delete_session(&session_id).await;
            }
        }
        ManagerResponse::ok("移除成功", ())
    }

    /// 清除所有 session（慎用）
    pub async fn remove_all_sessions(&self) -> ManagerResponse<()> {
        info!("响应manager移除所有的session");
        self.sessions.clear_all_sessions().await;
        ManagerResponse::ok("移除成功", ())
    }

    /// 获取session_id权限
    pub async fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        self.sessions.check_session_role(session_id).await
    }

    /// 获取在线用户及其 session_id
    pub async fn get_online_user(&self) -> ManagerResponse<OnlineUserTree> {
        info!("响应 manager 获取在线用户");

        // 获取所有在线用户树的原始数据
        let session_map = self.sessions.get_all_online_users_tree().await;

        // 创建一个新的 HashMap 来存放转换后的 UserSessionInfo 数据
        let mut users = HashMap::with_capacity(session_map.len());

        // 遍历每个用户及其会话列表
        for (user_id, sessions) in session_map {
            let mut session_infos = Vec::with_capacity(sessions.len());

            // 将每个 SessionInfo 转换为 UserSessionInfo
            for (session_id, info) in sessions {
                session_infos.push(info.to_pub(session_id));
            }

            users.insert(user_id, session_infos);
        }

        // 构造 OnlineUserTree 并返回响应
        ManagerResponse::ok("获取成功", OnlineUserTree { users })
    }
}
