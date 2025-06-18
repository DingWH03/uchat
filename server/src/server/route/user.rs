use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new().route(
        "/me",
        get(handler::request::user::me::handle_get_me)
            .put(handler::request::user::me::handle_put_me)
            .patch(handler::request::user::me::handle_patch_me)
            .delete(handler::request::user::me::handle_delete_me),
)

}
