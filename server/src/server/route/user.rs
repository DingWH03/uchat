use crate::api::handler::request::user;
use axum::{routing::{get, post}, Router};

pub fn router() -> Router {
    Router::new().route(
        "/me",
        get(user::me::handle_get_me)
            .put(user::me::handle_put_me)
            .patch(user::me::handle_patch_me)
            .delete(user::me::handle_delete_me),
    )
    .route("/avatar", post(user::avatar::handle_upload_avatar))
    .route("/contact/timestamps", post(user::contact::handle_get_contact_timestamps))
    .route("/contact/list", post(user::contact::handle_get_contact_list))
}
