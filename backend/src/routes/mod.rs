pub mod auth;
pub mod brands;
pub mod customers;
pub mod domains;
pub mod gateways;
pub mod settings;
pub mod setup;
pub mod staff;

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

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
        .route("/api/brands", get(brands::list).post(brands::create))
        .route(
            "/api/brands/{id}",
            get(brands::get).put(brands::update).delete(brands::remove),
        )
        .route(
            "/api/brands/{id}/gateways",
            get(gateways::list).post(gateways::create),
        )
        .route(
            "/api/gateways/{id}",
            put(gateways::update).delete(gateways::remove),
        )
        .route(
            "/api/brands/{id}/customers",
            get(customers::list).post(customers::create),
        )
        .route(
            "/api/customers/{ref}",
            put(customers::update).delete(customers::remove),
        )
        .route("/api/domains", get(domains::list).post(domains::create))
        .route(
            "/api/domains/{id}",
            put(domains::update).delete(domains::remove),
        )
        .route("/api/settings", get(settings::list).put(settings::update))
        .route_layer(middleware::from_fn_with_state(
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
