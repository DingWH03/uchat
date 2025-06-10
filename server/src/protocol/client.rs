use serde::{Deserialize, Serialize};
use crate::protocol::model::{UserSimpleInfo, GroupSimpleInfo, SessionMessage};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ClientRequest {
    #[serde(rename = "request")]
    Request {
        request: String,
    },
    #[serde(rename = "objrequest")]
    ObjRequest {
        request: String,
        id: u32,
    },
    #[serde(rename = "namerequest")]
    NameRequest {
        request: String,
        name: String,
    },
    #[serde(rename = "messagesrequest")]
    MessagesRequest {
        group: bool,
        id: u32,
        offset: u32,
    },
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordRequest {
    pub user_id: u32,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub userid: u32,
    pub password: String,
}

/// 好友请求具体枚举
#[derive(Deserialize, Debug)]
pub enum FriendRequestType {
    Add,
    Info,

}

#[derive(Deserialize, Debug)]
pub struct FriendRequest {
    pub request_type: FriendRequestType,
    pub id: u32,
}

/// 群聊请求具体枚举
#[derive(Deserialize, Debug)]
pub enum GroupRequestType {
    Join,
    Info,
    Creat,
    Leave,
}

#[derive(Deserialize, Debug)]
pub struct GroupRequest {
    pub request_type: GroupRequestType,
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub group_name: String,
    pub members: Vec<u32>, // 成员ID列表
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    SendMessage {
        receiver: u32,
        message: String,
    },
    SendGroupMessage {
        group_id: u32,
        message: String,
    },
}

/// 聊天记录类型
#[derive(Deserialize, Debug)]
pub enum ChatRecordType {
    UserMessage,
    GroupMessage,
}

/// 获取聊天记录的请求
#[derive(Deserialize, Debug)]
pub struct MessageRequest {
    pub record_type: ChatRecordType,
    pub id: u32, // 用户ID或群组ID
    pub offset: u32, // 偏移量，用于分页
}