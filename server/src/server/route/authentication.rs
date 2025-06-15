use axum::{routing::{get, post}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/register", post(handler::authentication::register::handle_register))
        .route("/login", post(handler::authentication::login::handle_login))
        .route("/logout", post(handler::authentication::logout::handle_logout))
        .route("/ws", get(handler::authentication::ws_connect::handle_connect))
        .route("/password", post(handler::authentication::password::handle_passwd))
}
