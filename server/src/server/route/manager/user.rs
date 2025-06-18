use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/count", get(handler::manager::user::handle_user_get_count))
        
}
