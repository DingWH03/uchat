// src/db/mod.rs
mod messages;

use crate::protocol::{
    GroupDetailedInfo, GroupSimpleInfo, MessageType, PatchUserRequest, UpdateUserRequest, UserDetailedInfo, UserSimpleInfo,
};
use anyhow::Result;

use sqlx::Arguments;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::env;

/// 结构体 `Database` 用于封装 MySQL 连接池
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// 异步初始化数据库连接池
    pub async fn new() -> Result<Self> {
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
    pub async fn get_password_hash(&self, id: u32) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.password_hash))
    }

    /// 更新用户密码
    pub async fn update_password(
        &self,
        id: u32,
        new_password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            new_password_hash,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
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

    /// 删除用户
    pub async fn delete_user(&self, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 完整更新用户信息
    pub async fn update_user_info_full(
        &self,
        id: u32,
        update: UpdateUserRequest,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET username = ? WHERE id = ?",
            update.username,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 部分更新用户信息
    pub async fn update_user_info_partial(
        &self,
        id: u32,
        patch: PatchUserRequest,
    ) -> Result<(), sqlx::Error> {
        let mut sql = String::from("UPDATE users SET ");
        let mut sets = Vec::new();
        let mut args = sqlx::mysql::MySqlArguments::default();

        if let Some(username) = patch.username {
            sets.push("username = ?");
            let _ = args.add(username);
        }

        if sets.is_empty() {
            // 没有要更新的字段
            return Ok(());
        }

        sql.push_str(&sets.join(", "));
        sql.push_str(" WHERE id = ?");
        let _ = args.add(id);

        sqlx::query_with(&sql, args).execute(&self.pool).await?;

        Ok(())
    }

    /// 根据id查找用户详细信息
    pub async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>> {
        let row = sqlx::query!("SELECT id AS user_id, username FROM users WHERE id = ?", id)
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
    pub async fn get_friends(&self, user_id: u32) -> Result<Vec<UserSimpleInfo>, sqlx::Error> {
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
        Ok(rows
            .into_iter()
            .map(|r| UserSimpleInfo {
                user_id: r.friend_id,
                username: r.username,
            })
            .collect())
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
        Ok(rows
            .into_iter()
            .map(|r| GroupSimpleInfo {
                group_id: r.group_id,
                title: r.title,
            })
            .collect())
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
        Ok(rows
            .into_iter()
            .map(|r| UserSimpleInfo {
                user_id: r.user_id,
                username: r.username,
            })
            .collect())
    }

    pub async fn create_group(
        &self,
        user_id: u32,
        group_name: &str,
        members: Vec<u32>,
    ) -> Result<u32> {
        // 创建群组
        let result = sqlx::query!(
            "INSERT INTO ugroups (name, creator_id) VALUES (?, ?)",
            group_name,
            user_id
        )
        .execute(&self.pool)
        .await?;

        // 下面的用法可以区分插入失败还是数据表错误
        // let result = match sqlx::query!(
        //     "INSERT INTO ugroups (name, creator_id) VALUES (?, ?)",
        //     group_name,
        //     user_id
        // )
        // .execute(&self.pool)
        // .await
        // {
        //     Ok(res) => res,
        //     Err(_) => return Ok(None),
        // };

        let group_id = result.last_insert_id() as u32;

        // 插入创建者
        sqlx::query!(
            "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
            group_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        // 排除创建者
        let members_to_add: Vec<u32> = members.into_iter().filter(|&id| id != user_id).collect();

        if !members_to_add.is_empty() {
            let mut builder =
                sqlx::QueryBuilder::new("INSERT INTO group_members (group_id, user_id)");

            builder.push_values(members_to_add.iter(), |mut b, member_id| {
                b.push_bind(group_id).push_bind(*member_id);
            });

            builder.build().execute(&self.pool).await?;
        }

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
    pub async fn join_group(&self, user_id: u32, group_id: u32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO group_members (user_id, group_id) VALUES (?, ?)",
            user_id,
            group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 退出群聊
    pub async fn leave_group(&self, user_id: u32, group_id: u32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM group_members WHERE user_id = ? AND group_id = ?",
            user_id,
            group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 添加私聊信息聊天记录，返回消息的自增 ID
    pub async fn add_message(
        &self,
        sender: u32,
        receiver: u32,
        message_type: MessageType,
        message: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO messages (sender_id, receiver_id, message_type, message)
            VALUES (?, ?, ?, ?)
            "#,
            sender,
            receiver,
            message_type as MessageType,
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

}
