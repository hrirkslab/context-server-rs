use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> &'static str {
    "OK"
}
