// server/route/mod.rs
mod authentication;
mod friend;
mod group;
mod manager;
mod message;
mod user;

use crate::api::handler;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/", get(handler::handle_request))
        .route("/ping", get(handler::ping))
        .nest("/auth", authentication::router())
        .nest("/friend", friend::router())
        .nest("/group", group::router())
        .nest("/message", message::router())
        .nest("/manager", manager::router())
        .nest("/user", user::router())
}
