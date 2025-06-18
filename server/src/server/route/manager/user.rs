use axum::{routing::{get, post}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/count", get(handler::manager::user::handle_user_get_count))
        .route("/role", post(handler::manager::user::handle_change_role))
        
}
