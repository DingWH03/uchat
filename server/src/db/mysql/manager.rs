use std::collections::HashMap;

use crate::{db::{error::DBError, ManagerDB, MessageDB}, protocol::{GroupSessionMessage, MessageType, RoleType, SessionMessage, UserSimpleInfo}};
use super::MysqlDB;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

#[async_trait]
impl ManagerDB for MysqlDB{
    /// 用户总数量
    async fn get_user_count(&self) -> Result<u32, DBError> {
        let row = sqlx::query!(
            "SELECT COUNT(id) as count FROM users"
        )
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
}