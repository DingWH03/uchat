use std::collections::HashMap;

use super::PgSqlDB;
use crate::{
    db::{MessageDB, error::DBError},
    protocol::{MessageType, SessionMessage},
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

#[async_trait]
impl MessageDB for PgSqlDB {
    /// 添加私聊信息聊天记录，返回消息的自增 ID
    async fn add_message(
        &self,
        sender: u32,
        receiver: u32,
        message_type: MessageType,
        message: &str,
    ) -> Result<u64, DBError> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO messages (sender_id, receiver_id, message_type, message)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            sender as i32,
            receiver as i32,
            message_type.to_string(), // 传字符串
            message
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.id as u64)
    }

    /// 添加离线消息记录
    async fn add_offline_message(
        &self,
        receiver_id: u32,
        is_group: bool,
        message_id: Option<u64>,
        group_message_id: Option<u64>,
    ) -> Result<(), DBError> {
        sqlx::query!(
            "INSERT INTO offline_messages (receiver_id, is_group, message_id, group_message_id)
            VALUES ($1, $2, $3, $4)",
            receiver_id as i32,
            is_group,
            message_id.map(|v| v as i32),
            group_message_id.map(|v| v as i32)
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加群聊信息聊天记录
    async fn add_group_message(
        &self,
        group_id: u32,
        sender: u32,
        message: &str,
    ) -> Result<u64, DBError> {
        let rec = sqlx::query!(
            r#"
            INSERT INTO ugroup_messages (group_id, sender_id, message)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            group_id as i32,
            sender as i32,
            message
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.id as u64)
    }

    /// 获取私聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError> {
        // 每页显示的消息数
        let limit = 30;
        // 计算要偏移的数量
        let offset_rows = offset * limit;

        // 不再使用 DATE_FORMAT，而是直接查询原始 timestamp 列
        let messages = sqlx::query!(
            r#"
            SELECT 
                sender_id,
                timestamp,
                message
            FROM messages
            WHERE 
                (sender_id = $1 AND receiver_id = $2)
                OR 
                (sender_id = $3 AND receiver_id = $4)
            ORDER BY timestamp ASC
            LIMIT $5
            OFFSET $6
            "#,
            sender as i32,
            receiver as i32,
            receiver as i32,
            sender as i32,
            limit as i32,
            offset_rows as i32
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .filter_map(|row| {
            row.timestamp.map(|ts| SessionMessage {
                sender_id: row.sender_id as u32,
                timestamp: ts,
                message: row.message,
            })
        })
        .collect();

        Ok(messages)
    }

    /// 获取群聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    async fn get_group_messages(
        &self,
        group_id: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>, DBError> {
        let limit = 30;
        let offset_rows = offset * limit;

        // 使用 query_as! 时，需要把表里的 `timestamp` 原样返回
        let rows = sqlx::query!(
            r#"
            SELECT
                sender_id,
                timestamp,
                message
            FROM ugroup_messages
            WHERE group_id = $1
            ORDER BY timestamp ASC
            LIMIT $2
            OFFSET $3
            "#,
            group_id as i32,
            limit as i64,
            offset_rows as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .filter_map(|row| {
                row.timestamp.map(|ts| SessionMessage {
                    sender_id: row.sender_id as u32,
                    timestamp: ts,
                    message: row.message,
                })
            })
            .collect();

        Ok(messages)
    }
    /// 获取某群聊最新一条消息时间戳
    async fn get_latest_timestamp_of_group(
        &self,
        group_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        let ts: Option<NaiveDateTime> = sqlx::query_scalar!(
            r#"
            SELECT "timestamp" as "timestamp: NaiveDateTime"
            FROM ugroup_messages
            WHERE group_id = $1
            ORDER BY "timestamp" DESC
            LIMIT 1
            "#,
            group_id as i32
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 用户加入群聊的所有的群消息最后的时间戳
    async fn get_latest_timestamps_of_all_groups(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>, DBError> {
        let result = sqlx::query!(
            r#"
            SELECT
                m.group_id,
                MAX(m."timestamp") as "timestamp: NaiveDateTime"
            FROM ugroup_messages m
            JOIN group_members um ON um.group_id = m.group_id
            WHERE um.user_id = $1
            GROUP BY m.group_id
            "#,
            user_id as i32
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result
            .into_iter()
            .filter_map(|row| row.timestamp.map(|ts| (row.group_id as u32, ts)))
            .collect())
    }
    /// 当前用户所有群聊中最新的一条消息的时间戳（全局最大）
    async fn get_latest_timestamp_of_all_group_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        let ts = sqlx::query_scalar!(
            r#"
            SELECT MAX(m."timestamp") as "timestamp: NaiveDateTime"
            FROM ugroup_messages m
            JOIN group_members gm ON m.group_id = gm.group_id
            WHERE gm.user_id = $1
            "#,
            user_id as i32
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 某个群某时间之后的消息
    async fn get_group_messages_after_timestamp(
        &self,
        group_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>, DBError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                sender_id,
                "timestamp" as "timestamp: NaiveDateTime",
                message
            FROM ugroup_messages
            WHERE group_id = $1
            AND "timestamp" > $2
            ORDER BY "timestamp" ASC
            "#,
            group_id as i32,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        let msgs = rows
            .into_iter()
            .filter_map(|row| {
                row.timestamp.map(|ts| SessionMessage {
                    sender_id: row.sender_id as u32,
                    timestamp: ts,
                    message: row.message,
                })
            })
            .collect();

        Ok(msgs)
    }
    // 当前用户所有群某时间之后的消息
    async fn get_all_group_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                m.group_id,
                m.sender_id,
                m."timestamp" as "timestamp: NaiveDateTime",
                m.message
            FROM ugroup_messages m
            JOIN group_members um ON m.group_id = um.group_id
            WHERE um.user_id = $1
            AND m."timestamp" > $2
            ORDER BY m."timestamp" ASC
            "#,
            user_id as i32,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                row.timestamp.map(|ts| {
                    (
                        row.group_id as u32,
                        SessionMessage {
                            sender_id: row.sender_id as u32,
                            timestamp: ts,
                            message: row.message,
                        },
                    )
                })
            })
            .collect())
    }
    /// 获取与某个用户的最后一条私聊消息时间戳
    async fn get_latest_timestamp_with_user(
        &self,
        user1_id: u32,
        user2_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        let ts = sqlx::query_scalar!(
            r#"
        SELECT MAX("timestamp") as "timestamp: NaiveDateTime"
        FROM messages
        WHERE (sender_id = $1 AND receiver_id = $2)
           OR (sender_id = $3 AND receiver_id = $4)
        "#,
            user1_id as i32,
            user2_id as i32,
            user2_id as i32,
            user1_id as i32
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 获取当前用户所有私聊会话的最后时间戳（按对方用户 ID 映射）
    async fn get_latest_timestamps_of_all_private_chats(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>, DBError> {
        let rows = sqlx::query!(
            r#"
        SELECT
            CASE
                WHEN sender_id = $1 THEN receiver_id
                ELSE sender_id
            END as peer_id,
            MAX("timestamp") as "timestamp: NaiveDateTime"
        FROM messages
        WHERE sender_id = $2 OR receiver_id = $3
        GROUP BY peer_id
        "#,
            user_id as i32,
            user_id as i32,
            user_id as i32
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| match (row.peer_id, row.timestamp) {
                (Some(peer_id), Some(ts)) => Some((peer_id as u32, ts)),
                _ => None,
            })
            .collect())
    }
    /// 获取当前用户所有私聊中最新的一条消息时间戳（全局最大）
    async fn get_latest_timestamp_of_all_private_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<NaiveDateTime>, DBError> {
        let ts = sqlx::query_scalar!(
            r#"
        SELECT MAX("timestamp") as "timestamp: NaiveDateTime"
        FROM messages
        WHERE sender_id = $1 OR receiver_id = $2
        "#,
            user_id as i32,
            user_id as i32
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 获取与某个用户某时间之后的聊天记录（时间递增）
    async fn get_private_messages_after_timestamp(
        &self,
        user1_id: u32,
        user2_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>, DBError> {
        let rows = sqlx::query!(
            r#"
        SELECT
            sender_id,
            "timestamp" as "timestamp: NaiveDateTime",
            message
        FROM messages
        WHERE ((sender_id = $1 AND receiver_id = $2) OR (sender_id = $3 AND receiver_id = $4))
          AND "timestamp" > $5
        ORDER BY "timestamp" ASC
        "#,
            user1_id as i32,
            user2_id as i32,
            user2_id as i32,
            user1_id as i32,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .filter_map(|row| {
                row.timestamp.map(|ts| SessionMessage {
                    sender_id: row.sender_id as u32,
                    timestamp: ts,
                    message: row.message,
                })
            })
            .collect();

        Ok(messages)
    }
    /// 获取所有私聊消息中某时间之后的所有聊天记录（带对方 ID）
    async fn get_all_private_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>, DBError> {
        let rows = sqlx::query!(
            r#"
        SELECT
            CASE
                WHEN sender_id = $1 THEN receiver_id
                ELSE sender_id
            END as peer_id,
            sender_id as "sender_id!",
            "timestamp" as "timestamp!: NaiveDateTime",
            message as "message!"
        FROM messages
        WHERE (sender_id = $2 OR receiver_id = $3) AND "timestamp" > $4
        ORDER BY "timestamp" ASC
        "#,
            user_id as i32,
            user_id as i32,
            user_id as i32,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                r.peer_id.map(|peer_id| {
                    (
                        peer_id as u32,
                        SessionMessage {
                            sender_id: r.sender_id as u32,
                            timestamp: r.timestamp,
                            message: r.message,
                        },
                    )
                })
            })
            .collect())
    }
}
