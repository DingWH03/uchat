use super::Request;
use axum::extract::ws::Message;
use uchat_protocol::{request::RequestResponse, RoleType};

impl Request {
    /// 检查session_id是否存在，返回user_id
    pub async fn check_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.check_session(session_id).await
    }

    /// 检查session_id是否存在，返回role
    pub async fn check_session_role(&self, session_id: &str) -> RequestResponse<RoleType> {
        match self.sessions.check_session_role(session_id).await {
            Some(role) => RequestResponse::ok("获取成功", role),
            None => RequestResponse::unauthorized(),
        }
    }

    /// 登陆session sender
    pub async fn register_session(
        &self,
        session_id: &str,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ) {
        self.sessions.register_sender(session_id, sender).await;
    }

    /// 撤销sender
    pub async fn unregister_session(&self, session_id: &str) {
        self.sessions.unregister_sender(session_id).await;
    }
}