use axum::{routing::{get, post}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/add", post(handler::request::friend::add_friend::handle_add_friend))
        .route("/list", get(handler::request::friend::list_friend::handle_list_friend))
        .route("/listv2", get(handler::request::friend::list_friend::handle_list_friend_with_status))
        .route("/info", get(handler::request::friend::info_friend::handle_info_friend))
        .route("/status", get(handler::request::friend::status_friend::handle_get_status_by_userid))
        
}
