use std::collections::{HashMap, HashSet};
use axum::extract::ws::Message;
use tokio::sync::mpsc::UnboundedSender;
use chrono::NaiveDateTime;

struct SessionInfo {
    user_id: u32,
    sender: Option<UnboundedSender<Message>>, // 初始为 None
    created_at: NaiveDateTime,
    ip: Option<String>,
}

pub struct SessionManager {
    sessions: HashMap<String, SessionInfo>,
    user_index: HashMap<u32, HashSet<String>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            user_index: HashMap::new(),
        }
    }

    /// 插入一个新的 Session，占位但尚未绑定 Sender
    /// session_id可能会重复，但暂时不考虑
    pub fn insert_session(&mut self, user_id: u32, session_id: String, ip: Option<String>) {
        let now = chrono::Utc::now().naive_utc();

        self.sessions.insert(session_id.clone(), SessionInfo {
            user_id,
            sender: None,
            created_at: now,
            ip,
        });

        self.user_index.entry(user_id).or_default().insert(session_id);
    }

    /// 检查某个 session 是否存在
    pub fn check_session(&self, session_id: &str) -> Option<u32> {
        if let Some(session_info) = self.sessions.get(session_id) {
            // session只要存在即可，暂时无需判断时间
            Some(session_info.user_id)
        }
        else {
            None
        }
    }

    /// 获取某user的所有 session_id
    pub fn get_sessions_by_user(&self, user_id: u32) -> Option<&HashSet<String>> {
        self.user_index.get(&user_id)
    }

    /// 为某个 session 绑定 WebSocket Sender
    pub fn register_sender(&mut self, session_id: &str, sender: UnboundedSender<Message>) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.sender = Some(sender);
        }
    }

    /// 撤销某个 session 的 WebSocket Sender
    pub fn unregister_sender(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if let Some(sender) = &session.sender {
                let _ = sender.send(Message::Close(None)); // 尝试优雅关闭
            }
            session.sender = None;
        }
    }

    /// 删除 session（退出或断开连接）
    pub fn delete_session(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.remove(session_id) {
            // 如果有 sender，尝试发送 Close 消息（优雅断开 WebSocket）
            if let Some(sender) = &session.sender {
                // 忽略 send 失败，说明连接可能已断开
                let _ = sender.send(Message::Close(None));
            }
            if let Some(set) = self.user_index.get_mut(&session.user_id) {
                set.remove(session_id);
                if set.is_empty() {
                    self.user_index.remove(&session.user_id);
                }
            }
        }
    }

    /// 发送消息给某个用户的所有 WebSocket 连接
    pub fn send_to_user(&self, user_id: u32, msg: Message) {
        if let Some(session_ids) = self.user_index.get(&user_id) {
            for session_id in session_ids {
                if let Some(info) = self.sessions.get(session_id) {
                    if let Some(sender) = &info.sender {
                        let _ = sender.send(msg.clone());
                    }
                }
            }
        }
    }

    pub fn send_to_session(&self, session_id: &str, msg: Message) {
        if let Some(info) = self.sessions.get(session_id) {
            if let Some(sender) = &info.sender {
                let _ = sender.send(msg.clone());
            }
        }
    }

    /// 根据session_id获取用户ID
    pub fn get_user_id_by_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.get(session_id).map(|s| s.user_id)
    }
}
