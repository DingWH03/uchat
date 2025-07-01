use axum::{routing::{get}, Router};
use crate::api::handler::request::message::group;

pub fn router() -> Router {
    Router::new()
        .route("/", get(group::handle_get_group_message))
        .route("/{group_id}/latest", get(group::handle_get_latest_timestamp_of_group))
        .route("/latest", get(group::handle_get_latest_timestamps_of_all_groups))
        .route("/max", get(group::handle_get_latest_timestamp_of_all_group_messages))
        .route("/{group_id}/after", get(group::handle_get_group_messages_after_timestamp))
        .route("/after", get(group::handle_get_all_group_messages_after_timestamp))
}
