use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;
use anyhow::Result;

use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{reader_packet, writer_packet};

/// CoreApi 负责与服务器进行通信：
/// - 通过 send_request(...) 发送请求
/// - 后台循环接收服务端的所有响应和推送消息
/// - 将接收到的消息放入异步队列 (mpsc::Sender)
/// - 用户可以通过 listen_messages(...) 从队列中获取消息并处理
#[derive(Clone)]
pub struct CoreApi {
    /// 发往服务器的写端（带缓冲）
    writer: Arc<Mutex<BufWriter<tokio::net::tcp::OwnedWriteHalf>>>,
    /// 接收消息的异步队列接收端
    msg_rx: Arc<Mutex<mpsc::Receiver<ServerResponse>>>,
    /// 接收消息的异步队列发送端
    msg_tx: mpsc::Sender<ServerResponse>,
}

impl CoreApi {
    /// 初始化 CoreApi 并建立连接，启动一个后台任务持续接收服务器消息
    pub async fn new(address: &str, port: u16) -> Result<Self> {
        // 1. 与服务器建立 TCP 连接
        let socket = TcpStream::connect((address, port)).await?;
        let (read_half, write_half) = socket.into_split();

        // 2. 准备一个 mpsc 通道，容量可自行调节
        let (tx, rx) = mpsc::channel(100);

        // 3. 构造 BufReader/BufWriter
        let reader = Arc::new(Mutex::new(BufReader::new(read_half)));
        let writer = Arc::new(Mutex::new(BufWriter::new(write_half)));

        // 4. 构造 CoreApi
        let core_api = Self {
            writer,
            msg_rx: Arc::new(Mutex::new(rx)),
            msg_tx: tx.clone(),
        };

        // 5. 启动后台任务，循环从服务器读取消息，放入队列
        {
            let reader = Arc::clone(&reader);
            tokio::spawn(async move {
                loop {
                    match reader_packet(&mut *reader.lock().await).await {
                        Ok(response) => {
                            // 收到服务器消息，放入队列
                            if let Err(e) = tx.send(response).await {
                                eprintln!("发送消息到队列失败: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("接收消息失败: {:?}", e);
                            eprintln!("这里可以添加重连逻辑，例如等待几秒后重新连接…");
                            // 比如:
                            // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            // 尝试再去连接服务器
                            break; 
                        }
                    }
                }
            });
        }

        Ok(core_api)
    }

    /// 发送请求到服务器（只负责发送，不读取响应）
    async fn send_request(&self, request: &ClientRequest) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer_packet(&mut *writer, request).await?;
        writer.flush().await?;
        Ok(())
    }

    /// 发送注册请求（不阻塞等待响应，响应将进入消息队列）
    pub async fn send_register(&self, username: &str, password: &str) -> Result<()> {
        let request = ClientRequest::Register {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.send_request(&request).await
    }

    /// 发送登录请求（不阻塞等待响应，响应将进入消息队列）
    pub async fn send_login(&self, user_id: u32, password: &str) -> Result<()> {
        let request = ClientRequest::Login {
            user_id,
            password: password.to_string(),
        };
        self.send_request(&request).await
    }

    /// 发送普通聊天消息
    pub async fn send_message(&self, receiver: u32, message: &str) -> Result<()> {
        let request = ClientRequest::SendMessage {
            receiver,
            message: message.to_string(),
        };
        self.send_request(&request).await
    }

    /// 获取在线用户
    pub async fn get_online_users(&self) -> Result<()> {
        let request = ClientRequest::Request {
            request: "online_users".to_string(),
        };
        self.send_request(&request).await
    }

    // --------------------------------------------------------------
    // 消息监听逻辑
    // --------------------------------------------------------------

    /// 启动一个任务，不断从队列中读取消息并处理。
    /// - `handler` 是用户自定义的回调，拿到每条消息后可以做进一步的处理
    /// - 这样就不会阻塞其他API的通信
    pub async fn listen_messages<F>(&self, mut handler: F)
    where
        F: FnMut(ServerResponse) + Send + 'static,
    {
        // 我们在这里克隆接收端的引用，然后在新的异步任务中不断地 `recv`
        let msg_rx = Arc::clone(&self.msg_rx);

        tokio::spawn(async move {
            loop {
                // 从队列中取出一条消息
                match msg_rx.lock().await.recv().await {
                    Some(response) => {
                        // 这里可以先做分类处理，比如分别处理聊天、错误、在线用户列表等
                        match &response {
                            ServerResponse::ReceiveMessage { sender, message, timestamp } => {
                                println!("[{}] 用户({})发来消息: {}", timestamp, sender, message);
                            }
                            ServerResponse::Error { message } => {
                                eprintln!("服务端错误: {}", message);
                            }
                            ServerResponse::OnlineUsers { user_ids, .. } => {
                                println!("当前在线用户列表: {:?}", user_ids);
                            }
                            _ => {
                                println!("收到其他类型响应/推送: {:?}", response);
                            }
                        }

                        // 交给用户传进来的回调进行额外处理
                        handler(response);
                    }
                    None => {
                        eprintln!("消息队列已经关闭，结束监听任务。");
                        break;
                    }
                }
            }
        });
    }
}
