use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tower_cookies::{Cookie, Cookies};

use crate::error::AppError;

pub const SESSION_COOKIE: &str = "pp_session";

/// Resolved current-user context inserted by `require_auth`.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: String,
    pub role: String, // admin | staff
    pub full_name: String,
    pub username: String,
    pub email: String,
    pub brand_id: String, // active brand (pp_brand parity)
    pub permissions: Option<crate::permissions::PermissionSet>,
    pub session_id: i64,
}

/// Create the session cookie for a token. `Secure` iff PUBLIC_BASE_URL is https.
pub fn build_cookie(config: &crate::config::Config, token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE, token.to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_secure(config.cookie_secure());
    cookie.set_max_age(tower_cookies::cookie::time::Duration::days(365));
    cookie.set_path("/");
    cookie
}

/// Clear the session cookie (logout).
pub fn clear_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::new(SESSION_COOKIE, "");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookie
}

// NOTE: DB lookups for the session live in `auth_guard` below; kept together
// with the cookie helpers so the auth flow reads top-to-bottom.

/// Middleware: require a valid session. Admin/staff both pass; role checks
/// happen per-handler via `CurrentUser`.
pub async fn require_auth(
    State(state): State<crate::state::AppState>,
    cookies: tower_cookies::Cookies,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    match load_current_user(&state, &cookies).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}

pub async fn load_current_user(
    state: &crate::state::AppState,
    cookies: &Cookies,
) -> Result<CurrentUser, AppError> {
    let token = cookies
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let token_hash = crate::auth::hash_token(&token);

    let row: Option<(String, String, String, String, String, i64)> = sqlx::query_as(
        "SELECT u.user_id, u.role, u.full_name, u.username, u.email, s.id
         FROM sessions s JOIN users u ON u.user_id = s.user_id
         WHERE s.token_hash = $1 AND s.status = 'active' AND u.status = 'active'",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await?;

    let (user_id, role, full_name, username, email, session_id) =
        row.ok_or(AppError::Unauthorized)?;

    // Active brand + permission blob for this user (PHP pp_brand cookie parity:
    // first active permission row wins).
    let brand_row: Option<(String, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT p.brand_id, p.permission FROM permissions p
         WHERE p.user_id = $1 AND p.status = 'active' LIMIT 1",
    )
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await?;

    let permissions = brand_row
        .as_ref()
        .and_then(|(_, blob)| blob.as_ref())
        .and_then(|v| {
            if let serde_json::Value::String(s) = v {
                crate::permissions::parse_json(s)
            } else {
                serde_json::from_value(v.clone()).ok()
            }
        });

    let brand_id = brand_row.map(|(b, _)| b).unwrap_or_default();

    Ok(CurrentUser {
        user_id,
        role,
        full_name,
        username,
        email,
        brand_id,
        permissions,
        session_id,
    })
}

/// Extract CurrentUser in a handler: `Extension<CurrentUser>`.
pub type CurrentUserExt = axum::Extension<CurrentUser>;

/// 401 JSON body for the SPA.
pub fn unauthorized_json() -> Response {
    (
        axum::http::StatusCode::UNAUTHORIZED,
        Json(json!({"status": "false", "message": "Unauthorized"})),
    )
        .into_response()
}
