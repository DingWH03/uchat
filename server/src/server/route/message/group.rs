use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handler::message::group_message::handle_get_group_message))
        
}
