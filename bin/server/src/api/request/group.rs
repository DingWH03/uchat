use uchat_protocol::{request::RequestResponse, GroupDetailedInfo, GroupSimpleInfo, UserSimpleInfo};
use log::{error, warn};

use super::Request;

impl Request {
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
            Ok(_) => {
                self.cache.invalidate_group_members(group_id).await;
                RequestResponse::ok("加入成功", ())},
            Err(e) => {
                error!("加入群聊失败：{}", e);
                RequestResponse::bad_request("群聊不存在或服务器错误")
            }
        }
    }
    /// 用户退出群聊
    pub async fn leave_group(&self, user_id: u32, group_id: u32) -> RequestResponse<()> {
        match self.db.leave_group(user_id, group_id).await {
            Ok(_) => {
                self.cache.invalidate_group_members(group_id).await;
                RequestResponse::ok("退出成功", ())},
            Err(e) => {
                error!("退出群聊失败：{}", e);
                RequestResponse::bad_request("群聊不存在或服务器错误")
            }
        }
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
}