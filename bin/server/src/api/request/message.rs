use std::sync::Arc;

use axum::extract::ws::Message;
use futures::{stream::FuturesUnordered, StreamExt};
use log::{debug, error, warn};
use uchat_model::{message::ServerMessage, MessageType};

use super::Request;


impl Request {
    /// 根据session_id发送消息
    pub async fn send_to_session(&self, session_id: &str, msg: Message) {
        self.sessions.send_to_session(session_id, msg).await
    }

    /// 发送给用户的所有 WebSocket 连接
    pub async fn send_to_user(&self, user_id: u32, msg: Message) {
        self.sessions.send_to_user(user_id, msg).await
    }

    /// 发送给用户所有的 WebSocket 连接（v2版）
    /// 发送给用户所有的 WebSocket 连接（v2版，支持二进制消息以提升效率）
    pub async fn send_to_user_v2(&self, sender_session_id: &str, receiver_id: u32, msg: &str) {
        let Some(sender_id) = self.check_session(sender_session_id).await else {
            warn!(
                "未能获取会话 {} 对应的用户ID，放弃处理此条消息",
                sender_session_id
            );
            return;
        };
        // 存储到数据库中
        match self
            .db
            .add_message(sender_id, receiver_id, MessageType::Text, msg)
            .await
        {
            // 新增了消息类型枚举，先在这挖一个坑
            Ok((timestamp, message_id)) => {
                debug!(
                    "用户 {} 发送私聊消息给用户 {} 成功，消息message_id: {}, timestamp: {}",
                    sender_id, receiver_id, message_id, timestamp
                );
                let server_message = ServerMessage::SendMessage {
                    message_id,
                    sender: sender_id,
                    receiver: receiver_id,
                    message: msg.to_string(),
                    timestamp,
                };
                // // 使用二进制序列化（如 serde_json），比 JSON 文本更高效
                // let bin = match serde_json::to_vec(&server_message) {
                //     Ok(data) => data,
                //     Err(e) => {
                //         error!("序列化消息为二进制失败: {:?}", e);
                //         return;
                //     }
                // };
                // self
                //     .send_to_user(
                //         receiver_id,
                //         Message::Binary(bin.into()),
                //     )
                //     .await;
                // let json = match serde_json::to_string(&server_message) {
                //     Ok(data) => data,
                //     Err(e) => {
                //         error!("序列化消息为JSON失败: {:?}", e);
                //         return;
                //     }
                // };
                // 暂时序列化为text消息
                let msg = Message::Binary(server_message.to_bytes().into());
                // 发送给接受用户所有的在线会话
                self.send_to_user(receiver_id, msg.clone()).await;
                // 发送给发送用户所有的在线会话，也便于多会话登陆消息同步
                self.send_to_user(sender_id, msg).await;
            }
            Err(e) => {
                error!(
                    "用户 {} 发送私聊消息给用户 {} 失败: {:?}",
                    sender_id, receiver_id, e
                ); // 如果数据库操作失败，直接返回
            }
        }
    }

    /// 根据群号发送群消息
    /// 如果群组不存在或发送失败，返回 false
    /// 先读取群聊成员列表，然后发送消息给每个成员
    pub async fn send_to_group(&self, group_id: u32, msg: Message) {
        // 1. 先查cache
        let member_ids = self.get_group_member_ids(group_id).await;
        if member_ids.is_empty() {
            warn!("群组 {} 成员列表为空，无法发送消息", group_id);
            return;
        }

        let sessions = self.sessions.clone();
        let msg = Arc::new(msg); // 共享消息，避免多次 clone
        let mut tasks = FuturesUnordered::new();

        for member_id in member_ids {
            let msg = Arc::clone(&msg); // 引用共享消息
            let sessions = Arc::clone(&sessions);
            tasks.push(tokio::spawn(async move {
                sessions.send_to_user(member_id as u32, (*msg).clone()).await;
            }));
        }

        // 等待所有发送任务完成
        while let Some(res) = tasks.next().await {
            if let Err(e) = res {
                error!("发送群聊消息部分或全部任务失败: {:?}", e);
            }
        }
    }

    /// 根据群号发送群消息
    /// 如果群组不存在或发送失败，返回 false
    /// 先读取群聊成员列表，然后发送消息给每个成员
    pub async fn send_to_group_v2(&self, sender_session_id: &str, group_id: u32, msg: &str) {
        let Some(sender_id) = self.check_session(sender_session_id).await else {
            warn!(
                "未能获取会话 {} 对应的用户ID，放弃处理此条消息",
                sender_session_id
            );
            return;
        };
        // 存储到数据库中
        match self.db.add_group_message(group_id, sender_id, msg).await {
            Ok((timestamp, message_id)) => {
                debug!(
                    "用户 {} 发送群消息给 {} 成功，消息message_id: {}, timestamp: {}",
                    sender_id, group_id, message_id, timestamp
                );
                let server_message = ServerMessage::SendGroupMessage {
                    message_id,
                    sender: sender_id,
                    group_id,
                    message: msg.to_string(),
                    timestamp,
                };
                // // 使用二进制序列化（如 serde_json），比 JSON 文本更高效
                // let bin = match serde_json::to_vec(&server_message) {
                //     Ok(data) => data,
                //     Err(e) => {
                //         error!("序列化消息为二进制失败: {:?}", e);
                //         return;
                //     }
                // };
                // self.send_to_group(group_id, Message::Binary(bin.into()))
                //     .await;
                // let json = match serde_json::to_string(&server_message) {
                //     Ok(data) => data,
                //     Err(e) => {
                //         error!("序列化消息为JSON失败: {:?}", e);
                //         return;
                //     }
                // };
                // let json =
                //     serde_json::to_string(&server_message).unwrap_or_else(|_| String::from("{}"));
                self.send_to_group(
                    group_id,
                    Message::Binary(server_message.to_bytes().into()),
                )
                .await;
            }
            Err(e) => {
                error!("用户 {} 发送群消息给 {} 失败: {:?}", sender_id, group_id, e); // 如果数据库操作失败，直接返回
            }
        }
    }
}