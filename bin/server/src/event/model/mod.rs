#[macro_use]
mod macros;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UserLogined,
    UserLoggedOut,
    UserProfileUpdated,
    GroupCreated,
    GroupUpdated,
    GroupDeleted,
    FriendAdded,
    FriendRemoved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventScope {
    User,
    Group,
    FriendsOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: u64,
    pub event_type: EventType,
    pub actor_id: u32,
    pub scope_type: EventScope,
    pub scope_id: u32,
    pub timestamp: i64,
    pub payload: Value,
}

impl Event {
    pub fn gen_event_id() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// 用于构建一个事件的通用函数
    pub fn new(
        event_type: EventType,
        actor_id: u32,
        scope_type: EventScope,
        scope_id: u32,
        payload: Value,
    ) -> Self {
        Self {
            event_id: Self::gen_event_id(),
            event_type,
            actor_id,
            scope_type,
            scope_id,
            timestamp: Utc::now().timestamp_millis(),
            payload,
        }
    }

}

define_event_factory! {
    /// 用户登录事件
    user_logined => (UserLogined, User, |user_id, actor_id, ip, device| {
        json!({ "ip": ip, "device": device })
    });

    /// 用户资料更新
    user_profile_updated => (UserProfileUpdated, User, |user_id, actor_id, changes| {
        json!(changes)
    });

    /// 群组创建事件
    group_created => (GroupCreated, Group, |group_id, actor_id, group_name| {
        json!({ "group_name": group_name })
    });

    /// 好友添加事件
    friend_added => (FriendAdded, FriendsOf, |friend_id, actor_id| {
        json!({ "friend_id": friend_id })
    });
}
