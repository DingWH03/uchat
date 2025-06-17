mod group;
mod user;

use axum::{Router};

pub fn router() -> Router {
    Router::new()
        .nest("/group", group::router())
        .nest("/user", user::router())
        
}
