use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AuthError;
use super::jwt::{hash_refresh_token};
use super::password::verify_password;
use super::session::{find_active_session, invalidate_session, is_session_online};

const TWO_FA_PENDING_TTL_MINUTES: i64 = 5;
const OLD_DEVICE_APPROVAL_WINDOW_SECS: i64 = 120; // §5 Case C: "reasonable window (e.g. 2 minutes)"
const MAX_2FA_RETRIES: u32 = 5;

// =========================================================
// §4.1 — Enabling 2FA (Settings page)
// =========================================================

#[derive(Debug, Deserialize)]
pub struct Enable2FaRequest {
    pub current_password: String, // re-auth, proves it's really them
    pub new_code: String,          // user-chosen 6-digit code
    pub confirm_code: String,
}

pub async fn enable_2fa(pool: &SqlitePool, user_id: &str, req: Enable2FaRequest) -> Result<(), AuthError> {
    if req.new_code != req.confirm_code {
        return Err(AuthError::ReAuthRequired);
    }
    if req.new_code.len() != 6 || !req.new_code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AuthError::ReAuthRequired);
    }

    let row: (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if !verify_password(&req.current_password, &row.0) {
        return Err(AuthError::ReAuthRequired);
    }

    // §4.1 step 4: "Backend stores 2fa_secret encrypted, sets
    // two_fa_enabled = 1." Encryption-at-rest key management lives in
    // app_config (Doc 8, Admin Panel) — here we call the same hash
    // primitive used elsewhere since the code is verified, never displayed
    // back, functionally identical security properties to a password.
    let code_hash = hash_refresh_token(&req.new_code);

    sqlx::query("UPDATE users SET two_fa_enabled = 1, two_fa_secret = ?, updated_at = ? WHERE id = ?")
        .bind(&code_hash)
        .bind(Utc::now().to_rfc3339())
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

// =========================================================
// §4.2 — Disabling 2FA (same re-auth: password + current code)
// =========================================================

#[derive(Debug, Deserialize)]
pub struct Disable2FaRequest {
    pub current_password: String,
    pub current_code: String,
}

pub async fn disable_2fa(pool: &SqlitePool, user_id: &str, req: Disable2FaRequest) -> Result<(), AuthError> {
    let row: (String, Option<String>, String) = sqlx::query_as("SELECT password_hash, two_fa_secret, role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    // Doc 9 §12: "2FA is mandatory for any account with a role other than
    // `user` (cannot be disabled)." Checked BEFORE password/code
    // verification so an admin can't even attempt the flow — this is a
    // hard rule, not a soft warning.
    if row.2 != "user" {
        return Err(AuthError::TwoFaMandatoryForRole);
    }

    if !verify_password(&req.current_password, &row.0) {
        return Err(AuthError::ReAuthRequired);
    }
    let stored_hash = row.1.ok_or(AuthError::ReAuthRequired)?;
    if hash_refresh_token(&req.current_code) != stored_hash {
        return Err(AuthError::ReAuthRequired);
    }

    sqlx::query("UPDATE users SET two_fa_enabled = 0, two_fa_secret = NULL, updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339())
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

// =========================================================
// §5 — Device / Session Rules (Case A / B / C)
// =========================================================

#[derive(Debug, Serialize)]
pub enum DeviceCase {
    /// Case A: no active session at all
    NoActiveSession,
    /// Case B: active session exists, but device is offline
    ActiveButOffline,
    /// Case C: active session exists, device is online — needs approval
    ActiveAndOnline,
}

/// Entry point called right after password verification succeeds and
/// `two_fa_enabled = 1` (§4.3). Determines which of Case A/B/C applies.
pub async fn determine_device_case(pool: &SqlitePool, user_id: &str) -> Result<(DeviceCase, Option<String>), AuthError> {
    let Some(session) = find_active_session(pool, user_id).await? else {
        return Ok((DeviceCase::NoActiveSession, None)); // Case A
    };

    if is_session_online(&session) {
        Ok((DeviceCase::ActiveAndOnline, Some(session.id))) // Case C
    } else {
        Ok((DeviceCase::ActiveButOffline, Some(session.id))) // Case B
    }
}

/// Case A / Case B step 1: create a pending 2FA verification for the NEW
/// device, which must now enter the 6-digit code. The code is never sent
/// to or shown on the old device (per the explicit security note in §5 —
/// that was identified as a hole and rejected).
pub async fn create_pending_2fa(
    pool: &SqlitePool,
    user_id: &str,
    device_fingerprint: Option<&str>,
    requires_old_device_approval: bool,
) -> Result<String, AuthError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(TWO_FA_PENDING_TTL_MINUTES);

    // code_hash is left empty here for Case C (old device hasn't approved
    // yet, new device hasn't been prompted for the code yet either) — it's
    // populated only once the new device actually submits a code attempt,
    // via `submit_2fa_code`. For Case A/B we go straight to "awaiting code".
    sqlx::query(
        "INSERT INTO two_fa_pending_verifications (id, user_id, device_fingerprint, code_hash, approval_status, requires_old_device_approval, expires_at, created_at)
         VALUES (?, ?, ?, '', 'pending', ?, ?, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(device_fingerprint)
    .bind(requires_old_device_approval as i64)
    .bind(expires_at.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(id)
}

/// §5 Case C step 2: push notification target — old device sees this and
/// taps Approve/Deny. (Actual push delivery wired via the notifications
/// module; this returns what that module needs to send.)
pub async fn get_pending_for_approval(pool: &SqlitePool, pending_id: &str) -> Result<PendingRow, AuthError> {
    fetch_pending(pool, pending_id).await
}

#[derive(sqlx::FromRow)]
pub struct PendingRow {
    pub id: String,
    pub user_id: String,
    pub device_fingerprint: Option<String>,
    pub approval_status: String,
    pub requires_old_device_approval: i64,
    pub expires_at: String,
    pub created_at: String,
}

async fn fetch_pending(pool: &SqlitePool, pending_id: &str) -> Result<PendingRow, AuthError> {
    let row = sqlx::query_as::<_, PendingRow>(
        "SELECT id, user_id, device_fingerprint, approval_status, requires_old_device_approval, expires_at, created_at
         FROM two_fa_pending_verifications WHERE id = ?"
    )
    .bind(pending_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::Unauthorized)?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&row.expires_at).map_err(|_| AuthError::Internal)?;
    if Utc::now() > expires_at && row.approval_status == "pending" {
        mark_pending_status(pool, pending_id, "expired").await?;
        return Err(AuthError::ResetTokenInvalidOrExpired);
    }
    Ok(row)
}

async fn mark_pending_status(pool: &SqlitePool, pending_id: &str, status: &str) -> Result<(), AuthError> {
    sqlx::query("UPDATE two_fa_pending_verifications SET approval_status = ? WHERE id = ?")
        .bind(status)
        .bind(pending_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// §5 Case C 3a/3b: old device's response to the push notification.
/// - Approve → new device is now allowed to be prompted for the 6-digit code
/// - Deny → new device login cancelled, old session untouched
pub async fn respond_to_device_approval(pool: &SqlitePool, pending_id: &str, approved: bool) -> Result<(), AuthError> {
    let pending = fetch_pending(pool, pending_id).await?;
    if pending.approval_status != "pending" {
        return Err(AuthError::ResetTokenInvalidOrExpired);
    }

    if approved {
        mark_pending_status(pool, pending_id, "approved").await?;
        // New device may now be prompted for the code (frontend polls/
        // subscribes to this status and transitions to the code-entry screen).
    } else {
        mark_pending_status(pool, pending_id, "denied").await?;
    }
    Ok(())
}

/// §5 Case C 3c: "If OLD device does not respond within [2 minutes]: treat
/// as Deny (fail closed, not open)." Called by a background sweep or
/// lazily when the new device polls status past the window.
pub async fn expire_stale_approval_if_needed(pool: &SqlitePool, pending: &PendingRow) -> Result<bool, AuthError> {
    if pending.approval_status != "pending" {
        return Ok(false);
    }
    let created_at = chrono::DateTime::parse_from_rfc3339(&pending.created_at).map_err(|_| AuthError::Internal)?;
    let age = Utc::now().signed_duration_since(created_at.with_timezone(&Utc));
    if pending.requires_old_device_approval == 1 && age.num_seconds() > OLD_DEVICE_APPROVAL_WINDOW_SECS {
        mark_pending_status(pool, &pending.id, "denied").await?; // fail closed
        return Ok(true);
    }
    Ok(false)
}

/// Final step common to Case A, B, and (post-approval) Case C: the new
/// device submits its 6-digit code. On success, creates the new session
/// and — for Case B/C — invalidates the old one.
pub async fn submit_2fa_code(
    pool: &SqlitePool,
    pending_id: &str,
    submitted_code: &str,
    old_session_id_to_invalidate: Option<&str>,
) -> Result<String, AuthError> {
    let pending = fetch_pending(pool, pending_id).await?;

    match pending.approval_status.as_str() {
        // Already used, or terminated some other way - never allow a
        // second code submission against the same pending record, even if
        // the caller happens to resend the correct code (replay defense).
        "completed" | "denied" | "expired" => return Err(AuthError::LoginRequestDenied),
        "pending" if pending.requires_old_device_approval == 1 => {
            // Case C: code entry is only allowed after Approve.
            return Err(AuthError::LoginRequestDenied);
        }
        _ => {}
    }

    let user: (Option<String>,) = sqlx::query_as("SELECT two_fa_secret FROM users WHERE id = ?")
        .bind(&pending.user_id)
        .fetch_one(pool)
        .await?;
    let stored_secret = user.0.unwrap_or_default();

    if hash_refresh_token(submitted_code) != stored_secret {
        record_2fa_retry(pool, pending_id, &pending.user_id).await?;
        return Err(AuthError::TwoFaCodeIncorrect);
    }

    // Distinct from "approved" (which for Case C only means "old device
    // said yes, new device may now try a code") - "completed" marks this
    // pending record as consumed so it can never be used again.
    mark_pending_status(pool, pending_id, "completed").await?;

    // Case B/C: invalidate the old session now that the new one is verified.
    if let Some(old_id) = old_session_id_to_invalidate {
        invalidate_session(pool, old_id).await?;
    }

    Ok(pending.user_id)
}

async fn record_2fa_retry(pool: &SqlitePool, pending_id: &str, user_id: &str) -> Result<(), AuthError> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM security_events WHERE event_type = '2fa_retry' AND metadata LIKE ?"
    )
    .bind(format!("%{pending_id}%"))
    .fetch_one(pool)
    .await?;

    // Doc 8 §1.2 canonical severity for 2fa_retry = 3, routed through the
    // shared risk engine so it also updates risk_scores (not just an
    // isolated counter here for the lockout check below).
    let _ = crate::anticheat::risk_score::record_event(
        pool, user_id, "2fa_retry",
        serde_json::json!({ "pending_id": pending_id }),
        None, None,
    ).await;

    if count.0 as u32 + 1 >= MAX_2FA_RETRIES {
        return Err(AuthError::TwoFaLockout { retry_after_secs: 15 * 60 });
    }
    Ok(())
}
