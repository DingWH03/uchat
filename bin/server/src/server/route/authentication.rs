use crate::api::handler;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route(
            "/register",
            post(handler::request::authentication::register::handle_register),
        )
        .route(
            "/login",
            post(handler::request::authentication::login::handle_login),
        )
        .route(
            "/logout",
            post(handler::request::authentication::logout::handle_logout),
        )
        .route(
            "/ws",
            get(handler::request::authentication::ws_connect::handle_connect),
        )
        .route(
            "/password",
            post(handler::request::authentication::password::handle_passwd),
        )
        .route(
            "/check_session",
            get(handler::request::authentication::session::handle_check_session),
        )
}
