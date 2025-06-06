// src/db.rs

use crate::protocol::{GroupSimpleInfo, Message, UserDetailedInfo, UserSimpleInfo, GroupDetailedInfo};
use anyhow::Result;
use chrono::NaiveDateTime;
use dotenv::dotenv;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::env;

/// 结构体 `Database` 用于封装 MySQL 连接池
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// 异步初始化数据库连接池
    pub async fn new() -> Result<Self> {
        // 加载 .env 文件中的环境变量
        dotenv().ok();

        // 从环境变量中获取数据库连接字符串
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL 环境变量未设置");

        // 创建连接池，设置最大连接数为 20
        let pool = MySqlPoolOptions::new()
            .max_connections(20)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// 查询用户密码哈希
    pub async fn get_password_hash(&self, id: u32) -> Result<Option<String>> {
        let row = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.password_hash))
    }

    /// 创建新用户
    pub async fn new_user(&self, username: &str, password_hash: &str) -> Result<Option<u32>> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            username,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        // 获取插入的自增ID
        let last_insert_id = result.last_insert_id() as u32;

        Ok(Some(last_insert_id))
    }

    /// 根据id查找用户详细信息
    pub async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>> {
        let row = sqlx::query!(
            "SELECT id AS user_id, username FROM users WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserDetailedInfo {
            user_id: r.user_id,
            username: r.username,
        }))
    }

    /// 设置UserDetailedInfo用户信息，当前用户信息较少，以后会考虑单独设置某一部分，例如个性签名，头像等
    // pub async fn set_userinfo(&self, id: u32, userinfo: UserDetailedInfo) -> Result<()> {

    // }

    /// 根据group_id获取群聊详细信息
    pub async fn get_groupinfo(&self, group_id: u32) -> Result<Option<GroupDetailedInfo>> {
        let row = sqlx::query!(
            "SELECT id AS group_id, name AS title FROM ugroups WHERE id = ?",
            group_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GroupDetailedInfo {
            group_id: r.group_id,
            title: r.title,
        }))
    }

    /// 根据user_id🔍好友列表，一般是自己查找自己的好友列表
    pub async fn get_friends(&self, user_id: u32) -> Result<Vec<UserSimpleInfo>> {
        let rows = sqlx::query!(
            "
            SELECT 
                f.friend_id, 
                u.username 
            FROM 
                friendships f
            JOIN 
                users u 
            ON 
                f.friend_id = u.id
            WHERE 
                f.user_id = ?
            ",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
    
        // 将查询结果映射到 UserSimpleInfo 结构体
        Ok(rows.into_iter().map(|r| UserSimpleInfo {
            user_id: r.friend_id,
            username: r.username,
        }).collect())
    }

    /// 根据user_id🔍群组列表，一般是自己查找自己的群组列表
    pub async fn get_groups(&self, user_id: u32) -> Result<Vec<GroupSimpleInfo>> {
        let rows = sqlx::query!(
            "
            SELECT 
                gm.group_id, 
                g.name AS title 
            FROM 
                group_members gm
            JOIN 
                ugroups g 
            ON 
                gm.group_id = g.id
            WHERE 
                gm.user_id = ?
            ",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
    
        // 将查询结果映射到 GroupSimpleInfo 结构体
        Ok(rows.into_iter().map(|r| GroupSimpleInfo {
            group_id: r.group_id,
            title: r.title,
        }).collect())
    }

    /// 根据group_id🔍群组成员列表
    pub async fn get_group_members(&self, group_id: u32) -> Result<Vec<UserSimpleInfo>> {
        let rows = sqlx::query!(
            "
            SELECT 
                gm.user_id, 
                u.username 
            FROM 
                group_members gm
            JOIN 
                users u 
            ON 
                gm.user_id = u.id
            WHERE 
                gm.group_id = ?
            ",
            group_id
        )
        .fetch_all(&self.pool)
        .await?;
    
        // 将查询结果映射到 GroupMemberInfo 结构体
        Ok(rows.into_iter().map(|r| UserSimpleInfo {
            user_id: r.user_id,
            username: r.username,
        }).collect())
    }
    

    /// 创建群组，默认将创建者加入群聊
    pub async fn create_group(&self, user_id: u32, group_name: &str) -> Result<u32> {
        // 插入新的群组，包含名称和创建者ID
        let result = sqlx::query!(
            "INSERT INTO ugroups (name, creator_id) VALUES (?, ?)",
            group_name,
            user_id
        )
        .execute(&self.pool)
        .await?;

        // 获取新插入的群组ID
        let group_id = result.last_insert_id() as u32;

        // 将创建者加入群聊成员表
        sqlx::query!(
            "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
            group_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(group_id)
    }

    /// 添加好友，user_id是发送者的id，friend_id是接收者的id
    /// 直接双向成为好友，暂不支持请求与同意机制
    pub async fn add_friend(&self, user_id: u32, friend_id: u32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 插入 (user_id, friend_id)
        sqlx::query!(
            "INSERT IGNORE INTO friendships (user_id, friend_id) VALUES (?, ?)",
            user_id,
            friend_id
        )
        .execute(&mut *tx)
        .await?;

        // 插入 (friend_id, user_id)
        sqlx::query!(
            "INSERT IGNORE INTO friendships (user_id, friend_id) VALUES (?, ?)",
            friend_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }


    /// 添加群组成员，user_id是发送者的id，group_id是接收者的id
    pub async fn add_group(&self, user_id: u32, group_id: u32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_members (user_id, group_id) VALUES (?, ?)",
            user_id,
            group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加私聊信息聊天记录，返回消息的自增 ID
    pub async fn add_message(&self, sender: u32, receiver: u32, message: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO messages (sender_id, receiver_id, message) VALUES (?, ?, ?)",
            sender,
            receiver,
            message
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id())
    }

    /// 添加离线消息记录
    pub async fn add_offline_message(
        &self,
        receiver_id: u32,
        is_group: bool,
        message_id: Option<u64>,
        group_message_id: Option<u64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO offline_messages (receiver_id, is_group, message_id, group_message_id)
            VALUES (?, ?, ?, ?)",
            receiver_id,
            is_group,
            message_id,
            group_message_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加群聊信息聊天记录
    pub async fn add_group_message(
        &self,
        group_id: u32,
        sender: u32,
        message: &str,
    ) -> Result<u64> {
        let result = sqlx::query!(
            "INSERT INTO ugroup_messages (group_id, sender_id, message)
            VALUES (?, ?, ?)",
            group_id,
            sender,
            message
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id())
    }


    /// 获取私聊聊天记录
    /// 返回值为元组，元组的第一个元素是发送者的id，第二个元素是timestap，第三个元素是消息内容
    /// offset是消息分组，一组消息30条，0代表最近的30条，1代表30-60条，以此类推
    pub async fn get_messages(
        &self,
        sender: u32,
        receiver: u32,
        offset: u32,
    ) -> Result<Vec<Message>> {
        // 每页显示的消息数
        let limit = 30;
        // 计算要偏移的数量
        let offset_rows = offset * limit;

        // 不再使用 DATE_FORMAT，而是直接查询原始 timestamp 列
        let messages = sqlx::query_as!(
            Message,
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
            ORDER BY `timestamp` DESC
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
    pub async fn get_group_messages(&self, group_id: u32, offset: u32) -> Result<Vec<Message>> {
        let limit = 30;
        let offset_rows = offset * limit;

        // 使用 query_as! 时，需要把表里的 `timestamp` 原样返回
        let messages = sqlx::query_as!(
            Message,
            r#"
            SELECT
                sender_id as `sender_id!`,
                `timestamp` as `timestamp!: NaiveDateTime`,
                message     as `message!`
            FROM ugroup_messages
            WHERE group_id = ?
            ORDER BY `timestamp` DESC
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
}
