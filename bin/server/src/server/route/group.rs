use crate::api::handler;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route(
            "/list",
            get(handler::request::group::list_group::handle_list_group),
        )
        .route(
            "/info",
            get(handler::request::group::info_group::handle_info_group),
        )
        .route(
            "/new",
            post(handler::request::group::creat_group::handle_creat_group),
        )
        .route(
            "/join",
            post(handler::request::group::join_group::handle_join_group),
        )
        .route(
            "/leave",
            post(handler::request::group::leave_group::handle_leave_group),
        )
        .route(
            "/members",
            get(handler::request::group::members_group::handle_members_group),
        )
}
