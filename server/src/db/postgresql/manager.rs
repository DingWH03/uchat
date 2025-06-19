use std::collections::HashMap;

use super::MysqlDB;
use crate::{
    db::{ManagerDB, MessageDB, error::DBError},
    protocol::{GroupSessionMessage, MessageType, RoleType, SessionMessage},
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

#[async_trait]
impl ManagerDB for MysqlDB {
    async fn get_user_count(&self) -> Result<u32, DBError> {
        let row = sqlx::query!("SELECT COUNT(id) as count FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.count as u32)
    }
    async fn change_user_role(&self, userid: u32, role: RoleType) -> Result<(), DBError> {
        sqlx::query!(
            r#"
        UPDATE users
        SET role = $1
        WHERE id = $2
        "#,
            role.to_string(),
            user_id as i32
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    /// 获取服务器前N条消息
    async fn get_recent_messages(
        &self,
        count: u32,
        offset: u32,
    ) -> Result<Vec<RecentPrivateMessage>, DBError> {
        let messages = sqlx::query_as::<_, RecentPrivateMessage>(
            r#"
            SELECT 
                id,
                sender_id,
                sender_username,
                receiver_id,
                receiver_username,
                message_type,
                message_preview,
                timestamp
            FROM recent_private_messages_view
            ORDER BY timestamp DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(count as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)  // 假设你在 impl 中有 `self.pool: PgPool`
        .await?;

        Ok(messages)
    }
    /// 获取某用户前N条消息
    async fn get_user_recent_messages(
        &self,
        count: u32,
        offset: u32,
        user_id: u32,
    ) -> Result<Vec<RecentPrivateMessage>, DBError> {
        let messages = sqlx::query_as::<_, RecentPrivateMessage>(
            r#"
            SELECT 
                id,
                sender_id,
                sender_username,
                receiver_id,
                receiver_username,
                message_type,
                message_preview,
                timestamp
            FROM recent_private_messages_view
            WHERE sender_id = $1 OR receiver_id = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id as i32)
        .bind(count as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
    /// 根据message_id删除某条聊天记录
    async fn delete_private_message(
        &self,
        message_id: i32
    ) -> Result<u64, DBError> {
        let result = sqlx::query!(
            "DELETE FROM messages WHERE id = $1",
            message_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
