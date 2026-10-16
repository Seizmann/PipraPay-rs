use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::auth::CurrentUser;
use crate::error::{ApiResult, AppError};
use crate::permissions::{self, PermissionSet};
use crate::state::AppState;

/// Require a resource permission; admins bypass. Mirrors hasPermission().
pub fn require_permission(user: &CurrentUser, module: &str, action: &str) -> Result<(), AppError> {
    if permissions::has_permission(user.permissions.as_ref(), &user.role, module, action) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn require_page(user: &CurrentUser, page: &str) -> Result<(), AppError> {
    if permissions::can_access_page(user.permissions.as_ref(), &user.role, page) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[derive(Deserialize)]
pub struct CreateStaffBody {
    pub full_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub brand_ids: Vec<String>,
}

/// GET /api/staff — list staff.
pub async fn list(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "staff", "view_permission_list")?;
    let rows: Vec<(String, String, String, String, String)> =
        sqlx::query_as("SELECT user_id, full_name, username, email, role FROM users ORDER BY id")
            .fetch_all(&state.pool)
            .await?;
    let staff: Vec<_> = rows
        .into_iter()
        .map(|(id, full_name, username, email, role)| json!({
            "user_id": id, "full_name": full_name, "username": username, "email": email, "role": role,
        }))
        .collect();
    Ok(Json(json!({"status":"true","staff":staff})))
}

/// POST /api/staff — create staff with full permission schema (PHP default).
pub async fn create(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<CreateStaffBody>,
) -> ApiResult<(axum::http::StatusCode, Json<serde_json::Value>)> {
    require_permission(&user, "staff", "create")?;
    if body.password.len() < 8 || body.username.is_empty() || body.email.is_empty() {
        return Err(AppError::BadRequest(
            "username, email and a password of at least 8 characters are required".into(),
        ));
    }
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
        .map_err(|e| anyhow::anyhow!("hash: {e}"))?;
    let new_id = format!("u{}", &crate::auth::generate_session_token()[..16]);
    sqlx::query(
        "INSERT INTO users (user_id, full_name, username, email, password_hash, role, status, totp_status, totp_secret) VALUES ($1,$2,$3,$4,$5,'staff','active','disable','--')",
    )
    .bind(&new_id)
    .bind(&body.full_name)
    .bind(&body.username)
    .bind(&body.email)
    .bind(&hash)
    .execute(&state.pool)
    .await?;

    // Assign to brands (default: current brand), each with the full schema.
    let mut brand_ids = body.brand_ids.clone();
    if brand_ids.is_empty() && !user.brand_id.is_empty() {
        brand_ids.push(user.brand_id.clone());
    }
    let blob = permissions::full_schema_json();
    for b in &brand_ids {
        sqlx::query("INSERT INTO permissions (brand_id, user_id, permission) VALUES ($1,$2,$3) ON CONFLICT (brand_id, user_id) DO NOTHING")
            .bind(b)
            .bind(&new_id)
            .bind(serde_json::Value::String(blob.clone()))
            .execute(&state.pool)
            .await?;
    }
    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({"status":"true","user_id":new_id})),
    ))
}

/// GET /api/staff/:id/permissions
pub async fn get_permissions(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "staff", "view_permission_list")?;
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT brand_id, permission FROM permissions WHERE user_id = $1")
            .bind(&id)
            .fetch_all(&state.pool)
            .await?;
    let perms: Vec<_> = rows
        .into_iter()
        .map(|(brand_id, blob)| {
            let parsed = match &blob {
                serde_json::Value::String(s) => serde_json::from_str::<PermissionSet>(s).ok(),
                v => serde_json::from_value(v.clone()).ok(),
            };
            json!({"brand_id": brand_id, "permission": parsed})
        })
        .collect();
    Ok(Json(json!({"status":"true","permissions":perms})))
}

/// PUT /api/staff/:id/permissions — replace permission blob(s).
#[derive(Deserialize)]
pub struct SetPermissionsBody {
    /// Per-brand permission sets. Empty list clears all permissions.
    pub entries: Vec<PermissionEntry>,
}

#[derive(Deserialize)]
pub struct PermissionEntry {
    pub brand_id: String,
    pub permission: PermissionSet,
}

pub async fn set_permissions(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(body): Json<SetPermissionsBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "staff", "edit_permission")?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM permissions WHERE user_id = $1")
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    for e in &body.entries {
        sqlx::query("INSERT INTO permissions (brand_id, user_id, permission) VALUES ($1,$2,$3)")
            .bind(&e.brand_id)
            .bind(&id)
            .bind(serde_json::json!(e.permission))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"status":"true"})))
}

/// PUT /api/staff/:id — edit profile fields / suspend.
#[derive(Deserialize)]
pub struct UpdateStaffBody {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub password: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(body): Json<UpdateStaffBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "staff", "edit")?;
    if let Some(pw) = &body.password {
        if pw.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".into(),
            ));
        }
        let hash =
            bcrypt::hash(pw, bcrypt::DEFAULT_COST).map_err(|e| anyhow::anyhow!("hash: {e}"))?;
        sqlx::query("UPDATE users SET password_hash = $1 WHERE user_id = $2")
            .bind(&hash)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(name) = &body.full_name {
        sqlx::query("UPDATE users SET full_name = $1 WHERE user_id = $2")
            .bind(name)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(email) = &body.email {
        sqlx::query("UPDATE users SET email = $1 WHERE user_id = $2")
            .bind(email)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(status) = &body.status {
        if status != "active" && status != "suspend" {
            return Err(AppError::BadRequest("invalid status".into()));
        }
        // Suspending a staff user expires their sessions.
        sqlx::query("UPDATE users SET status = $1 WHERE user_id = $2")
            .bind(status)
            .bind(&id)
            .execute(&state.pool)
            .await?;
        if status == "suspend" {
            sqlx::query("UPDATE sessions SET status = 'expired' WHERE user_id = $1")
                .bind(&id)
                .execute(&state.pool)
                .await?;
        }
    }
    sqlx::query("UPDATE users SET updated_at = now() WHERE user_id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}

/// DELETE /api/staff/:id
pub async fn remove(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    require_permission(&user, "staff", "delete")?;
    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE user_id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;
    if role == "admin" {
        return Err(AppError::BadRequest(
            "Cannot delete an admin account".into(),
        ));
    }
    sqlx::query("DELETE FROM users WHERE user_id = $1 AND role = 'staff'")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"true"})))
}
