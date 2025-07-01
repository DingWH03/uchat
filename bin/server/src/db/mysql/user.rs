use async_trait::async_trait;
use super::MysqlDB;
use anyhow::Result;
use uchat_protocol::{RoleType, UpdateTimestamps, request::{PatchUserRequest, UpdateUserRequest}, UserDetailedInfo};
use crate::db::{error::DBError, UserDB};
use sqlx::Arguments;

#[async_trait]
impl UserDB for MysqlDB {
    /// 设置UserDetailedInfo用户信息，当前用户信息较少，以后会考虑单独设置某一部分，例如个性签名，头像等
    // async fn set_userinfo(&self, id: u32, userinfo: UserDetailedInfo) -> Result<()> {

    // }
    /// 查询用户密码和身份
    async fn get_user_password_and_role(&self, user_id: u32) -> Result<(String, RoleType), DBError> {
        let row = sqlx::query!(
            r#"
            SELECT password_hash, role as `role: RoleType`
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let password = row.password_hash;
            Ok((password, row.role))
        } else {
            Err(DBError::NotFound)
        }
    }

    /// 查询用户密码哈希
    async fn get_password_hash(&self, id: u32) -> Result<String, DBError> {
        let row = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let password = row.password_hash;
            Ok(password)
        }
        else {
            Err(DBError::NotFound)
        }
    }

    /// 更新用户密码
    async fn update_password(
        &self,
        id: u32,
        new_password_hash: &str,
    ) -> Result<(), DBError> {
        sqlx::query!(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            new_password_hash,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取用户的好友和群组更新时间（返回时间戳，单位：秒）
    async fn get_update_timestamps(&self, id: u32) -> Result<UpdateTimestamps, DBError> {
        let row = sqlx::query!(
            "SELECT friends_updated_at, groups_updated_at FROM users WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let friends_ts = row.friends_updated_at.ok_or(DBError::NotFound)?.and_utc().timestamp();
            let groups_ts = row.groups_updated_at.ok_or(DBError::NotFound)?.and_utc().timestamp();
            Ok(UpdateTimestamps { friends_updated_at: friends_ts, groups_updated_at: groups_ts })
        } else {
            Err(DBError::NotFound)
        }
    }


    /// 创建新用户
    async fn new_user(&self, username: &str, password_hash: &str) -> Result<u32, DBError> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            username,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        // 获取插入的自增ID
        let last_insert_id = result.last_insert_id() as u32;

        Ok(last_insert_id)
    }

    /// 删除用户
    async fn delete_user(&self, id: u32) -> Result<(), DBError> {
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
    ) -> Result<(), DBError> {
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
    ) -> Result<(), DBError> {
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

    /// 更新用户头像
    async fn update_user_avatar(
        &self,
        id: u32,
        avatar_url: &str,
    ) -> Result<(), DBError> {
        sqlx::query!(
            "UPDATE users SET avatar_url = ? WHERE id = ?",
            avatar_url,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 根据id查找用户详细信息
    async fn get_userinfo(&self, id: u32) -> Result<Option<UserDetailedInfo>, DBError> {
        let row = sqlx::query_as!(
            UserDetailedInfo,
            r#"
            SELECT id as user_id, username, role as "role: RoleType", avatar_url
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

}