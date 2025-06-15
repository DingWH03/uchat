use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/group", get(handler::message::group_message::handle_get_group_message))
        .route("/user", get(handler::message::session_message::handle_get_session_message))
        
}
