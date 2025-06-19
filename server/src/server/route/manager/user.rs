use axum::{routing::{delete, get, post}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/count", get(handler::manager::user::handle_user_get_count))
        .route("/role", post(handler::manager::user::handle_change_role))
        .route("/list", get(handler::manager::user::handle_list_user))
        .route("/detail", get(handler::manager::user::handle_get_userinfo))
        .route("/friend", get(handler::manager::user::handle_get_friends)
                                        .delete(handler::manager::user::handle_delete_friendship))
        .route("/", delete(handler::manager::user::handle_delete_user))
        
}
