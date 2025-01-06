// src/core_api.rs
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;
use anyhow::Result;
use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{reader_packet, writer_packet};

pub struct CoreApi {
    writer: Arc<Mutex<BufWriter<tokio::net::tcp::OwnedWriteHalf>>>,
    reader: Arc<Mutex<BufReader<tokio::net::tcp::OwnedReadHalf>>>,
    // 你可以添加更多字段，例如当前用户信息等
}

impl CoreApi {
    /// 初始化 CoreApi 并建立连接
    pub async fn new(address: &str, port: u16) -> Result<Self> {
        let socket = TcpStream::connect((address, port)).await?;
        let (reader, writer) = socket.into_split();
        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(writer))),
            reader: Arc::new(Mutex::new(BufReader::new(reader))),
        })
    }

    /// 发送请求到服务器
    async fn send_request(&self, request: &ClientRequest) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer_packet(&mut *writer, request).await?;
        writer.flush().await?;
        Ok(())
    }

    /// 接收服务器响应
    async fn receive_response(&self) -> Result<ServerResponse> {
        let mut reader = self.reader.lock().await;
        let response = reader_packet(&mut *reader).await?;
        Ok(response)
    }

    /// 注册新用户
    pub async fn register(&self, username: &str, password: &str) -> Result<ServerResponse> {
        let request = ClientRequest::Register {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 登录
    pub async fn login(&self, user_id: u32, password: &str) -> Result<ServerResponse> {
        let request = ClientRequest::Login {
            user_id,
            password: password.to_string(),
        };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 发送消息
    pub async fn send_message(&self, receiver: u32, message: &str) -> Result<ServerResponse> {
        let request = ClientRequest::SendMessage {
            receiver,
            message: message.to_string(),
        };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 获取在线用户列表
    pub async fn get_online_users(&self) -> Result<ServerResponse> {
        let request = ClientRequest::Request {
            request: "online_users".to_string(),
        };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 获取用户名
    pub async fn get_my_username(&self) -> Result<ServerResponse> {
        let request = ClientRequest::Request {
            request: "my_username".to_string(),
        };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 检查用户信息
    pub async fn check_user_info(&self, user_id: u32) -> Result<ServerResponse> {
        let request = ClientRequest::CheckUserInfo { user_id };
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 处理接收的消息（可以在单独的任务中调用此方法以保持接收）
    pub async fn listen(&self, on_message: impl Fn(ServerResponse) + Send + 'static) -> Result<()> {
        let reader = Arc::clone(&self.reader);
        tokio::spawn(async move {
            loop {
                match reader_packet(&mut *reader.lock().await).await {
                    Ok(response) => {
                        on_message(response);
                    }
                    Err(e) => {
                        eprintln!("接收消息失败: {:?}", e);
                        break;
                    }
                }
            }
        });
        Ok(())
    }
}
