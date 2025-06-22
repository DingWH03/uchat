// server/src/api/doc.rs

use crate::api::handler::__path_handle_request;
use crate::api::handler::__path_ping;
use crate::api::handler::manager::online::tree::__path_handle_tree_online;
use crate::api::handler::request::authentication::login::__path_handle_login;
use crate::api::handler::request::authentication::logout::__path_handle_logout;
use crate::api::handler::request::authentication::password::__path_handle_passwd;
use crate::api::handler::request::authentication::register::__path_handle_register;
use crate::api::handler::request::authentication::ws_connect::__path_handle_connect;
use crate::api::handler::request::authentication::session::__path_handle_check_session;
use crate::api::handler::request::friend::info_friend::__path_handle_info_friend;
use crate::api::handler::request::friend::add_friend::__path_handle_add_friend;
use crate::api::handler::request::friend::list_friend::__path_handle_list_friend;
use crate::api::handler::request::friend::list_friend::__path_handle_list_friend_with_status;
use crate::api::handler::request::group::creat_group::__path_handle_creat_group;
use crate::api::handler::request::group::info_group::__path_handle_info_group;
use crate::api::handler::request::group::join_group::__path_handle_join_group;
use crate::api::handler::request::group::leave_group::__path_handle_leave_group;
use crate::api::handler::request::group::list_group::__path_handle_list_group;
use crate::api::handler::request::group::members_group::__path_handle_members_group;
use crate::api::handler::request::user::me::__path_handle_delete_me;
use crate::api::handler::request::user::me::__path_handle_get_me;
use crate::api::handler::request::user::me::__path_handle_patch_me;
use crate::api::handler::request::user::me::__path_handle_put_me;
use crate::protocol::request::ServerResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(ping, handle_request, 
        handle_login, handle_logout, handle_register, handle_passwd, handle_connect, handle_check_session,
        handle_delete_me, handle_patch_me, handle_put_me, handle_get_me, 
        handle_info_friend, handle_add_friend, handle_list_friend, handle_list_friend_with_status,
        handle_creat_group, handle_info_group, handle_join_group, handle_leave_group, handle_list_group, handle_members_group,
        handle_tree_online
    ),
    components(schemas(ServerResponse)),
    tags(
        (name = "测试接口", description = "仅用来测试对http的请求是否正常"),
        (name = "manager", description = "后台管理请求api")
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
