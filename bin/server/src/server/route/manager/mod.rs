mod message;
mod online;
mod user;
use crate::api::handler;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route(
            "/",
            get(handler::manager::dashboard::handle_admin_dashboard),
        )
        .nest("/user", user::router())
        .nest("/online", online::router())
        .nest("/message", message::router())
}
