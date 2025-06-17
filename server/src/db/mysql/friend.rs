use crate::db::error::DBError;
use crate::db::FriendDB;
use crate::db::mysql::MysqlDB;
use crate::protocol::UserSimpleInfo;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
impl FriendDB for MysqlDB {
    /// 根据user_id🔍好友列表，一般是自己查找自己的好友列表
    async fn get_friends(&self, user_id: u32) -> Result<Vec<UserSimpleInfo>, DBError> {
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
    /// 添加好友，user_id是发送者的id，friend_id是接收者的id
    /// 直接双向成为好友，暂不支持请求与同意机制
    async fn add_friend(&self, user_id: u32, friend_id: u32) -> Result<(), DBError> {
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
}
