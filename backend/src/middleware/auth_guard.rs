use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::AppState;
use crate::auth::errors::AuthError;
use crate::auth::jwt::{verify_access_token, AccessClaims};

/// §8 Protected Routes: "Every backend API endpoint independently
/// re-validates the JWT on every request via middleware." Frontend route
/// guards are UX-only and are never trusted — this is the real gate.
///
/// On success, injects `AccessClaims` into request extensions so handlers
/// can read `user_id` / `role` / `session_id` without re-parsing the token.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::Unauthorized)?;

    // §8: "If token missing, invalid, or expired → attempt silent
    // refresh; if that also fails → force redirect to Login." The silent
    // refresh itself is a frontend-initiated call to POST /auth/refresh
    // using the refresh token — this middleware's only job is to reject
    // (401) so the frontend knows to trigger that flow. It never performs
    // the refresh itself.
    let claims: AccessClaims = verify_access_token(token, &state.config.jwt_secret)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// Role-gate helper for admin-only routes (used on top of `require_auth`).
pub fn require_role(claims: &AccessClaims, allowed: &[&str]) -> Result<(), AuthError> {
    if allowed.contains(&claims.role.as_str()) {
        Ok(())
    } else {
        Err(AuthError::Unauthorized)
    }
}
