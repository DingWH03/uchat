use axum::{routing::{get}, Router};
use crate::api::handler;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handler::manager::dashboard::handle_admin_dashboard))
        
        
}
