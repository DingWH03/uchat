mod group;
mod privite;
use axum::Router;

pub fn router() -> Router {
    Router::new().nest("/privite", privite::router())
    // .nest("/group", group::router())
}
