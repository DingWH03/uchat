use async_trait::async_trait;
use axum::extract::ws::Message;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::protocol::RoleType;
use crate::session::{SenderStore, SessionInfo, SessionManagerTrait};
use crate::redis::SharedRedis;

pub struct SessionConfig {
    pub redis: SharedRedis,
    pub session_expire_secs: i64,
}

pub struct RedisSessionManager {
    redis: SharedRedis,
    sender_store: SenderStore,
    expire_secs: i64,
}

#[async_trait]
impl SessionManagerTrait for RedisSessionManager {
    type Config = SessionConfig;

    async fn new_with_config(config: Self::Config) -> Arc<Self> {
        Arc::new(Self {
            redis: config.redis,
            sender_store: SenderStore::new(),
            expire_secs: config.session_expire_secs,
        })
    }

    async fn insert_session(&self, user_id: u32, session_id: String, ip: Option<String>, role: RoleType) {
        let session = SessionInfo {
            user_id,
            created_at_secs: Utc::now().timestamp(),
            created_at_nsecs: Utc::now().timestamp_subsec_nanos(),
            ip,
            role,
        };
        let session_json = serde_json::to_string(&session).unwrap();
        let _ = self.redis.set_with_expire(&format!("session:{}", session_id), &session_json, self.expire_secs).await;
        let _ = self.redis.sadd(&format!("user_sessions:{}", user_id), &session_id).await;
    }

    async fn check_session(&self, session_id: &str) -> Option<u32> {
        let key = format!("session:{}", session_id);
        let result = self.redis.get_and_refresh(&key, self.expire_secs).await.ok().flatten()?;
        let session: SessionInfo = serde_json::from_str(&result).ok()?;
        Some(session.user_id)
    }

    async fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        let key = format!("session:{}", session_id);
        let result = self.redis.get_and_refresh(&key, self.expire_secs).await.ok().flatten()?;
        let session: SessionInfo = serde_json::from_str(&result).ok()?;
        Some(session.role)
    }

    async fn get_sessions_by_user(&self, user_id: u32) -> Option<Vec<String>> {
        self.redis.smembers(&format!("user_sessions:{}", user_id)).await.ok()
    }

    async fn register_sender(&self, session_id: &str, sender: UnboundedSender<Message>) {
        self.sender_store.insert(session_id, sender);
    }

    async fn unregister_sender(&self, session_id: &str) {
        if let Some(sender) = self.sender_store.remove(session_id) {
            let _ = sender.send(Message::Close(None));
        }
    }

    async fn delete_session(&self, session_id: &str) {
        if let Some(result) = self.redis.get(&format!("session:{}", session_id)).await.ok().flatten() {
            let session: SessionInfo = serde_json::from_str(&result).unwrap();
            let _ = self.redis.del(&format!("session:{}", session_id)).await;
            let _ = self.redis.srem(&format!("user_sessions:{}", session.user_id), session_id).await;
            self.sender_store.remove(session_id);
        }
    }

    async fn send_to_user(&self, user_id: u32, msg: Message) {
        if let Ok(session_ids) = self.redis.smembers(&format!("user_sessions:{}", user_id)).await {
            self.sender_store.broadcast(&session_ids, msg);
        }
    }

    async fn send_to_session(&self, session_id: &str, msg: Message) {
        self.sender_store.send(session_id, msg);
    }

    async fn get_user_id_by_session(&self, session_id: &str) -> Option<u32> {
        self.check_session(session_id).await
    }

    async fn clear_all_sessions(&self) {
        // 获取所有 session_id（遍历所有用户的 user_sessions）
        if let Ok(keys) = self.redis.scan_keys("user_sessions:*").await {
            for key in keys {
                let key = key.to_string();
                if let Ok(session_ids) = self.redis.smembers(&key).await {
                    // 删除所有 session:xxx 和 sender
                    for session_id in &session_ids {
                        let _ = self.redis.del(&format!("session:{}", session_id)).await;
                        self.sender_store.remove(session_id);
                    }
                }
                // 删除 user_sessions:uid
                let _ = self.redis.del(&key).await;
            }
        }

        // 发送关闭消息
        self.sender_store.clear_all();
    }


    async fn get_all_online_users_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        let mut result: HashMap<u32, Vec<(String, SessionInfo)>> = HashMap::new();

        if let Ok(keys) = self.redis.scan_keys("user_sessions:*").await {
            for key in keys {
                if let Some(user_id_str) = key.strip_prefix("user_sessions:") {
                    if let Ok(user_id) = user_id_str.parse::<u32>() {
                        if let Ok(session_ids) = self.redis.smembers(&key).await {
                            let full_keys: Vec<String> = session_ids.iter().map(|id| format!("session:{}", id)).collect();
                            if let Ok(sessions_json) = self.redis.mget(&full_keys).await {
                                for (session_id, json_opt) in session_ids.iter().zip(sessions_json) {
                                    if let Some(json) = json_opt {
                                        if let Ok(info) = serde_json::from_str::<SessionInfo>(&json) {
                                            result.entry(user_id).or_default().push((session_id.clone(), info));
                                        }
                                    }
                                }
                            }

                        }
                    }
                }
            }
        }

        result
    }

}
