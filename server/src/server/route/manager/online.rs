use axum::{routing::{delete, get}, Router};
use crate::api::handler::manager::online;

pub fn router() -> Router {
    Router::new()
        .route("/tree", get(online::tree::handle_tree_online))
        .route("/session", delete(online::session::handle_delete_session))
        
}
