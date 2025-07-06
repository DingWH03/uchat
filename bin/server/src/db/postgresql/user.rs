use super::PgSqlDB;
use crate::{
    db::{UserDB, error::DBError},
    protocol::{PatchUserRequest, UpdateUserRequest, UserDetailedInfo},
};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Arguments;

#[async_trait]
impl UserDB for PgSqlDB {
    /// 设置UserDetailedInfo用户信息，当前用户信息较少，以后会考虑单独设置某一部分，例如个性签名，头像等
    // async fn set_userinfo(&self, id: u32, userinfo: UserDetailedInfo) -> Result<()> {

    // }
    /// 查询用户密码哈希
    async fn get_password_hash(&self, id: u32) -> Result<Option<String>, DBError> {
        let row = sqlx::query!("SELECT password_hash FROM users WHERE id = $1", id as i32)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.password_hash))
    }

    /// 更新用户密码
    async fn update_password(&self, id: u32, new_password_hash: &str) -> Result<(), DBError> {
        sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            new_password_hash,
            id as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 创建新用户
    async fn new_user(&self, username: &str, password_hash: &str) -> Result<u32, DBError> {
        let row = sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id",
            username,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;

        // 获取插入的自增ID
        let last_insert_id = row.id as u32;

        Ok(last_insert_id)
    }

    /// 删除用户
    async fn delete_user(&self, id: u32) -> Result<(), DBError> {
        sqlx::query!("DELETE FROM users WHERE id = $1", id as i32)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 完整更新用户信息
    async fn update_user_info_full(
        &self,
        id: u32,
        update: UpdateUserRequest,
    ) -> Result<(), DBError> {
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = $2",
            update.username,
            id as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 部分更新用户信息
    async fn update_user_info_partial(
        &self,
        id: u32,
        patch: PatchUserRequest,
    ) -> Result<(), DBError> {
        let mut sql = String::from("UPDATE users SET ");
        let mut sets = Vec::new();
        let mut args = sqlx::postgres::PgArguments::default();
        let mut param_index = 1;

        if let Some(username) = patch.username {
            sets.push(format!("username = ${}", param_index));
            args.add(username);
            param_index += 1;
        }

        if sets.is_empty() {
            // 没有要更新的字段
            return Ok(());
        }

        sql.push_str(&sets.join(", "));
        sql.push_str(&format!(" WHERE id = ${}", param_index));
        args.add(id as i32);

        sqlx::query_with(&sql, args).execute(&self.pool).await?;

        Ok(())
    }

    /// 根据id查找用户详细信息
    async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>, DBError> {
        let row = sqlx::query!(
            "SELECT id AS user_id, username FROM users WHERE id = $1",
            id as i32
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserDetailedInfo {
            user_id: r.user_id as u32,
            username: r.username,
        }))
    }
}
