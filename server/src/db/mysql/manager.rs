use super::MysqlDB;
use crate::{
    db::{ManagerDB, error::DBError},
    protocol::{RecentPrivateMessage, RoleType, UserSimpleInfo},
};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
impl ManagerDB for MysqlDB {
    /// 用户总数量
    async fn get_user_count(&self) -> Result<u32, DBError> {
        let row = sqlx::query!("SELECT COUNT(id) as count FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.count as u32)
    }
    /// 获取全部用户
    async fn get_all_user(&self) -> Result<Vec<UserSimpleInfo>, DBError> {
        let rows = sqlx::query!(
            r#"
            SELECT id as user_id, username
            FROM users
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| UserSimpleInfo {
                user_id: row.user_id,
                username: row.username,
            })
            .collect();

        Ok(users)
    }
    /// 改变身份
    async fn change_user_role(&self, userid: u32, role: RoleType) -> Result<(), DBError> {
        sqlx::query!(
            r#"
        UPDATE users
        SET role = ?
        WHERE id = ?
        "#,
            role,
            userid
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
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(count)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
    /// 获取某用户前N条消息(包括收发的)
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
            WHERE sender_id = ? OR receiver_id = ?
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(user_id)
        .bind(count)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }
    /// 根据message_id删除某条聊天记录
    async fn delete_private_message(&self, message_id: u64) -> Result<u64, DBError> {
        let result = sqlx::query!("DELETE FROM messages WHERE id = ?", message_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
