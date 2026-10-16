pub mod setup;

use axum::{
    routing::{get, post},
    Router,
};

pub fn api_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/setup", post(setup::setup))
        .route("/api/setup/status", get(setup::setup_status))
}

async fn health() -> &'static str {
    "ok"
}
