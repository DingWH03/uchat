use axum::{routing::{delete, get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/recent", get(handler::manager::message::handle_get_recent_messages))
        .route("/user", get(handler::manager::message::handle_get_user_recent_messages))
        .route("/", delete(handler::manager::message::handle_delete_message))
        
}
