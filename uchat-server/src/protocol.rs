// src/protocol.rs
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSimpleInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailedInfo {
    pub user_id: u32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSimpleInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDetailedInfo {
    pub group_id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub sender_id: u32,
    pub message: String,
    pub timestamp: NaiveDateTime,
}


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
    #[serde(rename = "register")]
    Register {
        username: String,
        password: String,
    },
    #[serde(rename = "login")]
    Login {
        user_id: u32,
        password: String,
    },
    #[serde(rename = "send_message")]
    SendMessage {
        group_id: u32,
        receiver: u32,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ServerResponse {
    #[serde(rename = "generic_response")]
    GenericResponse {
        status: String,
        message: String,
    },
    #[serde(rename = "login_response")]
    LoginResponse {
        status: bool,
        message: String,
    },
    #[serde(rename = "register_response")]
    RegisterResponse {
        status: bool,
        message: String,
    },
    #[serde(rename = "receive_message")]
    ReceiveMessage {
        group_id: u32,
        sender: u32,
        message: String,
        timestamp: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "online_users")]
    OnlineUsers {
        flag: String,
        user_ids: Vec<u32>,
    },
    #[serde(rename = "userinfo")]
    UserInfo {
        user_id: u32,
        userinfo: UserDetailedInfo,
    },
    #[serde(rename = "groupinfo")]
    GroupInfo {
        group_id: u32,
        groupinfo: GroupDetailedInfo,
    },
    /// 响应objrequest->get_group_members
    #[serde(rename = "group_members")]
    GroupMembers {
        group_id: u32,
        member_ids: Vec<UserSimpleInfo>,
    },
    /// 响应request->get_friends
    #[serde(rename = "friend_list")]
    FriendList {
        friends: Vec<UserSimpleInfo>,
    },
    /// 响应request->get_groups
    #[serde(rename = "group_list")]
    GroupList {
        groups: Vec<GroupSimpleInfo>,
    },
    /// 响应messagesrequest
    #[serde(rename = "messages")]
    Messages {
        sender: u32,
        messages: Vec<Message>,
    },
    #[serde(rename = "groupmessages")]
    GroupMessages {
        group_id: u32,
        messages: Vec<Message>,
    },
}