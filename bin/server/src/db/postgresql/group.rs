use crate::db::GroupDB;
use crate::db::error::DBError;
use crate::db::postgresql::PgSqlDB;
use crate::protocol::{GroupDetailedInfo, GroupSimpleInfo, UserSimpleInfo};

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
impl GroupDB for PgSqlDB {
    /// 根据group_id获取群聊详细信息
    async fn get_groupinfo(&self, group_id: u32) -> Result<Option<GroupDetailedInfo>, DBError> {
        let row = sqlx::query!(
            "SELECT id AS group_id, name AS title FROM ugroups WHERE id = $1",
            group_id as i32
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| GroupDetailedInfo {
            group_id: r.group_id as u32,
            title: r.title,
        }))
    }

    /// 根据user_id🔍群组列表，一般是自己查找自己的群组列表
    async fn get_groups(&self, user_id: u32) -> Result<Vec<GroupSimpleInfo>, DBError> {
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
                gm.user_id = $1
            ",
            user_id as i32
        )
        .fetch_all(&self.pool)
        .await?;

        // 将查询结果映射到 GroupSimpleInfo 结构体
        Ok(rows
            .into_iter()
            .map(|r| GroupSimpleInfo {
                group_id: r.group_id as u32,
                title: r.title,
            })
            .collect())
    }

    /// 根据group_id🔍群组成员列表
    async fn get_group_members(&self, group_id: u32) -> Result<Vec<UserSimpleInfo>, DBError> {
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
                gm.group_id = $1
            ",
            group_id as i32
        )
        .fetch_all(&self.pool)
        .await?;

        // 将查询结果映射到 GroupMemberInfo 结构体
        Ok(rows
            .into_iter()
            .map(|r| UserSimpleInfo {
                user_id: r.user_id as u32,
                username: r.username,
            })
            .collect())
    }

    async fn create_group(
        &self,
        user_id: u32,
        group_name: &str,
        members: Vec<u32>,
    ) -> Result<u32, DBError> {
        // 创建群组
        let rec = sqlx::query!(
            "INSERT INTO ugroups (name, creator_id) VALUES ($1, $2) RETURNING id",
            group_name,
            user_id as i32
        )
        .fetch_one(&self.pool)
        .await?;

        let group_id = rec.id as u32;

        // 插入创建者
        sqlx::query!(
            "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2)",
            group_id as i32,
            user_id as i32
        )
        .execute(&self.pool)
        .await?;

        // 排除创建者
        let members_to_add: Vec<u32> = members.into_iter().filter(|&id| id != user_id).collect();

        if !members_to_add.is_empty() {
            let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
                "INSERT INTO group_members (group_id, user_id)",
            );

            builder.push_values(members_to_add.iter(), |mut b, member_id| {
                b.push_bind(group_id as i32).push_bind(*member_id as i32);
            });

            builder.build().execute(&self.pool).await?;
        }

        Ok(group_id)
    }

    /// 添加群组成员，user_id是发送者的id，group_id是接收者的id
    async fn join_group(&self, user_id: u32, group_id: u32) -> Result<(), DBError> {
        sqlx::query!(
            "INSERT INTO group_members (user_id, group_id) VALUES ($1, $2)",
            user_id as i32,
            group_id as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 退出群聊
    async fn leave_group(&self, user_id: u32, group_id: u32) -> Result<(), DBError> {
        sqlx::query!(
            "DELETE FROM group_members WHERE user_id = $1 AND group_id = $2",
            user_id as i32,
            group_id as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
