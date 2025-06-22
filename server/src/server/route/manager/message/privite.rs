use axum::{routing::{delete, get}, Router};
use crate::api::handler::manager::message::privite;

pub fn router() -> Router {
    Router::new()
        .route("/recent", get(privite::handle_get_recent_messages))
        .route("/user", get(privite::handle_get_user_recent_messages))
        .route("/", delete(privite::handle_delete_message)
                                        .get(privite::handle_get_message))
        
}
