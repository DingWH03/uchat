use tokio::net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;
use anyhow::{Result, Context};

use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{reader_packet, writer_packet};

#[derive(Clone)]
pub struct CoreApi {
    /// 通道：用于发送客户端请求给服务器
    client_request_sender: mpsc::Sender<ClientRequest>,

    /// 异步锁：用于接收从服务器返回的响应
    server_response_receiver: Arc<Mutex<mpsc::Receiver<ServerResponse>>>,

    /// 通道：用于从读取任务发送服务器响应到响应处理器
    server_response_sender: mpsc::Sender<ServerResponse>,
}

impl CoreApi {
    /// 创建一个新的 CoreApi 实例
    pub async fn new(address: &str, port: u16) -> Result<Self> {
        let socket = TcpStream::connect((address, port))
            .await
            .with_context(|| format!("无法连接到服务器 {}:{}", address, port))?;
        let (read_half, write_half) = socket.into_split();

        // 客户端请求队列
        let (client_request_sender, client_request_receiver) = mpsc::channel::<ClientRequest>(100);
        // 服务端响应队列
        let (server_response_sender, server_response_receiver) =
            mpsc::channel::<ServerResponse>(100);

        let server_response_receiver = Arc::new(Mutex::new(server_response_receiver));
        let reader = Arc::new(Mutex::new(BufReader::new(read_half)));
        let writer = Arc::new(Mutex::new(BufWriter::new(write_half)));

        let core_api = Self {
            client_request_sender,
            server_response_receiver,
            server_response_sender,
        };

        // 启动后台任务
        core_api.spawn_read_task(reader);
        core_api.spawn_write_task(writer, client_request_receiver);

        Ok(core_api)
    }

    /// 启动后台任务：从服务器读取响应
    fn spawn_read_task(&self, reader: Arc<Mutex<BufReader<OwnedReadHalf>>>) {
        let server_response_sender = self.server_response_sender.clone();

        tokio::spawn(async move {
            loop {
                match reader_packet(&mut *reader.lock().await).await {
                    Ok(response) => {
                        if let Err(e) = server_response_sender.send(response).await {
                            eprintln!("消息发送失败，队列已关闭: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("读取服务器消息失败: {:?}", e);
                        break;
                    }
                }
            }
            eprintln!("读取任务已结束");
        });
    }

    /// 启动后台任务：将客户端请求发送到服务器
    fn spawn_write_task(
        &self,
        writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
        mut client_request_receiver: mpsc::Receiver<ClientRequest>,
    ) {
        tokio::spawn(async move {
            while let Some(request) = client_request_receiver.recv().await {
                let mut writer_guard = writer.lock().await;

                if let Err(e) = writer_packet(&mut *writer_guard, &request).await {
                    eprintln!("发送请求失败: {:?}", e);
                    break;
                }

                if let Err(e) = writer_guard.flush().await {
                    eprintln!("写入数据时发生错误: {:?}", e);
                    break;
                }
            }
            eprintln!("写入任务已结束");
        });
    }

    // ---------------------------
    // 以下是对外暴露的API
    // ---------------------------

    /// 注册用户
    pub async fn send_register(&self, username: &str, password: &str) -> Result<()> {
        let request = ClientRequest::Register {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.client_request_sender
            .send(request)
            .await
            .with_context(|| "发送注册请求失败")
    }

    /// 用户登录
    pub async fn send_login(&self, user_id: u32, password: &str) -> Result<()> {
        let request = ClientRequest::Login {
            user_id,
            password: password.to_string(),
        };
        self.client_request_sender
            .send(request)
            .await
            .with_context(|| "发送登录请求失败")
    }

    /// 发送消息
    pub async fn send_message(&self, group_id: u32, receiver: u32, message: &str) -> Result<()> {
        let request = ClientRequest::SendMessage {
            group_id,
            receiver,
            message: message.to_string(),
        };
        self.client_request_sender
            .send(request)
            .await
            .with_context(|| "发送消息失败")
    }

    /// 获取在线用户列表
    pub async fn get_online_users(&self) -> Result<()> {
        let request = ClientRequest::Request {
            request: "online_users".to_string(),
        };
        self.client_request_sender
            .send(request)
            .await
            .with_context(|| "获取在线用户列表失败")
    }

    /// 监听消息队列，处理服务器推送
    pub async fn listen_messages<F>(&self, mut handler: F)
    where
        F: FnMut(ServerResponse) + Send + 'static,
    {
        let server_response_receiver = Arc::clone(&self.server_response_receiver);

        tokio::spawn(async move {
            while let Some(response) = server_response_receiver.lock().await.recv().await {
                // match &response {
                //     ServerResponse::ReceiveMessage { sender, message, timestamp } => {
                //         println!("[{}] 用户({})发来消息: {}", timestamp, sender, message);
                //     }
                //     ServerResponse::Error { message } => {
                //         eprintln!("服务端错误: {}", message);
                //     }
                //     ServerResponse::OnlineUsers { user_ids, .. } => {
                //         println!("当前在线用户列表: {:?}", user_ids);
                //     }
                //     _ => {
                //         println!("收到其他类型响应/推送: {:?}", response);
                //     }
                // }
                handler(response);
            }
            eprintln!("消息队列已关闭，停止监听");
        });
    }
}
