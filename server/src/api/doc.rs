// server/src/api/doc.rs

use utoipa::{OpenApi};
use crate::api::handler::__path_ping;
use crate::api::handler::__path_handle_request;
use crate::protocol::request::ServerResponse;

#[derive(OpenApi)]
#[openapi(
    paths(ping, handle_request),
    components(schemas(ServerResponse)),
    tags(
        (name = "测试接口", description = "仅用来测试对http的请求是否正常")
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
