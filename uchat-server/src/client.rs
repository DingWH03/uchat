// src/client.rs
use crate::api::Api;
use crate::protocol::{ClientRequest, ServerResponse, UserDetailedInfo, GroupDetailedInfo};
use crate::utils::{reader_packet, writer_packet};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Client {
    api: Arc<Mutex<Api>>,
    user_id: Arc<Mutex<u32>>,
    writer: Arc<Mutex<tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>>>,
    reader: Arc<Mutex<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>>,
    signed_in: Arc<AtomicBool>,
}

impl Client {
    pub fn new(
        socket: TcpStream,
        api: Arc<Mutex<Api>>,
        user_id: Arc<Mutex<u32>>,
        signed_in: Arc<AtomicBool>,
    ) -> Self {
        let (reader, writer) = socket.into_split();
        Self {
            api,
            user_id,
            writer: Arc::new(Mutex::new(tokio::io::BufWriter::new(writer))),
            reader: Arc::new(Mutex::new(tokio::io::BufReader::new(reader))),
            signed_in,
        }
    }
    pub async fn user_id(&self) -> u32 {
        let user_id = self.user_id.lock().await;
        user_id.clone()
    }
    pub async fn set_user_id(&self, new_user_id: u32) {
        let mut user_id = self.user_id.lock().await;
        *user_id = new_user_id;
    }
    pub async fn send_packet(&self, msg: &ServerResponse) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer_packet(&mut writer, &msg).await
    }
    pub async fn recv_packet(&self) -> Result<ClientRequest> {
        let mut reader = self.reader.lock().await;
        reader_packet(&mut reader).await
    }
    pub async fn receive_message(&self, group_id: u32, sender: u32, message: String) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let response = ServerResponse::ReceiveMessage {
            group_id,
            sender,
            message,
            timestamp,
        };
        self.send_packet(&response).await.unwrap();
    }
    // 设置登录状态
    pub fn set_signed(&self, signed: bool) {
        self.signed_in.store(signed, Ordering::SeqCst);
    }

    // 检查登录状态
    pub fn is_signed(&self) -> bool {
        self.signed_in.load(Ordering::SeqCst)
    }

    async fn handle_register(&self, username: String, password: String) -> ServerResponse {
        let status = {
            let api = self.api.lock().await;
            api.register(&username, &password).await
        };

        match status {
            Ok(user_id) => match user_id {
                Some(user_id) => ServerResponse::RegisterResponse {
                    status: true,
                    message: format!("注册成功，你的id为{}", user_id),
                },
                None => ServerResponse::RegisterResponse {
                    status: false,
                    message: "注册失败，请稍后重试".to_string(),
                },
            },
            Err(err) => {
                eprintln!("注册失败: {:?}", err);
                ServerResponse::RegisterResponse {
                    status: false,
                    message: "注册失败，请稍后重试".to_string(),
                }
            }
        }
    }

    async fn handle_login(&self, id: u32, password: String) -> ServerResponse {
        let status = {
            let mut api = self.api.lock().await;
            api.login(id, &password, Arc::new(Mutex::new(self.clone())))
                .await
        };

        match status {
            Ok(true) => {
                // 登录成功，更新用户状态
                self.set_signed(true);
                self.set_user_id(id).await;
                ServerResponse::LoginResponse {
                    status: true,
                    message: "登录成功".to_string(),
                }
            }
            Ok(false) => ServerResponse::LoginResponse {
                status: false,
                message: "账号或密码错误".to_string(),
            },
            Err(err) => {
                eprintln!("登录失败: {:?}", err);
                ServerResponse::LoginResponse {
                    status: false,
                    message: "登录失败，请稍后重试".to_string(),
                }
            }
        }
    }

    async fn handle_send_message(
        &self,
        group_id: u32,
        sender: u32,
        receiver: u32,
        message: String,
    ) -> ServerResponse {
        if group_id != 0 {
            let status = {
                let api = self.api.lock().await;
                api.send_group_message(group_id, sender, &message).await
            };

            ServerResponse::GenericResponse {
                status: if status {
                    "ok".to_string()
                } else {
                    "error".to_string()
                },
                message: if status {
                    "消息发送成功".to_string()
                } else {
                    "群组不存在".to_string()
                },
            }
        } else {
            let status = {
                let api = self.api.lock().await;
                api.send_message(sender, receiver, &message).await
            };

            ServerResponse::GenericResponse {
                status: if status {
                    "ok".to_string()
                } else {
                    "error".to_string()
                },
                message: if status {
                    "消息发送成功".to_string()
                } else {
                    "用户不存在".to_string()
                },
            }
        }
    }

    async fn get_online_users(&self) -> ServerResponse {
        let api = self.api.lock().await;
        let online_users = api.online_users().await;
        ServerResponse::OnlineUsers {
            flag: "ok".to_string(),
            user_ids: online_users,
        }
    }

    pub async fn send_error(&self, message: &str) {
        let response = ServerResponse::Error {
            message: message.to_string(),
        };
        self.send_packet(&response).await.unwrap();
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let request = match self.recv_packet().await {
                Ok(req) => req,
                Err(e) => {
                    // 检测到连接断开
                    eprintln!("客户端连接断开，错误: {:?}", e);
                    // 调用 Api.down 方法处理账号下线逻辑
                    let mut api = self.api.lock().await;
                    let user_id = self.user_id.lock().await;
                    if self.is_signed() {
                        api.down(*user_id).await;
                    } // 未登陆的情况下无需处理
                    break; // 跳出循环，停止处理客户端
                }
            };

            let response;

            if self.is_signed() {
                response = match request {
                    ClientRequest::SendMessage {
                        group_id,
                        receiver,
                        message,
                    } => {
                        let user_id = self.user_id().await;
                        self.handle_send_message(group_id, user_id, receiver, message)
                            .await
                    }
                    ClientRequest::ObjRequest { request, id } => match request.as_str() {
                        "get_group_members" => {
                            let api = self.api.lock().await;
                            let members = api.get_group_members(id).await;
                            ServerResponse::GroupMembers {
                                group_id: id,
                                member_ids: members?,
                            }
                        }
                        "user_info" => {
                            let api = self.api.lock().await;
                            match api.get_userinfo(id).await {
                                Ok(Some(userinfo)) => ServerResponse::UserInfo {
                                    user_id: id,
                                    userinfo: UserDetailedInfo {
                                        user_id: id,
                                        username: userinfo.username,
                                    },
                                },
                                _ => ServerResponse::Error {
                                    message: "用户不存在".to_string(),
                                },
                            }
                        }
                        "group_info" => {
                            let api = self.api.lock().await;
                            match api.get_groupinfo(id).await {
                                Ok(Some(groupinfo)) => ServerResponse::GroupInfo {
                                    group_id: id,
                                    groupinfo: GroupDetailedInfo {
                                        group_id: id,
                                        title: groupinfo.title,
                                    },
                                },
                                _ => ServerResponse::Error {
                                    message: "用户不存在".to_string(),
                                },
                            }
                        }
                        "add_friend" => {
                            let user_id = self.user_id().await;
                            let api = self.api.lock().await;
                            let status = api.add_friend(user_id, id).await;
                            ServerResponse::GenericResponse {
                                status: if status.is_ok() {
                                    "ok".to_string()
                                } else {
                                    "error".to_string()
                                },
                                message: if status.is_ok() {
                                    "添加好友成功".to_string()
                                } else {
                                    "添加好友失败".to_string()
                                },
                            }
                        }
                        "add_group" => {
                            let user_id = self.user_id().await;
                            let api = self.api.lock().await;
                            let status = api.add_group(user_id, id).await;
                            ServerResponse::GenericResponse {
                                status: if status.is_ok() {
                                    "ok".to_string()
                                } else {
                                    "error".to_string()
                                },
                                message: if status.is_ok() {
                                    "加入群聊成功".to_string()
                                } else {
                                    "加入群聊失败".to_string()
                                },
                            }
                        }
                        _ => ServerResponse::Error {
                            message: "未知请求".to_string(),
                        },
                    },
                    ClientRequest::NameRequest { request, name } => match request.as_str() {
                        "newgroup" => {
                            let api = self.api.lock().await;
                            let creator_id = self.user_id().await;
                            let result = api.create_group(creator_id, &name).await;
                            match result {
                                Ok(group_id) => ServerResponse::GenericResponse {
                                        status: "ok".to_string(),
                                        message: format!("创建成功，群聊id为{}", group_id),
                                    
                                },
                                Err(err) => {
                                    eprintln!("创建群聊失败: {:?}", err);
                                    ServerResponse::GenericResponse {
                                        status: "failed".to_string(),
                                        message: "创建群聊失败，请稍后重试".to_string(),
                                    }
                                }
                            }
                        }
                        _ => ServerResponse::Error {
                            message: "未知请求".to_string(),
                        },
                    }
                    ClientRequest::MessagesRequest { group, id, offset } => {
                        if group {
                            let api = self.api.lock().await;
                            let messages = api.get_group_messages(id, offset).await;
                            ServerResponse::GroupMessages {
                                group_id: id,
                                messages: messages?,
                            }
                        } else {
                            let user_id = self.user_id().await;
                            let api = self.api.lock().await;
                            let messages = api.get_messages(user_id, id, offset).await;
                            ServerResponse::Messages {
                                sender: id,
                                messages: messages?,
                            }
                        }
                    }
                    ClientRequest::Request { request } => match request.as_str() {
                        "get_groups" => {
                            let user_id = self.user_id().await;
                            let api = self.api.lock().await;
                            let groups = api.get_groups(user_id).await;
                            ServerResponse::GroupList {
                                groups: groups?,
                            }
                        }
                        "get_friends" => {
                            let user_id = self.user_id().await;
                            let api = self.api.lock().await;
                            let friends = api.get_friends(user_id).await;
                            ServerResponse::FriendList {
                                friends: friends?,
                            }
                        }
                        "online_users" => self.get_online_users().await,
                        "ping" => ServerResponse::GenericResponse {
                            status: "ok".to_string(),
                            message: "pong".to_string(),
                        },
                        _ => ServerResponse::Error {
                            message: "未知请求".to_string(),
                        },
                    },
                    ClientRequest::Register {
                        username: _,
                        password: _,
                    } => ServerResponse::Error {
                        message: "已登录状态下无法注册".to_string(),
                    },
                    ClientRequest::Login {
                        user_id: _,
                        password: _,
                    } => ServerResponse::Error {
                        message: "已登录状态下无法重复登录，请先注销".to_string(),
                    },
                }
            } else {
                response = match request {
                    ClientRequest::Register { username, password } => {
                        self.handle_register(username, password).await
                    }
                    ClientRequest::Login { user_id, password } => {
                        self.handle_login(user_id, password).await
                    }
                    _ => ServerResponse::Error {
                        message: "请先登陆".to_string(),
                    },
                };
            }

            // println!("响应: {:?}", response);

            // 尝试发送响应
            if let Err(e) = self.send_packet(&response).await {
                // 检测到发送失败（例如连接断开）
                eprintln!("发送数据失败，连接可能断开: {:?}", e);

                // 调用 Api.down 方法处理账号下线逻辑
                let mut api = self.api.lock().await;
                let user_id = self.user_id().await;
                api.down(user_id).await;

                break; // 跳出循环，停止处理客户端
            }
        }

        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // println!("客户端对象销毁");
        // 这里可以执行更多清理逻辑，例如从全局状态中移除客户端
    }
}
