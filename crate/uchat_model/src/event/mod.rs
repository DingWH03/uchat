pub mod content;

use serde::{Serialize, Deserialize};
use super::model::{UserId, GroupId, Timestamp, EventId};
use content::{EventContent};

// ---------- 精简后的事件结构 ----------

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind { System, User, Group }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind { LoginIn, UpdateProfile, MessageSent }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Event {
    pub event_id: EventId,
    pub timestamp: Timestamp,
    pub actor_kind: ActorKind,
    pub actor_user_id: Option<UserId>,
    pub actor_group_id: Option<GroupId>,
    pub event_kind: EventKind,

    // 根据不同事件类型，字段可能为空
    pub content: Option<EventContent>,
}

