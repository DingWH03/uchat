use uchat_model::{request::RequestResponse, UserSimpleInfo, UserSimpleInfoWithStatus, UserStatus};
use log::error;
use crate::api::error::RequestError;

use super::Request;

impl Request {
    /// 获取该用户所有在线好友的信息
    pub async fn get_online_friends(
        &self,
        user_id: u32,
    ) -> Result<Vec<UserSimpleInfo>, RequestError> {
        // 获取所有好友（从数据库）
        let all_friends = self.db.get_friends(user_id).await?;
        // 过滤在线好友（根据 sessions 判断）
        let mut online_friends = Vec::new();
        for friend in all_friends {
            let sessions = self.sessions.get_sessions_by_user(friend.user_id).await;
            if sessions.is_some_and(|sessions| !sessions.is_empty()) {
                online_friends.push(friend);
            }
        }

        Ok(online_friends)
    }

    /// 返回一个用户的好友列表
    pub async fn get_friends(&self, id: u32) -> RequestResponse<Vec<UserSimpleInfo>> {
        match self.db.get_friends(id).await {
            Ok(list) => RequestResponse::ok("获取成功", list),
            Err(e) => {
                error!("数据库获取好友列表失败: {}", e);
                RequestResponse::err(format!("服务器错误：{}", e))
            }
        }
    }
    /// 返回一个带有在线信息的好友列表
    pub async fn get_friends_with_status(
        &self,
        id: u32,
    ) -> RequestResponse<Vec<UserSimpleInfoWithStatus>> {
        let friends_resp = self.get_friends(id).await;
        if !friends_resp.status {
            return RequestResponse::err(friends_resp.message);
        }

        // 安全地解包数据
        let friends = match friends_resp.data {
            Some(friends) => friends,
            None => return RequestResponse::ok("获取成功", Vec::new()),
        };

        let session_manager = self.sessions.clone();
        let futures = friends.into_iter().map(|friend| {
            let session_manager = session_manager.clone();
            async move {
                let online = session_manager
                    .get_sessions_by_user(friend.user_id)
                    .await
                    .is_some_and(|sessions| !sessions.is_empty());

                UserSimpleInfoWithStatus {
                    base: friend,
                    online,
                }
            }
        });

        let result = futures::future::join_all(futures).await;

        RequestResponse::ok("获取成功", result)
    }

    /// 批量查询用户在线状态，返回 Vec<UserStatus>
    pub async fn get_status_by_userids(
        &self,
        user_ids: &[u32],
    ) -> RequestResponse<Vec<UserStatus>> {
        let session_manager = self.sessions.clone();

        // 生成异步任务，查询每个 user_id 是否在线，返回 UserStatus 结构体
        let futures = user_ids.iter().map(|&user_id| {
            let session_manager = session_manager.clone();
            async move {
                let online = session_manager
                    .get_sessions_by_user(user_id)
                    .await
                    .is_some_and(|sessions| !sessions.is_empty());
                UserStatus { user_id, online }
            }
        });

        let result = futures::future::join_all(futures).await;

        RequestResponse::ok("获取成功", result)
    }
    /// 通过user_id添加好友
    /// 当前版本无需确认直接通过
    pub async fn add_friend(&self, user_id: u32, friend_id: u32) -> RequestResponse<()> {
        match self.db.add_friend(user_id, friend_id).await {
            Ok(_) => {
                self.cache.invalidate_friends(user_id).await;
                RequestResponse::ok("添加成功", ())},
            Err(e) => {
                error!("数据库错误：{}", e);
                RequestResponse::bad_request("该用户不存在")
            }
        }
    }

}