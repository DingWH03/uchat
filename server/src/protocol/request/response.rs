use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::protocol::{model::{GroupDetailedInfo, GroupSimpleInfo, SessionMessage, UserDetailedInfo, UserSimpleInfo, UserSimpleInfoWithStatus}, RoleType};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
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
    #[serde(rename = "check_session_response")]
    CheckSessionResponse {
        status: bool,
        role: RoleType,
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
    /// 响应request->get_friends_with_status
    #[serde(rename = "friend_list_with_status")]
    FriendListWithStatus {
        friends: Vec<UserSimpleInfoWithStatus>,
    },
    /// 响应request->get_groups
    #[serde(rename = "group_list")]
    GroupList {
        groups: Vec<GroupSimpleInfo>,
    },
    /// 响应messagesrequest
    #[serde(rename = "messages")]
    Messages {
        friend_id: u32,
        messages: Vec<SessionMessage>,
    },
    #[serde(rename = "groupmessages")]
    GroupMessages {
        group_id: u32,
        messages: Vec<SessionMessage>,
    },
}

