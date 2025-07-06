use crate::api::handler::manager::user;
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/count", get(user::count::handle_user_get_count))
        .route("/role", post(user::role::handle_change_role))
        .route("/list", get(user::list::handle_list_user))
        .route("/detail", get(user::detail::handle_get_userinfo))
        .route(
            "/friend",
            get(user::friend::handle_get_friends).delete(user::friend::handle_delete_friendship),
        )
        .route("/", delete(user::delete::handle_delete_user))
}
