use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;

use crate::auth::CurrentUser;
use crate::error::ApiResult;
use crate::routes::staff::{require_page, require_permission};
use crate::state::AppState;

/// GET /api/settings — global ('both') settings keys.
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "system_settings")?;
    require_permission(&user, "system_settings", "manage_general")?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT option_name, value FROM settings WHERE brand_id = 'both' ORDER BY option_name",
    )
    .fetch_all(&state.pool)
    .await?;
    let map: serde_json::Map<String, serde_json::Value> = rows
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    Ok(Json(json!({"status":"true","settings":map})))
}

#[derive(Deserialize)]
pub struct SettingsBody {
    pub settings: std::collections::BTreeMap<String, String>,
}

/// PUT /api/settings — upsert global settings (system_settings.manage_general).
pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<SettingsBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_page(&user, "system_settings")?;
    require_permission(&user, "system_settings", "manage_general")?;
    for (k, v) in &body.settings {
        sqlx::query(
            "INSERT INTO settings (brand_id, option_name, value) VALUES ('both',$1,$2)
             ON CONFLICT (brand_id, option_name) DO UPDATE SET value = $2, updated_at = now()",
        )
        .bind(k)
        .bind(v)
        .execute(&state.pool)
        .await?;
    }
    Ok(Json(json!({"status":"true"})))
}
