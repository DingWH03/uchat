use axum::{routing::{delete, get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/tree", get(handler::manager::online::handle_tree_online))
        .route("/session", delete(handler::manager::online::handle_delete_session))
        
}
