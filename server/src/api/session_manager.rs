use axum::extract::ws::Message;
use chrono::{DateTime};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use crate::protocol::RoleType;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    user_id: u32,
    sender: Option<UnboundedSender<Message>>, // 初始为 None
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
    pub fn insert_session(&mut self, user_id: u32, session_id: String, ip: Option<String>, role: RoleType) {

        self.sessions.insert(
            session_id.clone(),
            SessionInfo {
                user_id,
                sender: None,
                created_at_secs: chrono::Utc::now().timestamp(),
                created_at_nsecs: chrono::Utc::now().timestamp_subsec_nanos(),
                ip,
                role
            },
        );

        self.user_index
            .entry(user_id)
            .or_default()
            .insert(session_id);
    }

    /// 检查某个 session 是否存在
    pub fn check_session(&self, session_id: &str) -> Option<u32> {
        if let Some(session_info) = self.sessions.get(session_id) {
            // session只要存在即可，暂时无需判断时间
            Some(session_info.user_id)
        } else {
            None
        }
    }

    /// 检查某个 session 的身份
    pub fn check_session_role(&self, session_id: &str) -> Option<RoleType> {
        if let Some(session_info) = self.sessions.get(session_id) {
            Some(session_info.role)
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

    /// 发送消息到某个特定的会话
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

    /// 清除所有的会话
    pub fn clear_all_sessions(&mut self) {
        // 主动关闭每个 session 的 WebSocket
        for (_session_id, session) in self.sessions.drain() {
            if let Some(sender) = session.sender {
                let _ = sender.send(Message::Close(None)); // 忽略发送错误
            }
        }

        // 清空 user_index 映射
        self.user_index.clear();
    }

    /// 获取所有在线用户的树形结构（不进行 clone，返回引用）
    pub fn get_all_online_users_tree(&self) -> HashMap<u32, Vec<(String, SessionInfo)>> {
        let mut map: HashMap<u32, Vec<(String, SessionInfo)>> = HashMap::new();

        for (session_id, info) in &self.sessions {
            map.entry(info.user_id)
                .or_default()
                .push((session_id.clone(), info.clone()));
        }

        map
    }
}
