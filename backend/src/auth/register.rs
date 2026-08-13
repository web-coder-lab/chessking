use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AuthError;
use super::password::hash_password;
use super::validation::{validate_email, validate_password, validate_username};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub device_fingerprint: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub state: String, // "verify_email" — NOT logged in yet, per §2.3 step 9
    pub message: String,
}

const VERIFICATION_TOKEN_TTL_MINUTES: i64 = 15;

/// §2.3 order of operations, followed exactly:
/// 1-2. validate format
/// 3-4. check username availability
/// 5-6. check email availability, bail with specific error before any row is created
/// 7. hash password
/// 8. create user row (email_verified = 0)
/// 9. generate email_verification_tokens row (15 min expiry)
/// 10. send verification email
/// 11. return "verify your email" state — NOT logged in
pub async fn register(pool: &SqlitePool, req: RegisterRequest, email: &crate::email::EmailClient, frontend_base_url: &str) -> Result<RegisterResponse, AuthError> {
    // Steps 1-2: format validation (client + server; this IS the server half)
    validate_username(&req.username)?;
    validate_email(&req.email)?;
    validate_password(&req.password)?;

    let username_lower = req.username.to_lowercase();
    let email_lower = req.email.to_lowercase();

    // Steps 3-4: availability checks BEFORE any row is created — this is
    // deliberate (§2.3 rationale): prevents mass-registering popular
    // usernames and squatting on them unverified forever.
    let username_exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM users WHERE username_lower = ?"
    )
    .bind(&username_lower)
    .fetch_optional(pool)
    .await?;
    if username_exists.is_some() {
        return Err(AuthError::UsernameTaken);
    }

    let email_exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM users WHERE email = ?"
    )
    .bind(&email_lower)
    .fetch_optional(pool)
    .await?;
    if email_exists.is_some() {
        return Err(AuthError::EmailTaken);
    }

    // Step 5: hash password (Argon2id)
    let password_hash = hash_password(&req.password)?;

    // Step 6: create user row, email_verified = 0
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Doc 6 §2.3: user creation + default item grant (board/piece_set/
    // avatar/banner, pre-equipped) run as ONE transaction — an account
    // must never exist without its defaults, and vice versa.
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO users (id, username, username_lower, email, password_hash, email_verified, rating, coin_balance, role, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, 1200, 0, 'user', 'active', ?, ?)"
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&username_lower)
    .bind(&email_lower)
    .bind(&password_hash)
    .bind(&now)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    crate::shop::inventory::grant_default_items(&mut tx, &user_id).await?;

    tx.commit().await?;

    // Doc 8 §9/§1.2: multi_account_same_device check - this was fully
    // implemented in anticheat::device_fingerprint but never actually
    // called from anywhere. Registration is the point its own doc
    // comment says it belongs. Fire-and-forget (ignored result): a
    // risk-scoring signal should never block or fail account creation.
    if let Some(fp) = &req.device_fingerprint {
        let _ = crate::anticheat::device_fingerprint::check_multi_account_same_device(pool, &user_id, fp).await;
    }

    // Step 7: generate verification token, 15-minute expiry
    let token = issue_verification_token(pool, &user_id).await?;

    // Step 8: send verification email. Fire-and-forget - the account is
    // already created; a transient email failure shouldn't fail the
    // whole registration response, and resend-verification exists for
    // exactly this case.
    let _ = email.send_verification_email(&req.email, &token, frontend_base_url).await;

    // Step 9: NOT logged in — frontend shows "verify your email" state
    Ok(RegisterResponse {
        state: "verify_email".to_string(),
        message: "Check your email to verify your account.".to_string(),
    })
}

/// Generates a fresh single-use verification token and invalidates any
/// previous unused ones for this user (§2.5: "old unused tokens
/// invalidated when a new one is generated").
async fn issue_verification_token(pool: &SqlitePool, user_id: &str) -> Result<String, AuthError> {
    sqlx::query("UPDATE email_verification_tokens SET used = 1 WHERE user_id = ? AND used = 0")
        .bind(user_id)
        .execute(pool)
        .await?;

    let token_plain = super::jwt::generate_opaque_refresh_token(); // reuse: random opaque token
    let token_hash = super::jwt::hash_refresh_token(&token_plain);
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(VERIFICATION_TOKEN_TTL_MINUTES);

    sqlx::query(
        "INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at, used, created_at)
         VALUES (?, ?, ?, ?, 0, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(token_plain)
}

/// §2.3 steps 10-13: validates the emailed token, marks email verified,
/// marks token used, then AUTO-LOGS IN (issues tokens, creates session).
pub async fn verify_email(pool: &SqlitePool, token_plain: &str) -> Result<(String, bool), AuthError> {
    let token_hash = super::jwt::hash_refresh_token(token_plain);

    #[derive(sqlx::FromRow)]
    struct TokenRow { id: String, user_id: String, expires_at: String, used: i64, pending_email: Option<String> }

    let row = sqlx::query_as::<_, TokenRow>(
        "SELECT id, user_id, expires_at, used, pending_email FROM email_verification_tokens WHERE token_hash = ?"
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

    if let Some(new_email) = &row.pending_email {
        // §2 change-email flow: this link confirms a NEW address, not the
        // one already on the account - swap it now that it's proven
        // reachable. Re-checked here (not just at request time) in case
        // someone else took this exact email in the window between
        // request and confirm; the UNIQUE constraint on users.email is
        // the actual backstop either way.
        let update_result = sqlx::query("UPDATE users SET email = ?, email_verified = 1, updated_at = ? WHERE id = ?")
            .bind(new_email)
            .bind(Utc::now().to_rfc3339())
            .bind(&row.user_id)
            .execute(pool)
            .await;

        if let Err(sqlx::Error::Database(db_err)) = &update_result {
            if db_err.is_unique_violation() {
                return Err(AuthError::EmailTaken);
            }
        }
        update_result?;
    } else {
        sqlx::query("UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(&row.user_id)
            .execute(pool)
            .await?;
    }

    let was_email_change = row.pending_email.is_some();

    sqlx::query("UPDATE email_verification_tokens SET used = 1 WHERE id = ?")
        .bind(&row.id)
        .execute(pool)
        .await?;

    Ok((row.user_id, was_email_change)) // caller (route handler) creates the session + tokens
}

/// §2.5 Resend backoff policy — escalating waits, then "contact support".
/// `resend_count` is however many resends have already been sent for this
/// registration (tracked by counting non-first tokens, or a dedicated
/// counter column — using a simple in-table count here for clarity).
pub fn resend_backoff_seconds(resend_count: u32) -> Option<i64> {
    match resend_count {
        0 => Some(30),           // 1st resend: 30 seconds
        1 => Some(5 * 60),       // 2nd resend: 5 minutes
        2 => Some(5 * 60 * 60),  // 3rd resend: 5 hours
        3 => Some(24 * 60 * 60), // 4th resend: 24 hours
        _ => None,               // beyond 4th: "Contact support" instead of resend
    }
}

/// §2.5: handles a resend request — re-checks verified status, applies
/// backoff, issues a new token if allowed.
pub async fn resend_verification(pool: &SqlitePool, user_id: &str, last_sent_at: Option<chrono::DateTime<Utc>>, resend_count: u32, email: &crate::email::EmailClient, frontend_base_url: &str) -> Result<(), AuthError> {
    let user: (i64, String) = sqlx::query_as("SELECT email_verified, email FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if user.0 == 1 {
        return Err(AuthError::AlreadyVerified);
    }

    let Some(wait_secs) = resend_backoff_seconds(resend_count) else {
        return Err(AuthError::ResendLimitExceeded);
    };

    if let Some(last) = last_sent_at {
        let elapsed = Utc::now().signed_duration_since(last).num_seconds();
        if elapsed < wait_secs {
            return Err(AuthError::ResendTooSoon { retry_after_secs: wait_secs - elapsed });
        }
    }

    let token = issue_verification_token(pool, user_id).await?;
    let _ = email.send_verification_email(&user.1, &token, frontend_base_url).await;
    // Note: per-IP/per-device rate limiting and risk-based CAPTCHA (Doc 7)
    // are applied in middleware before this function is ever called.
    Ok(())
}
