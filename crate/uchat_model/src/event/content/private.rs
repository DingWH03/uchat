use std::net::{IpAddr};
use serde::{Serialize, Deserialize};
use crate::{model::UserId, GroupId};


#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LoginInfo {
    pub status: LoginStatus,
    pub user_id: UserId,
    pub ip: IpAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum LoginStatus {
    Success,
    BadPassword,
    UserNotFound,
    Disabled,
}

/// (暂时)不需要记录更新的内容，只需要标记更新过个人信息，全量重新拉取即可
/// 以后可以扩展为记录具体更新了哪些字段
/// 可以记录是群组信息更新或是用户信息更新
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ProfileInfo {
    pub user_id: Option<UserId>,
    pub group_id: Option<GroupId>,
}
