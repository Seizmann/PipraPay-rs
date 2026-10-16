#![allow(dead_code)] // scaffolding: many pieces land in M2-M6
mod auth;
mod config;
mod error;
mod money;
mod permissions;
mod qr;
mod routes;
mod sms;
mod state;

use axum::Router;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load("SECRETS.md")?;
    tracing_subscriber_init();

    let pool = state::init_pool(&cfg.database_url).await?;
    // ponytail: Supabase :6543 transaction-mode pooler rejects sqlx's named
    // prepared statements, so migrations run via `sqlx migrate run` (CLI) —
    // startup only verifies the applied version matches the embedded set.
    let migrator = sqlx::migrate!("./migrations");
    let applied: Vec<(i64, bool)> = sqlx::query_as("SELECT version, success FROM _sqlx_migrations")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            anyhow::anyhow!("cannot read _sqlx_migrations (run `sqlx migrate run` first): {e}")
        })?;
    let latest = migrator.iter().map(|m| m.version).max().unwrap_or(0);
    let applied_versions: std::collections::HashSet<i64> =
        applied.iter().map(|(v, _)| *v).collect();
    let dirty = applied.iter().any(|(_, d)| !*d);
    if dirty
        || migrator
            .iter()
            .any(|m| !applied_versions.contains(&m.version))
    {
        anyhow::bail!("database migrations are behind or dirty — run `sqlx migrate run`");
    }
    let _ = latest;

    let app_state = state::AppState {
        pool,
        config: cfg.clone(),
    };

    // ponytail: dist resolved relative to the binary's project root; pass
    // PP_DIST_DIR to override when running from a different cwd.
    let dist_dir: PathBuf = std::env::var("PP_DIST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_e| {
            // backend/target/debug/piprapay-rs -> ../../frontend/dist
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/dist")
        });
    let index_html = dist_dir.join("index.html");
    // .fallback (not .not_found_service): the SPA's index.html must return 200.
    let spa = Router::new()
        .fallback_service(ServeDir::new(&dist_dir).fallback(ServeFile::new(index_html)));

    let app = routes::api_router(app_state.clone())
        .with_state(app_state)
        .layer(tower_cookies::CookieManagerLayer::new())
        .merge(spa);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("listening on 0.0.0.0:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

// ponytail: hand-rolled tracing init to avoid the tracing-subscriber dep;
// swap in tracing-subscriber if structured logs are ever needed.
fn tracing_subscriber_init() {
    // no-op for now; axum errors surface through AppError::Internal logging
}
