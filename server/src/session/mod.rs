// src/session/mod.rs
#[cfg(not(feature = "redis-support"))]
pub mod memory;
#[cfg(not(feature = "redis-support"))]
pub use crate::session::memory::SessionConfig;
use async_trait::async_trait;
use axum::extract::ws::Message;
use dashmap::DashMap;
use tokio::sync::{mpsc::UnboundedSender};
use crate::protocol::RoleType;
use std::{collections::HashMap, sync::Arc};
use chrono::{DateTime};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    user_id: u32,
    pub created_at_secs: i64,
    pub created_at_nsecs: u32, // 用于存储创建时间的秒数
    pub ip: Option<String>,
    role: RoleType,
}
impl SessionInfo {
    // 转成带时区的 DateTime，用于格式化显示或序列化
    pub fn created_at_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        DateTime::from_timestamp(self.created_at_secs, self.created_at_nsecs).unwrap()
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

    async fn insert_session(&self, user_id: u32, session_id: String, ip: Option<String>, role: RoleType);
    async fn check_session(&self, session_id: &str) -> Option<u32>;
    async fn check_session_role(&self, session_id: &str) -> Option<RoleType>;
    async fn get_sessions_by_user(&self, user_id: u32) -> Option<Vec<String>>;
    async fn register_sender(&self, session_id: &str, sender: tokio::sync::mpsc::UnboundedSender<Message>);
    async fn unregister_sender(&self, session_id: &str);
    async fn delete_session(&self, session_id: &str);
    async fn send_to_user(&self, user_id: u32, msg: Message);
    async fn send_to_session(&self, session_id: &str, msg: Message);
    async fn get_user_id_by_session(&self, session_id: &str) -> Option<u32>;
    async fn get_all_online_users_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>>;
    async fn clear_all_sessions(&self);
}

/// 工厂函数，根据 feature 选择 SessionManager 实现
pub async fn create_session_manager() -> Arc<dyn SessionManagerTrait<Config = SessionConfig>> {
    #[cfg(not(feature = "redis-support"))]
    {
        let config = SessionConfig {
            // 这里写内存配置内容，若无可空结构体也行
        };
        let manager = memory::SessionManager::new_with_config(config).await;
        manager
    }
    
    // 你后续可增加 redis-support 分支:
    /*
    #[cfg(feature = "redis-support")]
    {
        let config = RedisSessionConfig {
            // 填写 Redis 配置
        };
        let manager = redis::RedisSessionManager::new_with_config(config).await;
        manager
    }
    */
}