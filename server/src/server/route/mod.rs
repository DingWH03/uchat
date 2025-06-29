// server/route/mod.rs
mod authentication;
mod friend;
mod group;
mod message;
mod manager;
mod user;

use axum::{routing::get, Router};
use crate::api::handler;


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
