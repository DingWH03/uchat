use crate::api::handler::manager::online;
use axum::{
    Router,
    routing::{delete, get},
};

pub fn router() -> Router {
    Router::new()
        .route("/tree", get(online::tree::handle_tree_online))
        .route("/session", delete(online::session::handle_delete_session))
}
