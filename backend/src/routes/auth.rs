use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::Cookies;

use crate::auth::{self, CurrentUser};
use crate::error::{ApiResult, AppError};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String, // email or username (PHP parity)
    pub password: String,
}

#[derive(Deserialize)]
pub struct TwoFaBody {
    pub code: String,
}

/// POST /api/auth/login
/// Response: {status:"true", target:"2fa"|"dashboard"} — PHP parity shape.
pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<LoginBody>,
) -> ApiResult<Json<serde_json::Value>> {
    if body.username.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".into(),
        ));
    }

    type UserRow = (
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
    );
    let row: Option<UserRow> = sqlx::query_as(
        "SELECT user_id, role, full_name, username, email, password_hash, temp_password_hash
         FROM users WHERE (email = $1 OR username = $1)",
    )
    .bind(&body.username)
    .fetch_optional(&state.pool)
    .await?;

    let (user_id, _role, _full_name, _username, _email, password_hash, temp_hash) = row
        .ok_or_else(|| {
            AppError::BadRequest("The email or password you entered is incorrect.".into())
        })?;

    let password_ok = bcrypt::verify(&body.password, &password_hash).unwrap_or(false)
        || temp_hash
            .as_deref()
            .map(|t| bcrypt::verify(&body.password, t).unwrap_or(false))
            .unwrap_or(false);
    if !password_ok {
        return Err(AppError::BadRequest(
            "The email or password you entered is incorrect.".into(),
        ));
    }

    let status: String = sqlx::query_scalar("SELECT status FROM users WHERE user_id = $1")
        .bind(&user_id)
        .fetch_one(&state.pool)
        .await?;
    if status != "active" {
        return Err(AppError::Forbidden);
    }

    let totp_status: String =
        sqlx::query_scalar("SELECT totp_status FROM users WHERE user_id = $1")
            .bind(&user_id)
            .fetch_one(&state.pool)
            .await?;

    let token = auth::generate_session_token();
    let token_hash = auth::hash_token(&token);

    // Lazily generate the TOTP secret on first login (PHP parity).
    let totp_secret: String =
        sqlx::query_scalar("SELECT totp_secret FROM users WHERE user_id = $1")
            .bind(&user_id)
            .fetch_one(&state.pool)
            .await?;
    if totp_secret == "--" || totp_secret.is_empty() {
        sqlx::query("UPDATE users SET totp_secret = $1, updated_at = now() WHERE user_id = $2")
            .bind(auth::totp::generate_secret())
            .bind(&user_id)
            .execute(&state.pool)
            .await?;
    }

    // Session row (browser/device/ip captured best-effort from headers by caller
    // middleware later; minimal fields for M2).
    sqlx::query(
        "INSERT INTO sessions (user_id, token_hash, browser, device, ip, status) VALUES ($1,$2,'--','--','--','active')",
    )
    .bind(&user_id)
    .bind(&token_hash)
    .execute(&state.pool)
    .await?;

    cookies.add(auth::build_cookie(&state.config, &token));

    let target = if totp_status == "enable" {
        "2fa"
    } else {
        "dashboard"
    };
    Ok(Json(
        json!({"status":"true","target":target,"session_token":token}),
    ))
}

/// POST /api/auth/2fa — verifies the TOTP code; marks the session 2fa-passed
/// by rotating the token (PHP re-sets cookies after logoutCookie()).
pub async fn twofa(
    State(state): State<AppState>,
    cookies: Cookies,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<TwoFaBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let secret: String = sqlx::query_scalar("SELECT totp_secret FROM users WHERE user_id = $1")
        .bind(&user.user_id)
        .fetch_one(&state.pool)
        .await?;

    if secret == "--" || secret.is_empty() {
        return Err(AppError::BadRequest("2FA is not configured".into()));
    }

    if !auth::totp::verify(&secret, &user.email, &body.code) {
        return Err(AppError::BadRequest(
            "The code you entered is incorrect. Please try again.".into(),
        ));
    }

    // Rotate session token (PHP logoutCookie + re-set).
    let new_token = auth::generate_session_token();
    sqlx::query("UPDATE sessions SET token_hash = $1, updated_at = now() WHERE id = $2")
        .bind(auth::hash_token(&new_token))
        .bind(user.session_id)
        .execute(&state.pool)
        .await?;
    cookies.add(auth::build_cookie(&state.config, &new_token));

    Ok(Json(json!({"status":"true","target":"dashboard"})))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query("UPDATE sessions SET status = 'expired', updated_at = now() WHERE id = $1")
        .bind(user.session_id)
        .execute(&state.pool)
        .await?;
    cookies.remove(auth::clear_cookie());
    Ok(Json(json!({"status":"true"})))
}

/// GET /api/auth/me — current user + permissions (SPA bootstrap).
pub async fn me(user: axum::Extension<CurrentUser>) -> ApiResult<Json<serde_json::Value>> {
    let perm = user
        .permissions
        .as_ref()
        .map(|p| json!(p))
        .unwrap_or(json!(null));
    Ok(Json(json!({
        "status": "true",
        "user_id": user.user_id,
        "role": user.role,
        "full_name": user.full_name,
        "username": user.username,
        "email": user.email,
        "brand_id": user.brand_id,
        "permissions": perm,
    })))
}

/// GET /api/auth/totp-setup — QR (SVG) + secret for enabling 2FA. Admin only.
pub async fn totp_setup(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
) -> ApiResult<Json<serde_json::Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let secret: String = sqlx::query_scalar("SELECT totp_secret FROM users WHERE user_id = $1")
        .bind(&user.user_id)
        .fetch_one(&state.pool)
        .await?;
    let secret = if secret == "--" || secret.is_empty() {
        let s = auth::totp::generate_secret();
        sqlx::query("UPDATE users SET totp_secret = $1, updated_at = now() WHERE user_id = $2")
            .bind(&s)
            .bind(&user.user_id)
            .execute(&state.pool)
            .await?;
        s
    } else {
        secret
    };
    let url = auth::totp::otpauth_url(&secret, &user.email, "PipraPay")
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("totp url")))?;
    let svg = crate::qr::svg(&url)?;
    Ok(Json(json!({"status":"true","secret":secret,"qr_svg":svg})))
}

/// POST /api/auth/totp-toggle — enable/disable 2FA, requires a valid current code.
#[derive(Deserialize)]
pub struct TotpToggleBody {
    pub enable: bool,
    pub code: String,
}

pub async fn totp_toggle(
    State(state): State<AppState>,
    user: axum::Extension<CurrentUser>,
    Json(body): Json<TotpToggleBody>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let secret: String = sqlx::query_scalar("SELECT totp_secret FROM users WHERE user_id = $1")
        .bind(&user.user_id)
        .fetch_one(&state.pool)
        .await?;
    if secret == "--" || secret.is_empty() {
        return Err(AppError::BadRequest("2FA is not configured".into()));
    }
    if !auth::totp::verify(&secret, &user.email, &body.code) {
        return Err(AppError::BadRequest(
            "The code you entered is incorrect. Please try again.".into(),
        ));
    }
    sqlx::query("UPDATE users SET totp_status = $1, updated_at = now() WHERE user_id = $2")
        .bind(if body.enable { "enable" } else { "disable" })
        .bind(&user.user_id)
        .execute(&state.pool)
        .await?;
    Ok((StatusCode::OK, Json(json!({"status":"true"}))))
}
