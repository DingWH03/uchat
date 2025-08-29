// server/src/api/doc.rs

use crate::api::handler::__path_handle_request;
use crate::api::handler::__path_ping;
use crate::api::handler::manager::message::privite::__path_handle_delete_message;
use crate::api::handler::manager::message::privite::__path_handle_get_message;
use crate::api::handler::manager::message::privite::__path_handle_get_recent_messages;
use crate::api::handler::manager::message::privite::__path_handle_get_user_recent_messages;
use crate::api::handler::manager::online::session::__path_handle_delete_session;
use crate::api::handler::manager::online::tree::__path_handle_tree_online;
use crate::api::handler::manager::user::count::__path_handle_user_get_count;
use crate::api::handler::manager::user::delete::__path_handle_delete_user;
use crate::api::handler::manager::user::detail::__path_handle_get_userinfo;
use crate::api::handler::manager::user::friend::__path_handle_delete_friendship;
use crate::api::handler::manager::user::friend::__path_handle_get_friends;
use crate::api::handler::manager::user::list::__path_handle_list_user;
use crate::api::handler::manager::user::role::__path_handle_change_role;
use crate::api::handler::request::authentication::login::__path_handle_login;
use crate::api::handler::request::authentication::logout::__path_handle_logout;
use crate::api::handler::request::authentication::password::__path_handle_passwd;
use crate::api::handler::request::authentication::register::__path_handle_register;
use crate::api::handler::request::authentication::session::__path_handle_check_session;
use crate::api::handler::request::authentication::ws_connect::__path_handle_connect;
use crate::api::handler::request::friend::add_friend::__path_handle_add_friend;
use crate::api::handler::request::friend::info_friend::__path_handle_info_friend;
use crate::api::handler::request::friend::list_friend::__path_handle_list_friend;
use crate::api::handler::request::friend::list_friend::__path_handle_list_friend_with_status;
use crate::api::handler::request::friend::status_friend::__path_handle_get_status_by_userid;
use crate::api::handler::request::group::creat_group::__path_handle_creat_group;
use crate::api::handler::request::group::info_group::__path_handle_info_group;
use crate::api::handler::request::group::join_group::__path_handle_join_group;
use crate::api::handler::request::group::leave_group::__path_handle_leave_group;
use crate::api::handler::request::group::list_group::__path_handle_list_group;
use crate::api::handler::request::group::members_group::__path_handle_members_group;
use crate::api::handler::request::message::group::__path_handle_get_all_group_messages_after_timestamp;
use crate::api::handler::request::message::group::__path_handle_get_group_message;
use crate::api::handler::request::message::group::__path_handle_get_group_messages_after_timestamp;
use crate::api::handler::request::message::group::__path_handle_get_latest_timestamp_of_all_group_messages;
use crate::api::handler::request::message::group::__path_handle_get_latest_timestamp_of_group;
use crate::api::handler::request::message::group::__path_handle_get_latest_timestamps_of_all_groups;
use crate::api::handler::request::message::private::__path_handle_get_all_private_messages_after_timestamp;
use crate::api::handler::request::message::private::__path_handle_get_latest_timestamp_of_all_private_messages;
use crate::api::handler::request::message::private::__path_handle_get_latest_timestamp_with_user;
use crate::api::handler::request::message::private::__path_handle_get_latest_timestamps_of_all_private_chats;
use crate::api::handler::request::message::private::__path_handle_get_private_messages_after_timestamp;
use crate::api::handler::request::message::private::__path_handle_get_session_message;
use crate::api::handler::request::user::avatar::__path_handle_upload_avatar;
use crate::api::handler::request::user::contact::__path_handle_get_contact_list;
use crate::api::handler::request::user::contact::__path_handle_get_contact_timestamps;
use crate::api::handler::request::user::me::__path_handle_delete_me;
use crate::api::handler::request::user::me::__path_handle_get_me;
use crate::api::handler::request::user::me::__path_handle_patch_me;
use crate::api::handler::request::user::me::__path_handle_put_me;
use uchat_model::Empty;
use uchat_model::RoleType;
use uchat_model::UserDetailedInfo;
use uchat_model::UserSimpleInfo;
use uchat_model::request::RequestResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(ping, handle_request,
        handle_login, handle_logout, handle_register, handle_passwd, handle_connect, handle_check_session,
        handle_delete_me, handle_patch_me, handle_put_me, handle_get_me, handle_upload_avatar, handle_get_contact_timestamps, handle_get_contact_list,
        handle_info_friend, handle_add_friend, handle_list_friend, handle_list_friend_with_status, handle_get_status_by_userid,
        handle_creat_group, handle_info_group, handle_join_group, handle_leave_group, handle_list_group, handle_members_group,
        // 下面都是manager接口，即必须管理员权限才能访问的api
        // ----------------message----------------
        handle_get_group_message, handle_get_session_message,
        handle_get_all_private_messages_after_timestamp, handle_get_latest_timestamp_of_all_private_messages,
        handle_get_latest_timestamp_with_user, handle_get_latest_timestamps_of_all_private_chats,
        handle_get_private_messages_after_timestamp,
        handle_get_group_messages_after_timestamp, handle_get_all_group_messages_after_timestamp,
        handle_get_latest_timestamp_of_all_group_messages, handle_get_latest_timestamp_of_group,
        handle_get_latest_timestamps_of_all_groups,
        // ----------------user----------------
        handle_tree_online, handle_delete_session,
        handle_delete_message, handle_get_message, handle_get_recent_messages, handle_get_user_recent_messages,
        handle_user_get_count, handle_delete_user, handle_get_userinfo, handle_delete_friendship, handle_get_friends, handle_list_user, handle_change_role
    ),
    components(
        schemas(
            RequestResponse<String>,
            RequestResponse<Empty>,
            RequestResponse<UserSimpleInfo>,
            RequestResponse<UserDetailedInfo>,
            Empty,
            RoleType,
        )
    ),
    tags(
        (name = "测试接口", description = "仅用来测试对http的请求是否正常"),
        (name = "request/auth", description = "常规api：账号注册与身份认证"),
        (name = "request/friend", description = "常规api：好友"),
        (name = "request/group", description = "常规api：群组"),
        (name = "request/message", description = "常规api：聊天记录"),
        (name = "request/user", description = "常规api：个人信息"),
        (name = "manager/message", description = "后台管理：聊天记录"),
        (name = "manager/online", description = "后台管理：在线用户与会话"),
        (name = "manager/user", description = "后台管理：用户"),
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
