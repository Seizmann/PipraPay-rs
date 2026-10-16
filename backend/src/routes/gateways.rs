use axum::{
    extract::{Path, State},
    Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;

use crate::auth::CurrentUser;
use crate::error::{ApiResult, AppError};
use crate::routes::brands::ensure_brand_access;
use crate::routes::staff::{require_page, require_permission};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct GatewayBody {
    pub name: Option<String>,
    pub display: Option<String>,
    pub min_allow: Option<f64>,
    pub max_allow: Option<f64>,
    pub fixed_discount: Option<f64>,
    pub percentage_discount: Option<f64>,
    pub fixed_charge: Option<f64>,
    pub percentage_charge: Option<f64>,
    pub receive_number: Option<String>,
    pub instructions: Option<Vec<String>>,
    pub qr_image: Option<String>,
    pub primary_color: Option<String>,
    pub text_color: Option<String>,
    pub btn_color: Option<String>,
    pub btn_text_color: Option<String>,
    pub status: Option<String>,
}

#[allow(clippy::option_map_unit_fn)] // FromPrimitive trait fn
fn dec(v: Option<f64>) -> Option<Decimal> {
    Decimal::from_f64_retain(v?)
}

/// GET /api/brands/:brand_id/gateways
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "gateways")?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    let rows: Vec<(serde_json::Value,)> =
        sqlx::query_as("SELECT to_jsonb(g) FROM gateways g WHERE brand_id = $1 ORDER BY id")
            .bind(&brand_id)
            .fetch_all(&state.pool)
            .await?;
    Ok(Json(
        json!({"status":"true","gateways":rows.into_iter().map(|(v,)| v).collect::<Vec<_>>()}),
    ))
}

/// POST /api/brands/:brand_id/gateways — add a gateway to a brand.
pub async fn create(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(brand_id): Path<String>,
    Json(body): Json<GatewayBody>,
) -> ApiResult<(axum::http::StatusCode, Json<serde_json::Value>)> {
    require_permission(&user, "gateways", "create")?;
    ensure_brand_access(&state, &user, &brand_id).await?;
    if body.name.is_none() {
        return Err(AppError::BadRequest("name is required".into()));
    }
    let slug_default = body.name.as_ref().unwrap().to_lowercase().replace(' ', "-");
    let gateway_id = format!("g{}", &crate::auth::generate_session_token()[..16]);
    sqlx::query(
        "INSERT INTO gateways (gateway_id, brand_id, slug, name, display, min_allow, max_allow,
            fixed_discount, percentage_discount, fixed_charge, percentage_charge,
            receive_number, instructions, qr_image, primary_color, text_color, btn_color, btn_text_color, status)
         VALUES ($1,$2,$3,$4,COALESCE($5,$4),$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,COALESCE($19,'active'))",
    )
    .bind(&gateway_id)
    .bind(&brand_id)
    .bind(&slug_default)
    .bind(&body.name)
    .bind(&body.display)
    .bind(dec(body.min_allow))
    .bind(dec(body.max_allow))
    .bind(dec(body.fixed_discount))
    .bind(dec(body.percentage_discount))
    .bind(dec(body.fixed_charge))
    .bind(dec(body.percentage_charge))
    .bind(body.receive_number.as_deref().unwrap_or("--"))
    .bind(body.instructions.as_ref().map(|v| serde_json::json!(v)))
    .bind(&body.qr_image)
    .bind(&body.primary_color)
    .bind(&body.text_color)
    .bind(&body.btn_color)
    .bind(&body.btn_text_color)
    .bind(&body.status)
    .execute(&state.pool).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({"status":"true","gateway_id":gateway_id})),
    ))
}

/// PUT /api/gateways/:id
pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(gateway_id): Path<String>,
    Json(body): Json<GatewayBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "gateways", "edit")?;
    let brand_id: String =
        sqlx::query_scalar("SELECT brand_id FROM gateways WHERE gateway_id = $1")
            .bind(&gateway_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound)?;
    ensure_brand_access(&state, &user, &brand_id).await?;

    sqlx::query(
        "UPDATE gateways SET
            name = COALESCE($2, name),
            display = COALESCE($3, display),
            min_allow = COALESCE($4, min_allow),
            max_allow = COALESCE($5, max_allow),
            fixed_discount = COALESCE($6, fixed_discount),
            percentage_discount = COALESCE($7, percentage_discount),
            fixed_charge = COALESCE($8, fixed_charge),
            percentage_charge = COALESCE($9, percentage_charge),
            receive_number = COALESCE($10, receive_number),
            instructions = COALESCE($11, instructions),
            qr_image = COALESCE($12, qr_image),
            primary_color = COALESCE($13, primary_color),
            text_color = COALESCE($14, text_color),
            btn_color = COALESCE($15, btn_color),
            btn_text_color = COALESCE($16, btn_text_color),
            status = COALESCE($17, status),
            updated_at = now()
         WHERE gateway_id = $1",
    )
    .bind(&gateway_id)
    .bind(&body.name)
    .bind(&body.display)
    .bind(dec(body.min_allow))
    .bind(dec(body.max_allow))
    .bind(dec(body.fixed_discount))
    .bind(dec(body.percentage_discount))
    .bind(dec(body.fixed_charge))
    .bind(dec(body.percentage_charge))
    .bind(&body.receive_number)
    .bind(body.instructions.as_ref().map(|v| serde_json::json!(v)))
    .bind(&body.qr_image)
    .bind(&body.primary_color)
    .bind(&body.text_color)
    .bind(&body.btn_color)
    .bind(&body.btn_text_color)
    .bind(&body.status)
    .execute(&state.pool)
    .await?;
    Ok(Json(json!({"status":"true"})))
}

pub async fn remove(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(gateway_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "gateways", "delete")?;
    sqlx::query("DELETE FROM gateways WHERE gateway_id = $1")
        .bind(&gateway_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}
