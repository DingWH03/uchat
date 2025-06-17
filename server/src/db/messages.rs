use std::collections::HashMap;

use crate::protocol::{GroupSessionMessage, SessionMessage};
use super::Database;
use anyhow::Result;
use chrono::NaiveDateTime;

impl Database{

    /// 获取私聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    pub async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>> {
        // 每页显示的消息数
        let limit = 30;
        // 计算要偏移的数量
        let offset_rows = offset * limit;

        // 不再使用 DATE_FORMAT，而是直接查询原始 timestamp 列
        let messages = sqlx::query_as!(
            SessionMessage,
            r#"
            SELECT 
                sender_id AS `sender_id!`,
                `timestamp` AS `timestamp!: NaiveDateTime`,
                message AS `message!`
            FROM messages
            WHERE 
                (sender_id = ? AND receiver_id = ?)
                OR 
                (sender_id = ? AND receiver_id = ?)
            ORDER BY `timestamp` ASC
            LIMIT ?
            OFFSET ?
            "#,
            sender,
            receiver,
            receiver,
            sender,
            limit,
            offset_rows
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    /// 获取群聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    pub async fn get_group_messages(
        &self,
        group_id: u32,
        offset: u32,
    ) -> Result<Vec<SessionMessage>> {
        let limit = 30;
        let offset_rows = offset * limit;

        // 使用 query_as! 时，需要把表里的 `timestamp` 原样返回
        let messages = sqlx::query_as!(
            SessionMessage,
            r#"
            SELECT
                sender_id as `sender_id!`,
                `timestamp` as `timestamp!: NaiveDateTime`,
                message     as `message!`
            FROM ugroup_messages
            WHERE group_id = ?
            ORDER BY `timestamp` ASC
            LIMIT ?
            OFFSET ?
            "#,
            group_id,
            limit,
            offset_rows
        )
        .fetch_all(&self.pool)
        .await?;
        // 这里 messages 就已经是 Vec<Message>，无需再手动解析
        Ok(messages)
    }
    /// 获取某群聊最新一条消息时间戳
    pub async fn get_latest_timestamp_of_group(
        &self,
        group_id: u32,
    ) -> Result<Option<NaiveDateTime>> {
        let ts: Option<NaiveDateTime> = sqlx::query_scalar!(
            r#"
            SELECT `timestamp` as "timestamp: NaiveDateTime"
            FROM ugroup_messages
            WHERE group_id = ?
            ORDER BY `timestamp` DESC
            LIMIT 1
            "#,
            group_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 用户加入群聊的所有的群消息最后的时间戳
    pub async fn get_latest_timestamps_of_all_groups(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>> {
        let result = sqlx::query!(
            r#"
            SELECT
                m.group_id,
                MAX(m.`timestamp`) as "timestamp: NaiveDateTime"
            FROM ugroup_messages m
            JOIN group_members um ON um.group_id = m.group_id
            WHERE um.user_id = ?
            GROUP BY m.group_id
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result
            .into_iter()
            .filter_map(|row| row.timestamp.map(|ts| (row.group_id, ts)))
            .collect())
    }
    /// 当前用户所有群聊中最新的一条消息的时间戳（全局最大）
    pub async fn get_latest_timestamp_of_all_group_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<NaiveDateTime>> {
        let ts = sqlx::query_scalar!(
            r#"
            SELECT MAX(m.`timestamp`) as "timestamp: NaiveDateTime"
            FROM ugroup_messages m
            JOIN group_members gm ON m.group_id = gm.group_id
            WHERE gm.user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 某个群某时间之后的消息
    pub async fn get_group_messages_after_timestamp(
        &self,
        group_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>> {
        let msgs = sqlx::query_as!(
            SessionMessage,
            r#"
            SELECT
                sender_id as "sender_id!",
                `timestamp` as "timestamp!: NaiveDateTime",
                message as "message!"
            FROM ugroup_messages
            WHERE group_id = ?
            AND `timestamp` > ?
            ORDER BY `timestamp` ASC
            "#,
            group_id,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(msgs)
    }
    // 当前用户所有群某时间之后的消息
    pub async fn get_all_group_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>> {
        let rows = sqlx::query_as!(
            GroupSessionMessage,
            r#"
            SELECT
                m.group_id as `group_id!`,
                m.sender_id as `sender_id!`,
                m.`timestamp` as `timestamp!: NaiveDateTime`,
                m.message as `message!`
            FROM ugroup_messages m
            JOIN group_members um ON m.group_id = um.group_id
            WHERE um.user_id = ?
            AND m.`timestamp` > ?
            ORDER BY m.`timestamp` ASC
            "#,
            user_id,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.group_id,
                    SessionMessage {
                        sender_id: row.sender_id,
                        timestamp: row.timestamp,
                        message: row.message,
                    },
                )
            })
            .collect())
    }
    /// 获取与某个用户的最后一条私聊消息时间戳
    pub async fn get_latest_timestamp_with_user(
        &self,
        user1_id: u32,
        user2_id: u32,
    ) -> Result<Option<NaiveDateTime>> {
        let ts = sqlx::query_scalar!(
            r#"
        SELECT MAX(`timestamp`) as "timestamp: NaiveDateTime"
        FROM messages
        WHERE (sender_id = ? AND receiver_id = ?)
           OR (sender_id = ? AND receiver_id = ?)
        "#,
            user1_id,
            user2_id,
            user2_id,
            user1_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 获取当前用户所有私聊会话的最后时间戳（按对方用户 ID 映射）
    pub async fn get_latest_timestamps_of_all_private_chats(
        &self,
        user_id: u32,
    ) -> Result<HashMap<u32, NaiveDateTime>> {
        let rows = sqlx::query!(
            r#"
        SELECT
            CASE
                WHEN sender_id = ? THEN receiver_id
                ELSE sender_id
            END as peer_id,
            MAX(`timestamp`) as "timestamp: NaiveDateTime"
        FROM messages
        WHERE sender_id = ? OR receiver_id = ?
        GROUP BY peer_id
        "#,
            user_id,
            user_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|row| row.timestamp.map(|ts| (row.peer_id, ts)))
            .collect())
    }
    /// 获取当前用户所有私聊中最新的一条消息时间戳（全局最大）
    pub async fn get_latest_timestamp_of_all_private_messages(
        &self,
        user_id: u32,
    ) -> Result<Option<NaiveDateTime>> {
        let ts = sqlx::query_scalar!(
            r#"
        SELECT MAX(`timestamp`) as "timestamp: NaiveDateTime"
        FROM messages
        WHERE sender_id = ? OR receiver_id = ?
        "#,
            user_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(ts)
    }
    /// 获取与某个用户某时间之后的聊天记录（时间递增）
    pub async fn get_private_messages_after_timestamp(
        &self,
        user1_id: u32,
        user2_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<SessionMessage>> {
        let rows = sqlx::query_as!(
            SessionMessage,
            r#"
        SELECT
            sender_id as "sender_id!",
            `timestamp` as "timestamp!: NaiveDateTime",
            message as "message!"
        FROM messages
        WHERE ((sender_id = ? AND receiver_id = ?) OR (sender_id = ? AND receiver_id = ?))
          AND `timestamp` > ?
        ORDER BY `timestamp` ASC
        "#,
            user1_id,
            user2_id,
            user2_id,
            user1_id,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
    /// 获取所有私聊消息中某时间之后的所有聊天记录（带对方 ID）
    pub async fn get_all_private_messages_after_timestamp(
        &self,
        user_id: u32,
        after: NaiveDateTime,
    ) -> Result<Vec<(u32, SessionMessage)>> {
        let rows = sqlx::query!(
            r#"
        SELECT
            CASE
                WHEN sender_id = ? THEN receiver_id
                ELSE sender_id
            END as peer_id,
            sender_id as "sender_id!",
            `timestamp` as "timestamp!: NaiveDateTime",
            message as "message!"
        FROM messages
        WHERE (sender_id = ? OR receiver_id = ?) AND `timestamp` > ?
        ORDER BY `timestamp` ASC
        "#,
            user_id,
            user_id,
            user_id,
            after
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.peer_id,
                    SessionMessage {
                        sender_id: r.sender_id,
                        timestamp: r.timestamp,
                        message: r.message,
                    },
                )
            })
            .collect())
    }
}