mod messages;
mod user;

use crate::api::error::RequestError;
use crate::cache::CacheConfig;
use crate::cache::CacheManagerTrait;
use crate::db::DB;
use crate::db::error::DBError;
use crate::session::SessionConfig;
use crate::session::SessionManagerTrait;
use crate::storage::ObjectStorage;
use axum::extract::ws::Message;
use bcrypt::hash;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use log::{debug, error, info, warn};
use std::sync::Arc;
use uchat_protocol::{
    ContactList, GroupDetailedInfo, GroupSimpleInfo, MessageType, RoleType, UserSimpleInfo,
    UserSimpleInfoWithStatus, UserStatus, message::ServerMessage, request::RequestResponse,
};

pub struct Request {
    db: Arc<dyn DB>,
    sessions: Arc<dyn SessionManagerTrait<Config = SessionConfig>>,
    storage: Arc<dyn ObjectStorage + Send + Sync>,
    cache: Arc<dyn CacheManagerTrait<Config = CacheConfig>>, // 添加缓存管理器
}

impl Request {
    pub fn new(
        db: Arc<dyn DB>,
        sessions: Arc<dyn SessionManagerTrait<Config = SessionConfig>>,
        storage: Arc<dyn ObjectStorage + Send + Sync>,
        cache: Arc<dyn CacheManagerTrait<Config = CacheConfig>>,
    ) -> Self {
        Self {
            db,
            sessions,
            storage,
            cache,
        }
    }

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

    /// 检查session_id是否存在，返回user_id
    pub async fn check_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.check_session(session_id).await
    }

    /// 检查session_id是否存在，返回role
    pub async fn check_session_role(&self, session_id: &str) -> RequestResponse<RoleType> {
        match self.sessions.check_session_role(session_id).await {
            Some(role) => RequestResponse::ok("获取成功", role),
            None => RequestResponse::unauthorized(),
        }
    }

    /// 登陆session sender
    pub async fn register_session(
        &self,
        session_id: &str,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ) {
        self.sessions.register_sender(session_id, sender).await;
    }

    /// 撤销sender
    pub async fn unregister_session(&self, session_id: &str) {
        self.sessions.unregister_sender(session_id).await;
    }

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
        let Some(sender_id) = self.get_user_id_by_session(sender_session_id).await else {
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
                let json = match serde_json::to_string(&server_message) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("序列化消息为JSON失败: {:?}", e);
                        return;
                    }
                };
                // 暂时序列化为text消息
                let msg = Message::Text(axum::extract::ws::Utf8Bytes::from(json));
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
        let member_ids = if let Some(ids) = self.cache.get_group_members(group_id as u64).await {
            debug!(
                "从缓存中获取群组 {} 成员列表: {:?}",
                group_id, ids
            );
            ids
        } else {
            // 2. cache未命中，查数据库并写入cache
            match self.db.get_group_members(group_id).await {
                Err(e) => {
                    error!("获取群组 {} 成员失败: {:?}", group_id, e);
                    return;
                }
                Ok(members) => {
                    let ids: Vec<u64> = members.iter().map(|m| m.user_id as u64).collect();
                    self.cache.set_group_members(group_id as u64, ids.clone()).await;
                    ids
                }
            }
        };

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
        let Some(sender_id) = self.get_user_id_by_session(sender_session_id).await else {
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
                let json = match serde_json::to_string(&server_message) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("序列化消息为JSON失败: {:?}", e);
                        return;
                    }
                };
                // let json =
                //     serde_json::to_string(&server_message).unwrap_or_else(|_| String::from("{}"));
                self.send_to_group(
                    group_id,
                    Message::Text(axum::extract::ws::Utf8Bytes::from(json)),
                )
                .await;
            }
            Err(e) => {
                error!("用户 {} 发送群消息给 {} 失败: {:?}", sender_id, group_id, e); // 如果数据库操作失败，直接返回
            }
        }
    }

    /// 通过session_id查询用户ID
    pub async fn get_user_id_by_session(&self, session_id: &str) -> Option<u32> {
        self.sessions.get_user_id_by_session(session_id).await
    }

    /// 处理用户注册请求
    /// 用户名允许重复，会自动生成唯一的userid
    /// 用户名和密码不可为空
    /// 返回 'Ok(Some(user_id))' 如果注册成功
    pub async fn register(&self, username: &str, password: &str) -> RequestResponse<u32> {
        // 检查用户名和密码是否为空
        if username.is_empty() || password.is_empty() {
            warn!("用户名或密码不能为空");
            return RequestResponse::bad_request("用户名密码不得为空");
        }
        // Hash the password
        let hashed_password = match hash(password, 4) {
            Ok(hashed) => hashed,
            Err(e) => {
                error!("加密密码处理失败！错误: {}", e);
                return RequestResponse::err(format!("服务器错误：{}", e));
            }
        };
        let user_id = self.db.new_user(username, &hashed_password).await;
        match user_id {
            Ok(id) => {
                info!("用户 {} 注册成功", id);
                RequestResponse::ok("注册成功", id)
            }
            Err(e) => {
                error!("数据库用户注册失败: {:?}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
    }

    /// 更改用户的密码(需验证原密码)
    pub async fn change_user_password(
        &self,
        user_id: u32,
        old_password_hashed: &str,
        new_password: &str,
    ) -> RequestResponse<()> {
        // 检查用户名和密码是否为空
        if new_password.is_empty() {
            warn!("密码不能为空");
            return RequestResponse::bad_request("密码不得为空");
        }
        let password_hash = match self.db.get_password_hash(user_id).await {
            Ok(password) => password,
            Err(e) => {
                // 区分用户不存在和数据库错误
                match e {
                    DBError::NotFound => return RequestResponse::not_found(),
                    _ => return RequestResponse::err(format!("数据库错误：{}", e)),
                }
            }
        };

        match bcrypt::verify(old_password_hashed, &password_hash) {
            Ok(true) => {
                let new_hashed_password = match hash(new_password, 4) {
                    Ok(hashed) => hashed,
                    Err(e) => {
                        error!("新密码加密失败: {}", e);
                        return RequestResponse::err(format!("服务器错误：{}", e));
                    }
                };
                match self.db.update_password(user_id, &new_hashed_password).await {
                    Ok(_) => {
                        info!("用户 {} 密码更改成功", user_id);
                        RequestResponse::ok("密码更改成功", ())
                    }
                    Err(e) => {
                        error!("更新密码数据库操作失败: {:?}", e);
                        RequestResponse::err(format!("数据库错误：{}", e))
                    }
                }
            }
            Ok(false) => {
                warn!("用户 {} 原密码不正确", user_id);
                RequestResponse::unauthorized()
            }
            Err(e) => {
                error!("密码验证失败: {}", e);
                RequestResponse::err(format!("服务器错误：{}", e))
            }
        }
    }

    /// 返回群组的详细信息
    pub async fn get_groupinfo(&self, id: u32) -> RequestResponse<GroupDetailedInfo> {
        match self.db.get_groupinfo(id).await {
            Ok(Some(info)) => RequestResponse::ok("获取成功", info),
            Ok(None) => {
                warn!("数据库中无群组: {}的信息", id);
                RequestResponse::not_found()
            }
            Err(e) => {
                error!("获取群组的详细信息失败，检查数据库错误: {}", e);
                RequestResponse::err(format!("数据库错误：{}", e))
            }
        }
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

    /// 批量获取所有的用户和好友列表
    pub async fn get_contact_list(&self, user_id: u32) -> RequestResponse<ContactList> {
        let friends = match self.db.get_friends(user_id).await {
            Ok(friends) => friends,
            Err(e) => {
                error!("获取好友列表失败: {}", e);
                return RequestResponse::err(format!("服务器错误：{}", e));
            }
        };
        let groups = match self.db.get_groups(user_id).await {
            Ok(groups) => groups,
            Err(e) => {
                error!("获取群组列表失败: {}", e);
                return RequestResponse::err(format!("服务器错误：{}", e));
            }
        };
        RequestResponse::ok("获取成功", ContactList { friends, groups })
    }

    /// 获取一个用户的所有群聊
    pub async fn get_groups(&self, id: u32) -> RequestResponse<Vec<GroupSimpleInfo>> {
        match self.db.get_groups(id).await {
            Ok(list) => RequestResponse::ok("获取成功", list),
            Err(e) => {
                error!("数据库获取群聊列表失败: {}", e);
                RequestResponse::err(format!("服务器错误：{}", e))
            }
        }
    }
    /// 获取某个群聊的群聊成员
    pub async fn get_group_members(&self, group_id: u32) -> RequestResponse<Vec<UserSimpleInfo>> {
        match self.db.get_group_members(group_id).await {
            Ok(list) => RequestResponse::ok("获取成功", list),
            Err(e) => {
                error!("数据库获取群聊成员失败: {}", e);
                RequestResponse::err(format!("群组不存在服务器错误：{}", e))
            }
        }
    }
    /// 通过user_id添加好友
    /// 当前版本无需确认直接通过
    pub async fn add_friend(&self, user_id: u32, friend_id: u32) -> RequestResponse<()> {
        match self.db.add_friend(user_id, friend_id).await {
            Ok(_) => RequestResponse::ok("添加成功", ()),
            Err(e) => {
                error!("数据库错误：{}", e);
                RequestResponse::bad_request("该用户不存在")
            }
        }
    }
    /// 创建一个新的群聊，在创建时附带群成员列表
    pub async fn create_group(
        &self,
        user_id: u32,
        group_name: &str,
        members: Vec<u32>,
    ) -> RequestResponse<u32> {
        match self.db.create_group(user_id, group_name, members).await {
            Ok(id) => RequestResponse::ok("创建成功", id),
            Err(e) => {
                error!("数据库错误：{}", e);
                RequestResponse::err(format!("服务器错误：{}", e))
            }
        }
    }
    /// 用户申请加入群聊
    pub async fn join_group(&self, user_id: u32, group_id: u32) -> RequestResponse<()> {
        match self.db.join_group(user_id, group_id).await {
            Ok(_) => RequestResponse::ok("加入成功", ()),
            Err(e) => {
                error!("加入群聊失败：{}", e);
                RequestResponse::bad_request("群聊不存在或服务器错误")
            }
        }
    }
    /// 用户退出群聊
    pub async fn leave_group(&self, user_id: u32, group_id: u32) -> RequestResponse<()> {
        match self.db.leave_group(user_id, group_id).await {
            Ok(_) => RequestResponse::ok("退出成功", ()),
            Err(e) => {
                error!("退出群聊失败：{}", e);
                RequestResponse::bad_request("群聊不存在或服务器错误")
            }
        }
    }

    /// 对ping请求的响应
    pub async fn ping(&self) -> RequestResponse<()> {
        RequestResponse::ok("pong", ())
    }
}
