// server/src/api/doc.rs

use crate::api::handler::__path_handle_request;
use crate::api::handler::__path_ping;
use crate::api::handler::manager::online::tree::__path_handle_tree_online;
use crate::api::handler::request::friend::info_friend::__path_handle_info_friend;
use crate::api::handler::request::user::me::__path_handle_delete_me;
use crate::api::handler::request::user::me::__path_handle_get_me;
use crate::api::handler::request::user::me::__path_handle_patch_me;
use crate::api::handler::request::user::me::__path_handle_put_me;
use crate::protocol::request::ServerResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(ping, handle_request, handle_delete_me, handle_patch_me, handle_put_me, handle_get_me, handle_info_friend, handle_tree_online),
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
