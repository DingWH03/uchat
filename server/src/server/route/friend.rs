use axum::{routing::{get, post}, Router};
use crate::handler;

pub fn router() -> Router {
    Router::new()
        .route("/add", post(handler::friend::add_friend::handle_add_friend))
        .route("/list", get(handler::friend::list_friend::handle_list_friend))
        .route("/info", get(handler::friend::info_friend::handle_info_friend))
        
}
