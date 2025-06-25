// api/request/user
use super::Request;
use crate::{
    db::error::DBError,
    protocol::{
        message::ServerMessage, request::{PatchUserRequest, RequestResponse, UpdateUserRequest}, UpdateTimestamps, UserDetailedInfo
    },
};
use axum::extract::ws::Message;
use log::{error, info, warn};
use uuid::Uuid;

impl Request {
    /// 处理用户通过http请求登录
    /// 返回 'Ok(Some(session_cookie))' 如果登陆成功
    /// 返回 'Ok(None)' 如果用户不存在或密码错误
    /// 可以重复登陆，会分发不同的cookie
    pub async fn login(&mut self, id: u32, password: &str) -> RequestResponse<String> {
        let (password_hash, role) = match self.db.get_user_password_and_role(id).await {
            Ok(tuple) => tuple,
            Err(e) => {
                // 区分用户不存在和数据库错误
                match e {
                    DBError::NotFound => return RequestResponse::not_found(),
                    _ => return RequestResponse::err(format!("数据库错误：{}", e)),
                }
            }
        };

        let valid = match bcrypt::verify(password, &password_hash) {
            Ok(valid) => valid,
            Err(e) => {
                error!("密码校验失败: {}", e);
                return RequestResponse::err("密码校验失败".to_string());
            }
        };
        if !valid {
            return RequestResponse::unauthorized();
        }

        let session_cookie = Uuid::now_v7().to_string();
        // 检查是否是首次登录（无任何活跃 session）
        let is_first_login = {
            self.sessions
                .get_sessions_by_user(id)
                .await
                .map_or(true, |set| set.is_empty())
        };
        // 插入会话
        self.sessions
            .insert_session(id, session_cookie.clone(), None, role).await;

        info!("用户 {} 登录成功", id);

        // 仅首次登录广播上线消息
        if is_first_login {
            let online_friends = self.get_online_friends(id).await.unwrap_or_default();
            let server_message = ServerMessage::OnlineMessage { friend_id: id };
            let json = match serde_json::to_string(&server_message) {
                Ok(j) => j,
                Err(e) => {
                    error!("序列化上线消息失败: {}", e);
                    return RequestResponse::err(format!("序列化上线消息失败: {}", e));
                }
            };

            for friend in online_friends {
                self.send_to_user(
                    friend.user_id,
                    Message::Text(axum::extract::ws::Utf8Bytes::from(&json)),
                )
                .await;
            }
        }

        RequestResponse::ok("登陆成功", session_cookie)
    }
    /// 退出该会话
    pub async fn logout(&self, session_id: &str) -> RequestResponse<()> {

        // 获取当前用户 ID（用于广播）
        if let Some(user_id) = self.sessions.get_user_id_by_session(session_id).await {
            // 删除会话
            self.sessions.delete_session(session_id).await;

            // 判断是否是该用户的最后一个 session
            let still_online = self.sessions
                .get_sessions_by_user(user_id)
                .await
                .map_or(false, |s| !s.is_empty());

            if !still_online {
                // 如果该用户彻底下线，则广播 OfflineMessage

                let online_friends = self.get_online_friends(user_id).await.unwrap_or_default();
                let server_message = ServerMessage::OfflineMessage { friend_id: user_id };
                let json = match serde_json::to_string(&server_message) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("序列化上线消息失败: {}", e);
                        return RequestResponse::err(format!("序列化上线消息失败: {}", e));
                    }
                };
                for friend in online_friends {
                    self.send_to_user(
                        friend.user_id,
                        Message::Text(axum::extract::ws::Utf8Bytes::from(&json)),
                    )
                    .await;
                }
            }

            info!("会话 {} 已注销", session_id);
            RequestResponse::ok("注销成功", ())
        } else {
            warn!("会话 {} 不存在或已过期", session_id);
            RequestResponse::bad_request("会话不存在")
        }
    }

    /// 返回用户的详细信息
    pub async fn get_userinfo(&self, id: u32) -> RequestResponse<UserDetailedInfo> {
        match self.db.get_userinfo(id).await {
            Ok(Some(info)) => RequestResponse::ok("获取成功", info),
            Ok(None) => {
                warn!("数据库中无用户: {}的信息", id);
                RequestResponse::not_found()
            }
            Err(e) => {
                error!("获取用户的详细信息失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 删除用户，注销账号
    pub async fn delete_user(&self, id: u32) -> RequestResponse<()> {
        match self.db.delete_user(id).await {
            Ok(()) => RequestResponse::ok("注销成功", ()),
            Err(e) => {
                error!("注销用户账号失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 查询用户的好友和群组更新时间戳(单位：秒)
    pub async fn get_update_timestamps(&self, id: u32) -> RequestResponse<UpdateTimestamps> {
        match self.db.get_update_timestamps(id).await {
            Ok(timestamps) => RequestResponse::ok("获取成功", timestamps),
            Err(e) => {
                error!("获取用户更新时间戳失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 更新用户信息
    pub async fn update_user_info_full(
        &self,
        id: u32,
        update: UpdateUserRequest,
    ) -> RequestResponse<()> {
        match self.db.update_user_info_full(id, update).await {
            Ok(_) => RequestResponse::ok("更新成功", ()),
            Err(e) => {
                error!("更新信息失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 更新用户部分信息
    pub async fn update_user_info_partial(
        &self,
        id: u32,
        update: PatchUserRequest,
    ) -> RequestResponse<()> {
        match self.db.update_user_info_partial(id, update).await {
            Ok(_) => RequestResponse::ok("更新成功", ()),
            Err(e) => {
                error!("更新信息失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }

    /// 更新用户头像
    pub async fn update_avatar(
        &self,
        user_id: u32,
        file_bytes: &[u8],
        file_name: &str,
        content_type: &str,
    ) -> RequestResponse<String> {
        // 构建路径
        let timestamp = chrono::Utc::now().timestamp();
        let new_file_name = format!("{}_{}", timestamp, file_name);

        let object_dir = format!("avatars/{}/", user_id);
        let object_path = format!("{}{}", object_dir, new_file_name);

        // 上传到 MinIO（或其他实现）
        let result = self
            .storage
            .upload(&object_path, file_bytes, content_type)
            .await;

        let url = match result {
            Ok(url) => url,
            Err(e) => {
                error!("头像上传失败: {}", e);
                return RequestResponse::err("头像上传失败");
            }
        };

        // 删除该用户头像文件夹下除当前头像之外的其他文件
        if let Err(e) = self
            .storage
            .delete_prefix_except(&object_dir, &[&new_file_name])
            .await
        {
            error!("删除旧头像失败: {}", e);
        }

        // 更新数据库中头像字段
        let update_result = self
            .db
            .update_user_avatar(user_id, &url)
            .await;

        match update_result {
            Ok(_) => RequestResponse::ok("头像更新成功", url),
            Err(e) => {
                error!("头像URL写入数据库失败: {}", e);
                RequestResponse::err("头像更新失败")
            }
        }
    }
}
