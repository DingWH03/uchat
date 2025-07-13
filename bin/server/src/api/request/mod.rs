mod messages;
mod user;
mod session;
mod message;
mod utils;
mod group;
mod friend;

use crate::cache::CacheConfig;
use crate::cache::CacheManagerTrait;
use crate::db::DB;
use crate::session::SessionConfig;
use crate::session::SessionManagerTrait;
use crate::storage::ObjectStorage;
use log::{error};
use std::sync::Arc;
use uchat_protocol::{
    ContactList, request::RequestResponse,
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



    /// 对ping请求的响应
    pub async fn ping(&self) -> RequestResponse<()> {
        RequestResponse::ok("pong", ())
    }
}
