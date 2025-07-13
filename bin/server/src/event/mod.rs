pub mod model;
pub mod memory;
use model::Event;
use async_trait::async_trait;

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
