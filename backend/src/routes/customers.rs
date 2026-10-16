use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::auth::CurrentUser;
use crate::error::{ApiResult, AppError};
use crate::routes::brands::ensure_brand_access;
use crate::routes::staff::{require_page, require_permission};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CustomerBody {
    pub name: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
}

/// GET /api/brands/:brand_id/customers
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "customers")?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    let rows: Vec<(serde_json::Value,)> =
        sqlx::query_as("SELECT to_jsonb(c) FROM customers c WHERE brand_id = $1 ORDER BY id DESC")
            .bind(&brand_id)
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(
        json!({"status":"true","customers":rows.into_iter().map(|(v,)| v).collect::<Vec<_>>()}),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
    Json(body): Json<CustomerBody>,
) -> ApiResult<(axum::http::StatusCode, Json<serde_json::Value>)> {
    require_permission(&user, "customers", "create")?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let ref_id = crate::auth::generate_ref();
    sqlx::query("INSERT INTO customers (ref, brand_id, name, email, mobile) VALUES ($1,$2,$3,COALESCE($4,'--'),COALESCE($5,'--'))")
        .bind(&ref_id)
        .bind(&brand_id)
        .bind(&body.name)
        .bind(&body.email)
        .bind(&body.mobile)
        .execute(&state.pool).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({"status":"true","ref":ref_id})),
    ))
}

#[derive(Deserialize)]
pub struct UpdateCustomerBody {
    pub name: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub status: Option<String>,
    pub suspend_reason: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(ref_id): Path<String>,
    Json(body): Json<UpdateCustomerBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "customers", "edit")?;
    let brand_id: String = sqlx::query_scalar("SELECT brand_id FROM customers WHERE ref = $1")
        .bind(&ref_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    sqlx::query(
        "UPDATE customers SET
            name = COALESCE($2, name), email = COALESCE($3, email), mobile = COALESCE($4, mobile),
            status = COALESCE($5, status), suspend_reason = COALESCE($6, suspend_reason), updated_at = now()
         WHERE ref = $1",
    )
    .bind(&ref_id)
    .bind(&body.name).bind(&body.email).bind(&body.mobile)
    .bind(&body.status).bind(&body.suspend_reason)
    .execute(&state.pool).await?;
    Ok(Json(json!({"status":"true"})))
}

pub async fn remove(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(ref_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "customers", "delete")?;
    sqlx::query("DELETE FROM customers WHERE ref = $1")
        .bind(&ref_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}
