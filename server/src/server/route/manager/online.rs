use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/tree", get(handler::manager::online::handle_tree_online))
        
}
