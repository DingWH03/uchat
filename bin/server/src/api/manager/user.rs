use log::{error, info};

use uchat_model::{
    ManagerResponse, ManagerUserSimpleInfo, RoleType, UserDetailedInfo, UserSimpleInfo,
};

use super::Manager;

impl Manager {
    /// 查看所有用户
    pub async fn get_all_user(&self) -> ManagerResponse<Vec<ManagerUserSimpleInfo>> {
        info!("响应获取全部用户");
        match self.db.get_all_user().await {
            Ok(result) => ManagerResponse::ok("获取成功", result),
            Err(e) => {
                error!("查看所有用户失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 获取总用户人数
    pub async fn get_users_count(&self) -> ManagerResponse<u32> {
        info!("响应获取总用户人数");
        match self.db.get_user_count().await {
            Ok(num) => ManagerResponse::ok("获取成功", num),
            Err(e) => {
                error!("获取总用户人数失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 修改用户身份
    pub async fn set_user_role(&self, user_id: u32, role: RoleType) -> ManagerResponse<()> {
        info!("修改用户{}身份为{}", user_id, role);
        match self.db.change_user_role(user_id, role).await {
            Ok(_) => ManagerResponse::ok("修改成功", ()),
            Err(e) => {
                error!("修改用户身份失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 获取用户详细信息
    pub async fn get_user_detail(&self, user_id: u32) -> ManagerResponse<UserDetailedInfo> {
        info!("查看用户{}详细信息", user_id);
        match self.db.get_userinfo(user_id).await {
            Ok(result) => match result {
                Some(info) => ManagerResponse::ok("获取成功", info),
                None => ManagerResponse::err("用户不存在"),
            },
            Err(e) => {
                error!("获取用户详细信息失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 删除用户
    pub async fn delete_user(&self, user_id: u32) -> ManagerResponse<()> {
        info!("删除用户{}", user_id);
        match self.db.delete_user(user_id).await {
            Ok(_) => ManagerResponse::ok("删除成功", ()),
            Err(e) => {
                error!("删除用户失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 获取某用户的friend
    pub async fn get_friends(&self, user_id: u32) -> ManagerResponse<Vec<UserSimpleInfo>> {
        info!("查看用户{}好友", user_id);
        match self.db.get_friends(user_id).await {
            Ok(result) => ManagerResponse::ok("获取成功", result),
            Err(e) => {
                error!("获取好友失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
    /// 删除某用户某好友
    pub async fn delete_friendship(&self, user_id: u32, friend_id: u32) -> ManagerResponse<()> {
        info!("删除{}与{}的好友关系", user_id, friend_id);
        match self.db.delete_friendship(user_id, friend_id).await {
            Ok(_) => ManagerResponse::ok("删除成功", ()),
            Err(e) => {
                error!("删除好友关系失败，检查数据库错误: {}", e);
                ManagerResponse::err(format!("数据库错误：{}", e))
            }
        }
    }
}
