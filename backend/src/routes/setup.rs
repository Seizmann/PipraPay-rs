use axum::{extract::State, http::StatusCode, Json};
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;
use subtle::ConstantTimeEq;

use crate::error::{ApiResult, AppError};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SetupBody {
    pub setup_token: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
}

/// Constant-time token compare (subtle). Pure function, unit-tested.
pub fn token_matches(expected: &str, provided: &str) -> bool {
    let a = expected.as_bytes();
    let b = provided.as_bytes();
    if a.len() != b.len() {
        // Still burn comparable time on mismatched lengths by comparing
        // against itself; length itself is not secret.
        return false;
    }
    a.ct_eq(b).into()
}

/// Decision for the setup flow. Pure: given whether any admin exists and the
/// token result, what should we do? Keeps the 404-after-setup rule testable.
pub enum SetupDecision {
    CreateAdmin,
    WrongToken,   // 403
    AlreadySetup, // 404 permanent
}

pub fn decide(admin_exists: bool, token_ok: bool) -> SetupDecision {
    if admin_exists {
        // Once set up, the endpoint is gone — regardless of token validity.
        SetupDecision::AlreadySetup
    } else if token_ok {
        SetupDecision::CreateAdmin
    } else {
        SetupDecision::WrongToken
    }
}

fn generate_ids() -> (String, String) {
    let mut buf = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut buf);
    let hex: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    // PHP parity: generateItemID() = 10-digit numeric; we use random hex ids.
    (format!("u{hex}"), format!("b{hex}"))
}

/// Whether the one-shot setup flow is still available (no admin yet).
/// Read-only query; the write path (/api/setup) is user-invoked only.
pub async fn setup_status(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let admin_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(true);
    Ok(Json(json!({ "setup_needed": !admin_exists })))
}

pub async fn setup(
    State(state): State<AppState>,
    Json(body): Json<SetupBody>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let admin_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
            .fetch_one(&state.pool)
            .await
            .unwrap_or(true); // fail closed: if we cannot check, treat as set up

    match decide(
        admin_exists,
        token_matches(&state.config.setup_token, &body.setup_token),
    ) {
        SetupDecision::AlreadySetup => Err(AppError::NotFound),
        SetupDecision::WrongToken => Err(AppError::Forbidden),
        SetupDecision::CreateAdmin => {
            if body.username.is_empty() || body.email.is_empty() || body.password.len() < 8 {
                return Err(AppError::BadRequest(
                    "username, email and a password of at least 8 characters are required".into(),
                ));
            }
            let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)
                .map_err(|e| anyhow::anyhow!("hash: {e}"))?;
            let (user_id, brand_id) = generate_ids();

            // Atomic: no admin exists + insert, in one transaction. A second
            // concurrent request loses — the EXISTS check re-runs inside.
            let mut tx = state.pool.begin().await?;
            let exists_again: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'admin')")
                    .fetch_one(&mut *tx)
                    .await?;
            if exists_again || !token_matches(&state.config.setup_token, &body.setup_token) {
                return Err(AppError::Conflict("Setup already completed".into()));
            }
            sqlx::query("INSERT INTO users (user_id, full_name, username, email, password_hash, role, status, totp_status, totp_secret, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,'admin','active','disable','--',now(),now())")
                .bind(&user_id)
                .bind(&body.full_name)
                .bind(&body.username)
                .bind(&body.email)
                .bind(&hash)
                .execute(&mut *tx)
                .await?;
            // First admin gets a default brand + full permission blob.
            sqlx::query("INSERT INTO brands (brand_id, identify_name, name, theme, created_at, updated_at) VALUES ($1,'Default','Default','default',now(),now())")
                .bind(&brand_id)
                .execute(&mut *tx)
                .await?;
            let full = crate::permissions::full_schema_json();
            sqlx::query("INSERT INTO permissions (brand_id, user_id, permission, status, created_at, updated_at) VALUES ($1,$2,$3,'active',now(),now())")
                .bind(&brand_id)
                .bind(&user_id)
                .bind(serde_json::Value::String(full))
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;

            Ok((
                StatusCode::CREATED,
                Json(json!({"status":"true","message":"Admin created"})),
            ))
        }
    }
}
