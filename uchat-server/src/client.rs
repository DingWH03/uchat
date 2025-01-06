// src/client.rs
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::protocol::{User, ClientRequest, ServerResponse};
use crate::utils::{reader_packet, writer_packet};
use crate::api::Api;

#[derive(Clone)]
pub struct Client {
    api: Arc<Mutex<Api>>,
    user: Arc<Mutex<User>>,
    writer: Arc<Mutex<tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>>>,
    reader: Arc<Mutex<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>>,
    signed_in: Arc<Mutex<bool>>,
}

impl Client {
    pub fn new(
        socket: TcpStream,
        api: Arc<Mutex<Api>>,
        user: Arc<Mutex<User>>,
        signed_in: Arc<Mutex<bool>>,
    ) -> Self {
        let (reader, writer) = socket.into_split();
        Self {
            api,
            user,
            writer: Arc::new(Mutex::new(tokio::io::BufWriter::new(writer))),
            reader: Arc::new(Mutex::new(tokio::io::BufReader::new(reader))),
            signed_in,
        }
    }
    pub async fn user_id(&self) -> u32 {
        let user = self.user.lock().await;
        user.user_id.clone()
    }
    pub async fn username(&self) -> String {
        let user = self.user.lock().await;
        user.username.clone()
    }
    pub async fn send_packet(&mut self, msg: &ServerResponse) -> Result<()> {
        writer_packet(&mut self.writer, &msg).await
    }
    pub async fn recv_packet(&mut self) -> Result<ClientRequest> {
        reader_packet(&mut self.reader).await
    }
    pub async fn receive_message(&mut self, sender: u32, message: String) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let response = ServerResponse::ReceiveMessage {
            sender,
            message,
            timestamp,
        };
        self.send_packet(&response).await.unwrap();
    }

    async fn handle_register(&self, username: String, password: String) -> ServerResponse {
        let status = {
            let api = self.api.lock().await;
            api.register(&username, &password).await
        };
    
        match status {
            Ok(user_id) => {
                match user_id {
                    Some(user_id) => {
                        ServerResponse::AuthResponse {
                            status: "success".to_string(),
                            message: format!("注册成功，你的id为{}", user_id),
                        }
                    }
                    None => {
                        ServerResponse::AuthResponse {
                            status: "error".to_string(),
                            message: "注册失败，请稍后重试".to_string(),
                        }
                    }
                    
                }
            },
            Err(err) => {
                eprintln!("注册失败: {:?}", err);
                ServerResponse::AuthResponse {
                    status: "error".to_string(),
                    message: "注册失败，请稍后重试".to_string(),
                }
            }
        }
    }
    

    async fn handle_login(&mut self, id: u32, password: String) -> ServerResponse {
        let status = {
            let mut api = self.api.lock().await;
            api.login(id, &password, Arc::new(Mutex::new(self.clone()))).await
        };
    
        match status {
            Ok(true) => {
                // 登录成功，更新用户状态
                let mut signed_in = self.signed_in.lock().await;
                *signed_in = true;
                let api = self.api.lock().await;
                match api.get_username(id).await { // 获取用户名，且成功或失败后都需要修改用户名
                    Ok(Some(name)) => {
                        let mut user = self.user.lock().await;
                        user.user_id = id;
                        user.username = name;
                    }
                    _ => {
                        let mut user = self.user.lock().await;
                        user.user_id = id;
                        user.username = "未知用户".to_string();
                    }
                }
    
                ServerResponse::AuthResponse {
                    status: "success".to_string(),
                    message: "登录成功".to_string(),
                }
            }
            Ok(false) => ServerResponse::AuthResponse {
                status: "error".to_string(),
                message: "账号或密码错误".to_string(),
            },
            Err(err) => {
                eprintln!("登录失败: {:?}", err);
                ServerResponse::AuthResponse {
                    status: "error".to_string(),
                    message: "登录失败，请稍后重试".to_string(),
                }
            }
        }
    }
    

    async fn handle_send_message(
        &self,
        sender: u32,
        receiver: u32,
        message: String,
    ) -> ServerResponse {
        let status = {
            let api = self.api.lock().await;
            api.send_message(sender, receiver, &message).await
        };

        ServerResponse::AuthResponse {
            status: if status { "ok".to_string() } else { "error".to_string() },
            message: if status { "消息发送成功".to_string() } else { "用户不存在".to_string() },
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

    pub async fn send_error(&mut self, message: &str) {
        let response = ServerResponse::Error {
            message: message.to_string(),
        };
        self.send_packet(&response).await.unwrap();
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = match self.recv_packet().await {
                Ok(req) => req,
                Err(e) => {
                    // 检测到连接断开
                    eprintln!("客户端连接断开，错误: {:?}", e);
                    // 调用 Api.down 方法处理账号下线逻辑
                    let mut api = self.api.lock().await;
                    let user = self.user.lock().await;
                    let is_signed_in = self.signed_in.lock().await;
                    if *is_signed_in {api.down(user.user_id).await;} // 未登陆的情况下无需处理
                    break; // 跳出循环，停止处理客户端
                }
            };

            let response = match request {
                ClientRequest::SendMessage { receiver, message } => {
                    let signed_in = self.signed_in.lock().await;
                    if !*signed_in {
                        ServerResponse::Error {
                            message: "请先登录".to_string(),
                        }
                    } else {
                        let user_id = self.user_id().await;
                        self.handle_send_message(user_id, receiver, message).await
                    }
                }
                ClientRequest::Request { request } => match request.as_str() {
                    "online_users" => self.get_online_users().await,
                    "my_username" => {
                        let username = self.username().await;
                        ServerResponse::UserName {
                            user_id: self.user_id().await,
                            username,
                        }
                    }
                    "ping" => ServerResponse::AuthResponse {
                        status: "ok".to_string(),
                        message: "pong".to_string(),
                    },
                    _ => ServerResponse::Error {
                        message: "未知请求".to_string(),
                    },
                },
                ClientRequest::CheckUserInfo { user_id } => {
                    let api = self.api.lock().await;
                    match api.get_username(user_id).await {
                        Ok(Some(username)) => ServerResponse::UserName { user_id, username },
                        _ => ServerResponse::Error {
                            message: "用户不存在".to_string(),
                        },
                    }
                }
                ClientRequest::Register { username, password } => {
                    self.handle_register(username, password).await
                }
                ClientRequest::Login { user_id, password } => {
                    self.handle_login(user_id, password).await
                }
            };

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
