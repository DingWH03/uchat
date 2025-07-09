use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UserProfileUpdated,
    UserLogined,
    UserLoggedOut,
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
    FriendsOf
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: u64,         // u64 类型的事件 ID
    pub event_type: EventType,       // 事件类型
    pub actor_id: u32,         // 发起人
    pub scope_type: EventScope,       // "USER", "GROUP"
    pub scope_id: u32,         // "user_A", "group_123"
    pub timestamp: i64, // 时间戳，单位为毫秒
    pub payload: Value,           // 动态字段
}

#[async_trait]
pub trait EventManagerTrait: Send + Sync {
    type Config: Send + Sync;

    /// 初始化（配置支持）
    async fn new_with_config(config: Self::Config) -> Self
    where
        Self: Sized;

    /// 写入事件
    async fn insert_event(&self, event: Event) -> anyhow::Result<()>;

    /// 获取某用户相关事件（自己 + 好友 + 群组）——由客户端携带 id 请求
    async fn fetch_events(
        &self,
        user_id: u32,
        watch_ids: &[u32],
        since_ms: i64,
    ) -> anyhow::Result<Vec<Event>>;

    /// 批量写入事件（定期落地到 DB）
    async fn persist_events(&self) -> anyhow::Result<()>;

    /// 清除所有事件（可选）
    async fn clear_all(&self) -> anyhow::Result<()>;
}
