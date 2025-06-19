use log::{error, info};

use crate::{protocol::{ManagerResponse, RecentPrivateMessage}};

use super::Manager;

impl Manager {
    /// 获取服务器最近的messages
    pub async fn get_recent_messages(
        &self,
        count: u32,
        offset: u32,
    ) -> ManagerResponse<Vec<RecentPrivateMessage>> {
        info!(
            "响应manager获取最近message: count: {}, offset: {}",
            count, offset
        );
        let result = self.db.get_recent_messages(count, offset).await;
        match result {
            Ok(data) => ManagerResponse::ok("获取成功", data),
            Err(e) => {
                error!("获取近期聊天记录失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 获取某用户最近的messages
    pub async fn get_user_recent_messages(
        &self,
        count: u32,
        offset: u32,
        user_id: u32,
    ) -> ManagerResponse<Vec<RecentPrivateMessage>> {
        info!(
            "响应manager获取用户{}最近message: count: {}, offset: {}",
            user_id, count, offset
        );
        let result = self.db.get_user_recent_messages(count, offset, user_id).await;
        match result {
            Ok(data) => ManagerResponse::ok("获取成功", data),
            Err(e) => {
                error!("获取近期聊天记录失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 根据message id删除聊天记录
    pub async fn delete_message(
        &self,
        message_id: u64,
    ) -> ManagerResponse<u64> {
        info!(
            "响应manager删除message: {}",
            message_id
        );
        let result = self.db.delete_private_message(message_id).await;
        match result {
            Ok(index) => ManagerResponse::ok("获取成功", index),
            Err(e) => {
                error!("删除失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
}
