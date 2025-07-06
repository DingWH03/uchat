use crate::api::handler::request::message::private;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/", get(private::handle_get_session_message))
        .route(
            "/{friend_id}/latest",
            get(private::handle_get_latest_timestamp_with_user),
        )
        .route(
            "/latest",
            get(private::handle_get_latest_timestamps_of_all_private_chats),
        )
        .route(
            "/max",
            get(private::handle_get_latest_timestamp_of_all_private_messages),
        )
        .route(
            "/{friend_id}/after",
            get(private::handle_get_private_messages_after_timestamp),
        )
        .route(
            "/after",
            get(private::handle_get_all_private_messages_after_timestamp),
        )
}
