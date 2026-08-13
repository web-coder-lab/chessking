use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AuthError;
use super::jwt::{generate_opaque_refresh_token, hash_refresh_token};
use super::password::hash_password;
use super::validation::validate_password;

const RESET_TOKEN_TTL_MINUTES: i64 = 15;

/// §6 steps 1-3: always returns success regardless of whether the email
/// exists, to avoid leaking which emails are registered. The caller
/// (route handler) shows the same generic message either way:
/// "If this email is registered, a reset link has been sent."
pub async fn request_password_reset(pool: &SqlitePool, email: &str, email_client: &crate::email::EmailClient, frontend_base_url: &str) -> Result<(), AuthError> {
    let email_lower = email.to_lowercase();

    let user: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(&email_lower)
        .fetch_optional(pool)
        .await?;

    // Only do real work if the email exists — but never let that fact
    // leak through timing or response shape (caller returns the same
    // response either way).
    if let Some((user_id,)) = user {
        // Invalidate previous unused tokens, same pattern as email
        // verification tokens.
        sqlx::query("UPDATE password_reset_tokens SET used = 1 WHERE user_id = ? AND used = 0")
            .bind(&user_id)
            .execute(pool)
            .await?;

        let token_plain = generate_opaque_refresh_token();
        let token_hash = hash_refresh_token(&token_plain);
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(RESET_TOKEN_TTL_MINUTES);

        sqlx::query(
            "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, used, created_at)
             VALUES (?, ?, ?, ?, 0, ?)"
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&token_hash)
        .bind(expires_at.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        let _ = email_client.send_password_reset_email(email, &token_plain, frontend_base_url).await;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// §6 steps 6-9: validate token, validate new password strength, update
/// hash, mark token used, invalidate ALL sessions for this user (force
/// logout everywhere — protects against an attacker being locked out by
/// the real owner resetting the password).
pub async fn reset_password(pool: &SqlitePool, req: ResetPasswordRequest) -> Result<(), AuthError> {
    validate_password(&req.new_password)?;

    let token_hash = hash_refresh_token(&req.token);

    #[derive(sqlx::FromRow)]
    struct TokenRow { id: String, user_id: String, expires_at: String, used: i64 }

    let row = sqlx::query_as::<_, TokenRow>(
        "SELECT id, user_id, expires_at, used FROM password_reset_tokens WHERE token_hash = ?"
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::ResetTokenInvalidOrExpired)?;

    if row.used == 1 {
        return Err(AuthError::ResetTokenInvalidOrExpired);
    }
    let expires_at = chrono::DateTime::parse_from_rfc3339(&row.expires_at).map_err(|_| AuthError::Internal)?;
    if Utc::now() > expires_at {
        return Err(AuthError::ResetTokenInvalidOrExpired);
    }

    let new_hash = hash_password(&req.new_password)?;

    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(Utc::now().to_rfc3339())
        .bind(&row.user_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE password_reset_tokens SET used = 1 WHERE id = ?")
        .bind(&row.id)
        .execute(pool)
        .await?;

    // §6 step 8: force logout everywhere — invalidate every active session.
    sqlx::query("UPDATE sessions SET is_active = 0 WHERE user_id = ? AND is_active = 1")
        .bind(&row.user_id)
        .execute(pool)
        .await?;

    Ok(())
}
