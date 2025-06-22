use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::protocol::{model::{GroupDetailedInfo, GroupSimpleInfo, SessionMessage, UserDetailedInfo, UserSimpleInfo, UserSimpleInfoWithStatus}, RoleType};


#[derive(Serialize, Debug, ToSchema)]
pub struct RequestResponse<T> {
    pub status: bool,
    pub code: u16,
    pub message: String, // 提示信息统一为 String
    pub data: Option<T>, // 仅在成功时存在数据
}

impl<T> RequestResponse<T> {
    pub fn ok(message: impl Into<String>, data: T) -> Self {
        Self {
            status: true,
            code: 200,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            status: false,
            code: 500,
            message: message.into(),
            data: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: false,
            code: 400,
            message: message.into(),
            data: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: false,
            code: 401,
            message: "认证失败".to_string(),
            data: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: false,
            code: 404,
            message: "找不到信息".to_string(),
            data: None,
        }
    }
}

impl<T: serde::Serialize> IntoResponse for RequestResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK);
        (status, Json(self)).into_response()
    }
}


/// 通用响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

/// 登录响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LoginResponse {
    pub status: bool,
    pub message: String,
}

/// 检查会话响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CheckSessionResponse {
    pub status: bool,
    pub role: RoleType,
}

/// 注册响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RegisterResponse {
    pub status: bool,
    pub message: String,
}

/// 收到消息
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ReceiveMessage {
    pub group_id: u32,
    pub sender: u32,
    pub message: String,
    pub timestamp: String,
}

/// 错误响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

/// 在线用户列表
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct OnlineUsers {
    pub flag: String,
    pub user_ids: Vec<u32>,
}

/// 用户信息响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UserInfo {
    pub user_id: u32,
    pub userinfo: UserDetailedInfo,
}

/// 群聊信息响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GroupInfo {
    pub group_id: u32,
    pub groupinfo: GroupDetailedInfo,
}

/// 群成员列表响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GroupMembers {
    pub group_id: u32,
    pub member_ids: Vec<UserSimpleInfo>,
}

/// 好友列表响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FriendList {
    pub friends: Vec<UserSimpleInfo>,
}

/// 好友列表（含状态）响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FriendListWithStatus {
    pub friends: Vec<UserSimpleInfoWithStatus>,
}

/// 群组列表响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GroupList {
    pub groups: Vec<GroupSimpleInfo>,
}

/// 私聊消息响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Messages {
    pub friend_id: u32,
    pub messages: Vec<SessionMessage>,
}

/// 群聊消息响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GroupMessages {
    pub group_id: u32,
    pub messages: Vec<SessionMessage>,
}

// #[derive(Serialize, Deserialize, Debug, ToSchema)]
// #[serde(tag = "action")]
// pub enum ServerResponse {
//     #[serde(rename = "generic_response")]
//     GenericResponse {
//         status: String,
//         message: String,
//     },
//     #[serde(rename = "login_response")]
//     LoginResponse {
//         status: bool,
//         message: String,
//     },
//     #[serde(rename = "check_session_response")]
//     CheckSessionResponse {
//         status: bool,
//         role: RoleType,
//     },
//     #[serde(rename = "register_response")]
//     RegisterResponse {
//         status: bool,
//         message: String,
//     },
//     #[serde(rename = "receive_message")]
//     ReceiveMessage {
//         group_id: u32,
//         sender: u32,
//         message: String,
//         timestamp: String,
//     },
//     #[serde(rename = "error")]
//     Error {
//         message: String,
//     },
//     #[serde(rename = "online_users")]
//     OnlineUsers {
//         flag: String,
//         user_ids: Vec<u32>,
//     },
//     #[serde(rename = "userinfo")]
//     UserInfo {
//         user_id: u32,
//         userinfo: UserDetailedInfo,
//     },
//     #[serde(rename = "groupinfo")]
//     GroupInfo {
//         group_id: u32,
//         groupinfo: GroupDetailedInfo,
//     },
//     /// 响应objrequest->get_group_members
//     #[serde(rename = "group_members")]
//     GroupMembers {
//         group_id: u32,
//         member_ids: Vec<UserSimpleInfo>,
//     },
//     /// 响应request->get_friends
//     #[serde(rename = "friend_list")]
//     FriendList {
//         friends: Vec<UserSimpleInfo>,
//     },
//     /// 响应request->get_friends_with_status
//     #[serde(rename = "friend_list_with_status")]
//     FriendListWithStatus {
//         friends: Vec<UserSimpleInfoWithStatus>,
//     },
//     /// 响应request->get_groups
//     #[serde(rename = "group_list")]
//     GroupList {
//         groups: Vec<GroupSimpleInfo>,
//     },
//     /// 响应messagesrequest
//     #[serde(rename = "messages")]
//     Messages {
//         friend_id: u32,
//         messages: Vec<SessionMessage>,
//     },
//     #[serde(rename = "groupmessages")]
//     GroupMessages {
//         group_id: u32,
//         messages: Vec<SessionMessage>,
//     },
// }

