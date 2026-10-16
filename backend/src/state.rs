use sqlx::postgres::PgPoolOptions;
use std::str::FromStr;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: crate::config::Config,
}

pub async fn init_pool(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    // ponytail: the :6543 transaction-mode pooler leaks prepared statements
    // across connections ("sqlx_s_1 already exists") even with the statement
    // cache off, so the app pool connects to the session pooler (:5432).
    // Migrations keep using DATABASE_URL (either port works there, CLI runs
    // one statement per connection).
    let app_url = if database_url.contains(":6543/") {
        database_url.replace(":6543/", ":5432/")
    } else {
        database_url.to_string()
    };
    init_pool_opts(&app_url, 0).await
}

/// `statement_cache_capacity = 0` disables prepared statements — required by
/// Supabase's :6543 transaction-mode pooler. Migrations use the CLI, which
/// has its own connection; the app pool keeps the cache off.
pub async fn init_pool_opts(
    database_url: &str,
    statement_cache_capacity: usize,
) -> anyhow::Result<sqlx::PgPool> {
    let opts = sqlx::postgres::PgConnectOptions::from_str(database_url)?
        .statement_cache_capacity(statement_cache_capacity);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(opts)
        .await?;
    Ok(pool)
}
