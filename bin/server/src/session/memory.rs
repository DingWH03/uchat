use super::{SenderStore, SessionManagerTrait};
use crate::session::SessionInfo;
use async_trait::async_trait;
use axum::extract::ws::Message;
use dashmap::{DashMap, DashSet};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uchat_protocol::RoleType;

pub struct SessionConfig; // 空结构体，可扩展为带参数结构体

pub struct SessionManager {
    sessions: DashMap<String, SessionInfo>,
    user_index: DashMap<u32, DashSet<String>>,
    senders: SenderStore,
}

#[async_trait]
impl SessionManagerTrait for SessionManager {
    type Config = SessionConfig;

    async fn new_with_config(_: Self::Config) -> Arc<Self> {
        Arc::new(Self {
            sessions: DashMap::new(),
            user_index: DashMap::new(),
            senders: SenderStore::new(),
        })
    }
    async fn insert_session(
        &self,
        user_id: u32,
        session_id: String,
        ip: Option<String>,
        role: RoleType,
    ) {
        self.sessions.insert(
            session_id.clone(),
            SessionInfo {
                user_id,
                created_at_secs: chrono::Utc::now().timestamp(),
                created_at_nsecs: chrono::Utc::now().timestamp_subsec_nanos(),
                ip,
                role,
            },
        );
        self.user_index
            .entry(user_id)
            .or_insert_with(DashSet::new)
            .insert(session_id);
    }

    async fn check_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.get(session_id).map(|s| s.user_id)
    }

    async fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        self.sessions.get(session_id).map(|s| s.role)
    }

    async fn get_sessions_by_user(&self, user_id: u32) -> Option<Vec<String>> {
        self.user_index
            .get(&user_id)
            .map(|set| set.iter().map(|r| r.key().clone()).collect())
    }

    async fn register_sender(&self, session_id: &str, sender: UnboundedSender<Message>) {
        self.senders.insert(session_id, sender);
    }

    async fn unregister_sender(&self, session_id: &str) {
        if let Some(sender) = self.senders.remove(session_id) {
            let _ = sender.send(Message::Close(None));
        }
    }

    async fn delete_session(&self, session_id: &str) {
        // 阶段一：尝试从 sessions 中移除会话，并记录相关 user_id
        let user_id = self.sessions.remove(session_id).map(|(_, session)| {
            self.senders.remove(session_id);
            session.user_id
        });

        // 阶段二：如果有 user_id，则从 user_index 中移除 session_id
        if let Some(user_id) = user_id {
            // 注意：不嵌套访问 DashMap，防止死锁
            let should_remove_user_index = {
                if let Some(set) = self.user_index.get_mut(&user_id) {
                    set.remove(session_id);
                    set.is_empty()
                } else {
                    false
                }
            };

            // 如果用户不再拥有任何会话，则移除 user_index 映射
            if should_remove_user_index {
                self.user_index.remove(&user_id);
            }
        }
    }

    async fn send_to_user(&self, user_id: u32, msg: Message) {
        if let Some(set) = self.user_index.get(&user_id) {
            let ids: Vec<String> = set.iter().map(|r| r.key().clone()).collect();
            self.senders.broadcast(&ids, msg);
        }
    }

    async fn send_to_session(&self, session_id: &str, msg: Message) {
        self.senders.send(session_id, msg);
    }

    async fn get_user_id_by_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.get(session_id).map(|s| s.user_id)
    }

    async fn clear_all_sessions(&self) {
        self.senders.clear_all();
        self.sessions.clear();
        self.user_index.clear();
    }

    async fn get_all_online_users_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        let mut map: HashMap<u32, Vec<(String, SessionInfo)>> = HashMap::new();
        for entry in self.sessions.iter() {
            map.entry(entry.user_id)
                .or_default()
                .push((entry.key().clone(), entry.clone()));
        }
        map
    }
}
