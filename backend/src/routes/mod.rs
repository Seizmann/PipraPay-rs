pub mod auth;
pub mod setup;
pub mod staff;

use axum::routing::{get, post, put};
use axum::Router;

pub fn api_router(state: crate::state::AppState) -> Router<crate::state::AppState> {
    let authed = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/2fa", post(auth::twofa))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/totp-setup", get(auth::totp_setup))
        .route("/api/auth/totp-toggle", post(auth::totp_toggle))
        .route("/api/staff", get(staff::list).post(staff::create))
        .route("/api/staff/{id}", put(staff::update).delete(staff::remove))
        .route(
            "/api/staff/{id}/permissions",
            get(staff::get_permissions).put(staff::set_permissions),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            crate::auth::require_auth,
        ));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/setup", post(setup::setup))
        .route("/api/setup/status", get(setup::setup_status))
        .route("/api/auth/login", post(auth::login))
        .merge(authed)
}

async fn health() -> &'static str {
    "ok"
}
