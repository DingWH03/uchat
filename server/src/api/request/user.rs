// api/request/user
use axum::extract::ws::Message;
use log::{info, warn};
use uuid::Uuid;
use crate::{api::error::UserError, protocol::{PatchUserRequest, ServerMessage, UpdateUserRequest}};
use super::Request;

impl Request {
    /// 处理用户通过http请求登录
    /// 返回 'Ok(Some(session_cookie))' 如果登陆成功
    /// 返回 'Ok(None)' 如果用户不存在或密码错误
    /// 可以重复登陆，会分发不同的cookie
    pub async fn login(&mut self, id: u32, password: &str) -> Result<String, UserError> {
        let password_hash = self.db.get_password_hash(id).await?;
        let password_hash = password_hash.ok_or(UserError::UserNotFound)?;

        let valid = bcrypt::verify(password, &password_hash)?;
        if !valid {
            return Err(UserError::InvalidPassword);
        }

        let session_cookie = Uuid::now_v7().to_string();

        // 检查是否是首次登录（无任何活跃 session）
        let is_first_login = {
            let sessions_read_guard = self.sessions.read().await;
            sessions_read_guard
                .get_sessions_by_user(id)
                .map_or(true, |set| set.is_empty())
        };

        // 插入会话
        self.sessions
            .write()
            .await
            .insert_session(id, session_cookie.clone(), None);

        info!("用户 {} 登录成功", id);

        // 仅首次登录广播上线消息
        if is_first_login {
            let online_friends = self.get_online_friends(id).await.unwrap_or_default();
            let server_message = ServerMessage::OnlineMessage { friend_id: id };
            let json = serde_json::to_string(&server_message)?;

            for friend in online_friends {
                self.send_to_user(
                    friend.user_id,
                    Message::Text(axum::extract::ws::Utf8Bytes::from(&json)),
                )
                .await;
            }
        }

        Ok(session_cookie)
    }
    /// 退出该会话
    pub async fn logout(&self, session_id: &str) -> Result<(), anyhow::Error> {
        let mut sessions_write_guard = self.sessions.write().await;

        // 获取当前用户 ID（用于广播）
        if let Some(user_id) = sessions_write_guard.get_user_id_by_session(session_id) {
            // 删除会话
            sessions_write_guard.delete_session(session_id);

            // 判断是否是该用户的最后一个 session
            let still_online = sessions_write_guard
                .get_sessions_by_user(user_id)
                .map_or(false, |s| !s.is_empty());

            if !still_online {
                // 如果该用户彻底下线，则广播 OfflineMessage
                drop(sessions_write_guard); // 提前释放锁，避免死锁

                let online_friends = self.get_online_friends(user_id).await.unwrap_or_default();
                let server_message = ServerMessage::OfflineMessage { friend_id: user_id };
                let json = serde_json::to_string(&server_message)?;
                for friend in online_friends {
                    self.send_to_user(
                        friend.user_id,
                        Message::Text(axum::extract::ws::Utf8Bytes::from(&json)),
                    )
                    .await;
                }
            }

            info!("会话 {} 已注销", session_id);
            Ok(())
        } else {
            warn!("会话 {} 不存在或已过期", session_id);
            Err(UserError::SessionNotFound.into())
        }
    }
    /// 删除用户，注销账号
    pub async fn delete_user(&self, id: u32) -> Result<(), sqlx::Error> {
        self.db.delete_user(id).await
    }
    /// 更新用户信息
    pub async fn update_user_info_full(
        &self, 
        id: u32,
        update: UpdateUserRequest,
    ) -> Result<(), sqlx::Error> {
        self.db.update_user_info_full(id, update).await
    }
    /// 更新用户部分信息
    pub async fn update_user_info_partial(
        &self, 
        id: u32,
        update: PatchUserRequest,
    ) -> Result<(), sqlx::Error> {
        self.db.update_user_info_partial(id, update).await
    }
}