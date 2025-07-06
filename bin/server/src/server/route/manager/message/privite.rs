use crate::api::handler::manager::message::privite;
use axum::{
    Router,
    routing::{delete, get},
};

pub fn router() -> Router {
    Router::new()
        .route("/recent", get(privite::handle_get_recent_messages))
        .route("/user", get(privite::handle_get_user_recent_messages))
        .route(
            "/",
            delete(privite::handle_delete_message).get(privite::handle_get_message),
        )
}
