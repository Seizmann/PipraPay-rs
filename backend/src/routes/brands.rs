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

#[derive(Deserialize)]
pub struct CreateBrandBody {
    pub name: String,
    pub identify_name: Option<String>,
}

/// GET /api/brands — brands this user has permission rows for (admin: all).
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "brands")?;
    let rows: Vec<(String, String, Option<String>)> = if user.role == "admin" {
        sqlx::query_as("SELECT brand_id, identify_name, name FROM brands ORDER BY id")
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_as(
            "SELECT b.brand_id, b.identify_name, b.name FROM brands b
             JOIN permissions p ON p.brand_id = b.brand_id
             WHERE p.user_id = $1 AND p.status = 'active' ORDER BY b.id",
        )
        .bind(&user.user_id)
        .fetch_all(&state.pool)
        .await?
    };
    let brands: Vec<_> = rows.into_iter()
        .map(|(brand_id, identify_name, name)| json!({"brand_id": brand_id, "identify_name": identify_name, "name": name}))
        .collect();
    Ok(Json(json!({"status":"true","brands":brands})))
}

pub async fn create(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<CreateBrandBody>,
) -> ApiResult<(axum::http::StatusCode, Json<serde_json::Value>)> {
    require_permission(&user, "brands", "create")?;
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let brand_id = format!("b{}", &crate::auth::generate_session_token()[..16]);
    sqlx::query("INSERT INTO brands (brand_id, identify_name, name) VALUES ($1,$2,$3)")
        .bind(&brand_id)
        .bind(body.identify_name.unwrap_or_else(|| body.name.clone()))
        .bind(&body.name)
        .execute(&state.pool)
        .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({"status":"true","brand_id":brand_id})),
    ))
}

#[derive(Deserialize)]
pub struct UpdateBrandBody {
    pub name: Option<String>,
    pub identify_name: Option<String>,
    pub support_email_address: Option<String>,
    pub support_phone_number: Option<String>,
    pub support_website: Option<String>,
    pub whatsapp_number: Option<String>,
    pub telegram: Option<String>,
    pub timezone: Option<String>,
    pub payment_tolerance: Option<f64>,
    pub logo: Option<String>,
    pub notify_email_enabled: Option<bool>,
    pub notify_telegram_enabled: Option<bool>,
    pub notify_telegram_chat_id: Option<String>,
}

/// PUT /api/brands/:id — general settings (brand_settings.edit permission).
pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
    Json(body): Json<UpdateBrandBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "brand_settings")?;
    require_permission(&user, "brand_settings", "edit")?;
    ensure_brand_access(&state, &user, &brand_id).await?;

    sqlx::query(
        "UPDATE brands SET
            name = COALESCE($2, name),
            identify_name = COALESCE($3, identify_name),
            support_email_address = COALESCE($4, support_email_address),
            support_phone_number = COALESCE($5, support_phone_number),
            support_website = COALESCE($6, support_website),
            whatsapp_number = COALESCE($7, whatsapp_number),
            telegram = COALESCE($8, telegram),
            timezone = COALESCE($9, timezone),
            payment_tolerance = COALESCE($10, payment_tolerance),
            logo = COALESCE($11, logo),
            notify_email_enabled = COALESCE($12, notify_email_enabled),
            notify_telegram_enabled = COALESCE($13, notify_telegram_enabled),
            notify_telegram_chat_id = COALESCE($14, notify_telegram_chat_id),
            updated_at = now()
         WHERE brand_id = $1",
    )
    .bind(&brand_id)
    .bind(&body.name)
    .bind(&body.identify_name)
    .bind(&body.support_email_address)
    .bind(&body.support_phone_number)
    .bind(&body.support_website)
    .bind(&body.whatsapp_number)
    .bind(&body.telegram)
    .bind(&body.timezone)
    .bind(
        body.payment_tolerance
            .and_then(rust_decimal::Decimal::from_f64_retain),
    )
    .bind(&body.logo)
    .bind(body.notify_email_enabled)
    .bind(body.notify_telegram_enabled)
    .bind(&body.notify_telegram_chat_id)
    .execute(&state.pool)
    .await?;
    Ok(Json(json!({"status":"true"})))
}

/// Staff may only touch brands listed in their permission rows.
pub async fn ensure_brand_access(
    state: &AppState,
    user: &CurrentUser,
    brand_id: &str,
) -> Result<(), AppError> {
    if user.role == "admin" {
        return Ok(());
    }
    let ok: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM permissions WHERE user_id = $1 AND brand_id = $2 AND status = 'active')",
    )
    .bind(&user.user_id)
    .bind(brand_id)
    .fetch_one(&state.pool)
    .await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// DELETE /api/brands/:id
pub async fn remove(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "brands", "delete")?;
    sqlx::query("DELETE FROM brands WHERE brand_id = $1")
        .bind(&brand_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}

/// GET /api/brands/:id — full brand row for settings page.
pub async fn get(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "brand_settings")?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT to_jsonb(b) FROM brands b WHERE brand_id = $1")
            .bind(&brand_id)
            .fetch_optional(&state.pool)
            .await?;
    match row {
        Some((v,)) => Ok(Json(json!({"status":"true","brand":v}))),
        None => Err(AppError::NotFound),
    }
}
