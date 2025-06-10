pub mod authentication;
pub mod friend;
pub mod group;
pub mod ws;
pub mod message;

use axum::Extension;
use crate::server::AppState;

pub async fn handle_request() -> &'static str {
    "Hello, world!"
}

pub async fn ping(Extension(state): Extension<AppState>) -> String {
    let request = state.request.lock().await;
    request.ping().await
}