use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::auth::CurrentUser;
use crate::error::{ApiResult, AppError};
use crate::routes::staff::{require_page, require_permission};
use crate::state::AppState;

/// GET /api/domains — allowlist (domains.whitelist = view).
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "domains")?;
    require_permission(&user, "domains", "whitelist")?;
    let rows: Vec<(serde_json::Value,)> =
        sqlx::query_as("SELECT to_jsonb(d) FROM domains d ORDER BY id")
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(
        json!({"status":"true","domains":rows.into_iter().map(|(v,)| v).collect::<Vec<_>>()}),
    ))
}

#[derive(Deserialize)]
pub struct DomainBody {
    pub domain: String,
    pub status: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<DomainBody>,
) -> ApiResult<(axum::http::StatusCode, Json<serde_json::Value>)> {
    require_page(&user, "domains")?;
    require_permission(&user, "domains", "whitelist")?;
    let d = body.domain.trim().to_lowercase();
    if d.is_empty() || !d.contains('.') {
        return Err(AppError::BadRequest("a valid domain is required".into()));
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM domains WHERE domain = $1)")
        .bind(&d)
        .fetch_one(&state.pool)
        .await?;
    if exists {
        return Err(AppError::Conflict("domain already allowed".into()));
    }
    sqlx::query("INSERT INTO domains (domain, status) VALUES ($1, COALESCE($2,'active'))")
        .bind(&d)
        .bind(&body.status)
        .execute(&state.pool)
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({"status":"true"})),
    ))
}

/// PUT /api/domains/:id — activate/deactivate (domains.edit).
#[derive(Deserialize)]
pub struct DomainUpdate {
    pub status: String,
}

pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<DomainUpdate>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "domains")?;
    require_permission(&user, "domains", "edit")?;
    if body.status != "active" && body.status != "inactive" {
        return Err(AppError::BadRequest("invalid status".into()));
    }
    sqlx::query("UPDATE domains SET status = $1, updated_at = now() WHERE id = $2")
        .bind(&body.status)
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}

/// DELETE /api/domains/:id
pub async fn remove(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "domains")?;
    require_permission(&user, "domains", "delete")?;
    sqlx::query("DELETE FROM domains WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}
