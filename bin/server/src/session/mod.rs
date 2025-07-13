// src/session/mod.rs
#[cfg(not(feature = "redis-support"))]
pub mod memory;
#[cfg(not(feature = "redis-support"))]
pub use crate::session::memory::SessionConfig;
#[cfg(feature = "redis-support")]
pub mod redis;
#[cfg(feature = "redis-support")]
pub use crate::session::redis::SessionConfig;
use async_trait::async_trait;
use axum::extract::ws::Message;
use chrono::DateTime;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::{IpAddr}, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use uchat_protocol::{RoleType, manager::UserSessionInfo};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionInfo {
    user_id: u32,
    pub created_at_secs: i64,
    pub created_at_nsecs: u32, // 用于存储创建时间的秒数
    pub ip: Option<IpAddr>,
    role: RoleType,
}
impl SessionInfo {
    // 转成带时区的 DateTime，用于格式化显示或序列化
    pub fn created_at_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        DateTime::from_timestamp(self.created_at_secs, self.created_at_nsecs).unwrap()
    }
    // 为 SessionInfo 实现一个方法来转换成 UserSessionInfo
    pub fn to_pub(&self, session_id: String) -> UserSessionInfo {
        UserSessionInfo {
            session_id,
            user_id: self.user_id,
            created_at: self.created_at_datetime(),
            ip: self.ip.map(|ip| ip.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct SenderStore {
    inner: Arc<DashMap<String, UnboundedSender<Message>>>,
}

impl SenderStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, session_id: &str, sender: UnboundedSender<Message>) {
        self.inner.insert(session_id.to_string(), sender);
    }

    pub fn remove(&self, session_id: &str) -> Option<UnboundedSender<Message>> {
        if let Some(sender) = self.inner.get(session_id) {
            // 尝试发送关闭消息
            let _ = sender.send(Message::Close(None));
        }
        // 移除并返回
        self.inner.remove(session_id).map(|(_, sender)| sender)
    }

    pub fn send(&self, session_id: &str, msg: Message) {
        if let Some(sender) = self.inner.get(session_id) {
            let _ = sender.send(msg);
        }
    }

    pub fn broadcast(&self, session_ids: &[String], msg: Message) {
        for session_id in session_ids {
            self.send(session_id, msg.clone());
        }
    }

    pub fn clear_all(&self) {
        for entry in self.inner.iter() {
            let _ = entry.send(Message::Close(None));
        }
        self.inner.clear();
    }
}

#[async_trait]
pub trait SessionManagerTrait: Send + Sync {
    type Config: Send + Sync;

    async fn new_with_config(config: Self::Config) -> Arc<Self>
    where
        Self: Sized;

    async fn insert_session(
        &self,
        user_id: u32,
        session_id: String,
        ip: Option<IpAddr>,
        role: RoleType,
    );
    async fn check_session(&self, session_id: &str) -> Option<u32>;
    async fn check_session_role(&self, session_id: &str) -> Option<RoleType>;
    async fn get_sessions_by_user(&self, user_id: u32) -> Option<Vec<String>>;
    async fn register_sender(
        &self,
        session_id: &str,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    );
    async fn unregister_sender(&self, session_id: &str);
    async fn delete_session(&self, session_id: &str);
    async fn send_to_user(&self, user_id: u32, msg: Message);
    async fn send_to_session(&self, session_id: &str, msg: Message);
    async fn get_all_online_users_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>>;
    async fn clear_all_sessions(&self);
}

/// 工厂函数，根据 feature 选择 SessionManager 实现
pub async fn create_session_manager(
    config: SessionConfig,
) -> Arc<dyn SessionManagerTrait<Config = SessionConfig>> {
    #[cfg(not(feature = "redis-support"))]
    {
        let manager = memory::SessionManager::new_with_config(config).await;
        manager
    }

    // redis-support 分支:
    #[cfg(feature = "redis-support")]
    {
        redis::RedisSessionManager::new_with_config(config).await
    }
}
