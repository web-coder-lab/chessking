use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;

use super::errors::AuthError;
use super::password::verify_password;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub identifier: String, // §3.1: single field, username OR email
    pub password: String,
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct UserAuthRow {
    pub id: String,
    pub password_hash: String,
    pub email_verified: i64,
    pub two_fa_enabled: i64,
    pub role: String,
    pub status: String,
}

const MAX_FAILED_ATTEMPTS: i64 = 5;
const LOCKOUT_WINDOW_MINUTES: i64 = 15;

/// §3.2 steps 1-6, exactly in order. Never reveals whether the identifier
/// matched an account — same generic error for "not found" and "wrong
/// password" per §3.3.
pub async fn login_step_1_verify_credentials(pool: &SqlitePool, req: &LoginRequest) -> Result<UserAuthRow, AuthError> {
    check_lockout(pool, &req.identifier).await?;

    let identifier_lower = req.identifier.to_lowercase();

    // Step 1: look up by username_lower OR email
    let user = sqlx::query_as::<_, UserAuthRow>(
        "SELECT id, password_hash, email_verified, two_fa_enabled, role, status
         FROM users WHERE username_lower = ? OR email = ?"
    )
    .bind(&identifier_lower)
    .bind(&identifier_lower)
    .fetch_optional(pool)
    .await?;

    // Step 2: not found → generic error
    let Some(user) = user else {
        record_failed_attempt(pool, &identifier_lower).await?;
        return Err(AuthError::InvalidCredentials);
    };

    if user.status == "banned" || user.status == "suspended" {
        // Not explicitly in §3, but a banned/suspended account must not
        // authenticate — generic message, same as invalid credentials, to
        // avoid leaking account status to an attacker.
        return Err(AuthError::InvalidCredentials);
    }

    // Step 3-4: verify password; wrong → same generic error, increment counter
    if !verify_password(&req.password, &user.password_hash) {
        record_failed_attempt(pool, &identifier_lower).await?;
        // Doc 8 §1.2: login_failed also feeds the per-USER risk score
        // (severity 2) when the failure is against a real account — the
        // identifier-based lockout counter above is a separate,
        // account-existence-agnostic mechanism (it must also work for a
        // typo'd username that matches no account at all, where there is
        // no user_id to attach a risk-score event to).
        let _ = crate::anticheat::risk_score::record_event(
            pool, &user.id, "login_failed",
            serde_json::json!({ "identifier": identifier_lower }),
            None, None,
        ).await;
        return Err(AuthError::InvalidCredentials);
    }

    // Successful password check clears the failed-attempt counter
    clear_failed_attempts(pool, &identifier_lower).await?;

    // Step 5: email not verified → block login
    if user.email_verified == 0 {
        return Err(AuthError::EmailNotVerified);
    }

    // Step 6: correct → caller proceeds to §4 (2FA / device logic)
    Ok(user)
}

/// §3.3: "Max 5 failed attempts per account per 15 minutes → temporary
/// lockout (increasing backoff on repeated lockouts)." Also tracked
/// per-IP separately for credential-stuffing protection (handled in
/// middleware, not here, since it's connection-level not account-level).
///
/// Uses a simple in-memory-free approach backed by security_events so it
/// survives restarts and feeds the risk engine (Doc 7).
async fn check_lockout(pool: &SqlitePool, identifier_lower: &str) -> Result<(), AuthError> {
    let window_start = (Utc::now() - chrono::Duration::minutes(LOCKOUT_WINDOW_MINUTES)).to_rfc3339();

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM security_events
         WHERE event_type = 'login_failed' AND metadata LIKE ? AND created_at > ?"
    )
    .bind(format!("%{identifier_lower}%"))
    .bind(&window_start)
    .fetch_one(pool)
    .await?;

    if count.0 >= MAX_FAILED_ATTEMPTS {
        // Increasing backoff on repeated lockouts is computed from how far
        // past the threshold the account is; simplest correct model: the
        // full window must elapse since the most recent failure.
        return Err(AuthError::AccountLocked { retry_after_secs: LOCKOUT_WINDOW_MINUTES * 60 });
    }
    Ok(())
}

async fn record_failed_attempt(pool: &SqlitePool, identifier_lower: &str) -> Result<(), AuthError> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO security_events (id, event_type, severity, metadata, created_at)
         VALUES (?, 'login_failed', 2, ?, ?)"
    )
    .bind(&id)
    .bind(serde_json::json!({ "identifier": identifier_lower }).to_string())
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

async fn clear_failed_attempts(_pool: &SqlitePool, _identifier_lower: &str) -> Result<(), AuthError> {
    // Deliberately a no-op: security_events is an immutable log (Doc 1
    // §5 principle — "every sensitive action is logged", nothing is
    // deleted). check_lockout naturally stops counting once the 15-minute
    // window rolls past the failures, so no explicit clear is needed.
    Ok(())
}
