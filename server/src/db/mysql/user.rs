use async_trait::async_trait;
use super::MysqlDB;
use anyhow::Result;
use crate::{db::UserDB, protocol::{PatchUserRequest, UpdateUserRequest, UserDetailedInfo}};
use sqlx::Arguments;

#[async_trait]
impl UserDB for MysqlDB {
    /// 设置UserDetailedInfo用户信息，当前用户信息较少，以后会考虑单独设置某一部分，例如个性签名，头像等
    // async fn set_userinfo(&self, id: u32, userinfo: UserDetailedInfo) -> Result<()> {

    // }
    /// 查询用户密码哈希
    async fn get_password_hash(&self, id: u32) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.password_hash))
    }

    /// 更新用户密码
    async fn update_password(
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
    async fn new_user(&self, username: &str, password_hash: &str) -> Result<Option<u32>> {
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
    async fn delete_user(&self, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 完整更新用户信息
    async fn update_user_info_full(
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
    async fn update_user_info_partial(
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
    async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>> {
        let row = sqlx::query!("SELECT id AS user_id, username FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| UserDetailedInfo {
            user_id: r.user_id,
            username: r.username,
        }))
    }
}