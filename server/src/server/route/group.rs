use axum::{routing::{get, post}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/list", get(handler::group::list_group::handle_list_group))
        .route("/info", get(handler::group::info_group::handle_info_group))
        .route("/new", post(handler::group::creat_group::handle_creat_group))
        .route("/join", post(handler::group::join_group::handle_join_group))
        .route("/leave", post(handler::group::leave_group::handle_leave_group))
        .route("/members", get(handler::group::members_group::handle_members_group))
        
}
