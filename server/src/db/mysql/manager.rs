use std::collections::HashMap;

use crate::{db::{error::DBError, ManagerDB, MessageDB}, protocol::{GroupSessionMessage, MessageType, RoleType, SessionMessage}};
use super::MysqlDB;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDateTime;

#[async_trait]
impl ManagerDB for MysqlDB{
    async fn get_user_count(&self) -> Result<u32, DBError> {
        let row = sqlx::query!(
            "SELECT COUNT(id) as count FROM users"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count as u32)
    }
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