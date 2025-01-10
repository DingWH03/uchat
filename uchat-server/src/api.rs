use crate::client::Client;
use bcrypt::hash;
use bcrypt::BcryptError;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex; // 引入 bcrypt 库
use crate::db::Database as DB;

pub struct Api {
    db: DB,
    clients: HashMap<u32, Arc<Mutex<Client>>>,
}

impl Api {
    pub fn new(
        db: DB,
        clients: HashMap<u32, Arc<Mutex<Client>>>,
    ) -> Self {
        Self { db, clients }
    }

    /// 处理用户登陆请求
    pub async fn login(
        &mut self,
        id: u32,
        password: &str,
        client: Arc<Mutex<Client>>, // 将自己的引用传进来
    ) -> Result<bool, sqlx::Error> {
        // 查询结果密码哈希
        let password_hash = self.db.get_password_hash(id).await;

        match password_hash {
            Ok(Some(password_hash)) => {
                // 验证用户输入的密码是否与数据库中的哈希值匹配
                match bcrypt::verify(password, &password_hash) {
                    Ok(valid) => {
                        if valid {
                            if let Some(client) = self.clients.get(&id) {
                                let client = client.lock().await;
                                client.send_error("用户重复登录").await;
                                return Ok(false);
                            } else {
                                // 将客户端引用存入 clients 中
                                // &self.clients.insert(row.id.to_string(), client);
                                let _ = &self.clients.insert(id, client);
                                println!("用户 {} 登录", id);
                                return Ok(true);
                            }
                        }
                        Ok(valid)
                    }
                    Err(_) => Ok(false), // 如果验证失败或发生错误，返回 false
                }
            }
            Ok(None) => Ok(false),
            Err(_) => Ok(false),
        }
        
    }

    /// 处理用户注册请求
    pub async fn register(&self, username: &str, password: &str) -> Result<Option<u32>, BcryptError> {
        let hashed = hash(&password, 4)?;
        let user_id = self.db.new_user(username, &hashed).await;
        match user_id {
            Ok(Some(user_id)) => Ok(Some(user_id)),
            _ => Ok(None),
        }
    }

    /// 处理用户下线请求
    pub async fn down(&mut self, user_id: u32) {
        println!("用户 {} 下线", user_id);
        self.clients.remove(&user_id);
    }

    /// 处理用户发送私聊消息请求
    pub async fn send_message(&self, sender: u32, receiver: u32, message: &str) -> bool {
        
        if receiver == 0 {
            // 创建一个异步任务集合
            let clients = self.clients.clone(); // 克隆 Arc 引用

            // 创建一个异步任务集合
            let tasks: Vec<_> = clients
                .iter()
                .map(|(client_id, client)| {
                    let sender = sender;
                    let message = message.to_string();
                    let _client_id = client_id.clone(); // 克隆 client_id
                    let client = client.clone(); // 克隆锁

                    tokio::spawn(async move {
                        let client = client.lock().await;
                        client.receive_message(sender, message).await;
                    })
                })
                .collect();

            // 等待所有任务完成
            for task in tasks {
                if let Err(err) = task.await {
                    println!("并发任务失败: {}", err);
                }
            }
            true
        } else {
            // 单个接收者处理逻辑
            if let Some(client) = self.clients.get(&receiver) {
                let client = client.lock().await;
                client
                    .receive_message(sender, message.to_string())
                    .await;
                true
            } else {
                println!("接收者 {} 不在线或不存在", receiver);
                false
            }
        }
    }

    /// 返回在线id列表
    pub async fn online_users(&self) -> Vec<u32> {
        self.clients.keys().cloned().collect()
    }
    /// 返回自己的用户名
    pub async fn get_username(&self, id: u32) -> Result<Option<String>, sqlx::Error> {
        self.db.get_username(id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
    /// 返回自己的好友列表
    pub async fn get_friends(&self, id: u32) -> Result<Vec<u32>, sqlx::Error> {
        self.db.get_friends(id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
    /// 返回自己的群聊列表
    pub async fn get_groups(&self, id: u32) -> Result<Vec<u32>, sqlx::Error> {
        self.db.get_groups(id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
    pub async fn get_group_members(&self, group_id: u32) -> Result<Vec<u32>, sqlx::Error> {
        self.db.get_group_members(group_id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
    /// 添加好友
    pub async fn add_friend(&self, user_id: u32, friend_id: u32) -> Result<(), sqlx::Error> {
        self.db.add_friend(user_id, friend_id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
    /// 添加群聊
    pub async fn add_group(&self, user_id: u32, group_id: u32) -> Result<(), sqlx::Error> {
        self.db.add_group(user_id, group_id).await.map_err(|e| sqlx::Error::Decode(e.into()))
    }
}
