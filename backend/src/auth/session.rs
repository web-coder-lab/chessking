use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AuthError;
use super::jwt::{generate_opaque_refresh_token, hash_refresh_token};

pub struct NewSessionInput<'a> {
    pub user_id: &'a str,
    pub device_fingerprint: Option<&'a str>,
    pub ip_address: Option<&'a str>,
    pub browser: Option<&'a str>,
    pub os: Option<&'a str>,
}

pub struct IssuedSession {
    pub session_id: String,
    pub refresh_token_plain: String, // sent to client once, never stored
}

/// §5 online threshold: "no heartbeat/activity in the last 2 minutes" =
/// offline. Used across all three device-conflict cases (A/B/C).
const ONLINE_THRESHOLD_SECS: i64 = 120;

/// Creates a brand-new session row. Refresh token expiry is exactly 3 days
/// per §7.
pub async fn create_session(
    pool: &SqlitePool,
    input: NewSessionInput<'_>,
    gh: Option<&crate::db::GitHubStore>,
) -> Result<IssuedSession, AuthError> {
    let session_id = Uuid::new_v4().to_string();
    let refresh_plain = generate_opaque_refresh_token();
    let refresh_hash = hash_refresh_token(&refresh_plain);
    let now = Utc::now();
    let expires_at = now + Duration::days(3);

    sqlx::query(
        "INSERT INTO sessions (id, user_id, refresh_token_hash, previous_refresh_token_hash, device_fingerprint, ip_address, browser, os, is_active, last_seen_at, created_at, expires_at)
         VALUES (?, ?, ?, NULL, ?, ?, ?, ?, 1, ?, ?, ?)"
    )
    .bind(&session_id)
    .bind(input.user_id)
    .bind(&refresh_hash)
    .bind(input.device_fingerprint)
    .bind(input.ip_address)
    .bind(input.browser)
    .bind(input.os)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .execute(pool)
    .await?;

    // Part 2: durable session on GitHub
    if let Some(store) = gh {
        let gs = super::github_sessions::new_session(
            session_id.clone(),
            input.user_id,
            &refresh_hash,
            input.device_fingerprint,
        );
        super::github_sessions::save_session(store, &gs).await?;
    }

    Ok(IssuedSession { session_id, refresh_token_plain: refresh_plain })
}

#[derive(sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub user_id: String,
    pub refresh_token_hash: String,
    pub previous_refresh_token_hash: Option<String>,
    pub is_active: i64,
    pub last_seen_at: Option<String>,
    pub expires_at: String,
}

/// §5: is the device behind this session currently "online"?
pub fn is_session_online(session: &SessionRow) -> bool {
    let Some(last_seen) = &session.last_seen_at else { return false };
    let Ok(last_seen_dt) = chrono::DateTime::parse_from_rfc3339(last_seen) else { return false };
    let age = Utc::now().signed_duration_since(last_seen_dt.with_timezone(&Utc));
    age.num_seconds() < ONLINE_THRESHOLD_SECS
}

/// Finds the single active session for a user, if any (§5 branch input).
pub async fn find_active_session(pool: &SqlitePool, user_id: &str) -> Result<Option<SessionRow>, AuthError> {
    let row = sqlx::query_as::<_, SessionRow>(
        "SELECT id, user_id, refresh_token_hash, previous_refresh_token_hash, is_active, last_seen_at, expires_at
         FROM sessions WHERE user_id = ? AND is_active = 1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// §5 Case B step 3: "Old session row is immediately invalidated."
/// The old device only discovers this on its next action/refresh attempt
/// (no silent forced logout mid-use) — that check happens naturally in
/// `rotate_refresh_token` below, since an invalidated session's refresh
/// token no longer works.
pub async fn invalidate_session(pool: &SqlitePool, session_id: &str) -> Result<(), AuthError> {
    sqlx::query("UPDATE sessions SET is_active = 0 WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// §7 refresh token rotation + reuse detection:
/// - every refresh issues a new refresh token, invalidates the previous one
/// - if a rotated-out (already-used) token is presented again, that's a
///   stolen-token signal → kill the entire session chain, force re-login
pub async fn rotate_refresh_token(pool: &SqlitePool, presented_token: &str, gh: Option<&crate::db::GitHubStore>) -> Result<(String, IssuedSession), AuthError> {
    let presented_hash = hash_refresh_token(presented_token);

    // Matches on the CURRENT hash (normal case) OR the PREVIOUS one (the
    // token has been superseded by a later rotation already) - previously
    // this only matched the current hash, so a stale-but-superseded token
    // simply found no row at all and fell through to a generic
    // Unauthorized, indistinguishable from garbage input and with no
    // reuse detection, no risk-score signal, and no chain kill.
    let session = sqlx::query_as::<_, SessionRow>(
        "SELECT id, user_id, refresh_token_hash, previous_refresh_token_hash, is_active, last_seen_at, expires_at
         FROM sessions WHERE refresh_token_hash = ? OR previous_refresh_token_hash = ?"
    )
    .bind(&presented_hash)
    .bind(&presented_hash)
    .fetch_optional(pool)
    .await?;

    let session = if let Some(s) = session {
        s
    } else if let Some(store) = gh {
        // Part 2: after process restart SQL is empty — recover from GitHub
        let Some(gs) = super::github_sessions::find_by_refresh_hash(store, &presented_hash).await? else {
            return Err(AuthError::Unauthorized);
        };
        SessionRow {
            id: gs.id,
            user_id: gs.user_id,
            refresh_token_hash: gs.refresh_token_hash,
            previous_refresh_token_hash: gs.previous_refresh_token_hash,
            is_active: if gs.is_active { 1 } else { 0 },
            last_seen_at: gs.last_seen_at,
            expires_at: gs.expires_at,
        }
    } else {
        return Err(AuthError::Unauthorized);
    };

    let is_stale_generation_replay = session.refresh_token_hash != presented_hash
        && session.previous_refresh_token_hash.as_deref() == Some(presented_hash.as_str());

    if session.is_active == 0 || is_stale_generation_replay {
        // Doc 8 §2: "Refresh-token reuse detection ... treated as
        // invalid_or_tampered_jwt and invalidates the whole session chain
        // immediately (this one IS an immediate hard action, not just a
        // score add, because token theft is time-sensitive)." Two cases
        // land here:
        //  1. is_active == 0: an already-invalidated session's last token
        //     replayed (e.g. after Case B forced logout, or manual logout).
        //  2. is_stale_generation_replay: the CURRENT session already
        //     rotated past this exact token - someone else is holding a
        //     stale copy. The legitimate device is fine, but per the
        //     "time-sensitive, kill the chain" instruction we don't take
        //     the chance that a later token was compromised too.
        if session.is_active == 1 {
            invalidate_session(pool, &session.id).await?;
        }
        let _ = crate::anticheat::risk_score::record_event(
            pool, &session.user_id, "invalid_or_tampered_jwt",
            serde_json::json!({ "session_id": session.id, "reason": "refresh_token_reuse" }),
            None, None,
        ).await;
        tracing::warn!(session_id = %session.id, "refresh token reuse detected");
        return Err(AuthError::RefreshTokenReuseDetected);
    }

    let expires_at = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map_err(|_| AuthError::Internal)?;
    if Utc::now() > expires_at {
        invalidate_session(pool, &session.id).await?;
        return Err(AuthError::Unauthorized);
    }

    let new_refresh_plain = generate_opaque_refresh_token();
    let new_refresh_hash = hash_refresh_token(&new_refresh_plain);
    let now = Utc::now();

    sqlx::query("UPDATE sessions SET previous_refresh_token_hash = refresh_token_hash, refresh_token_hash = ?, last_seen_at = ? WHERE id = ?")
        .bind(&new_refresh_hash)
        .bind(now.to_rfc3339())
        .bind(&session.id)
        .execute(pool)
        .await?;

    if let Some(store) = gh {
        let gs = super::github_sessions::GhSession {
            id: session.id.clone(),
            user_id: session.user_id.clone(),
            refresh_token_hash: new_refresh_hash.clone(),
            previous_refresh_token_hash: Some(session.refresh_token_hash.clone()),
            device_fingerprint: None,
            is_active: true,
            created_at: session.expires_at.clone(), // best-effort; not critical
            expires_at: session.expires_at.clone(),
            last_seen_at: Some(now.to_rfc3339()),
        };
        let _ = super::github_sessions::save_session(store, &gs).await;
    }

    Ok((
        session.user_id.clone(),
        IssuedSession { session_id: session.id, refresh_token_plain: new_refresh_plain },
    ))
}

/// §7 Logout: destroy session row, revoke refresh token, log the event.
/// (Event logging into security_events/admin_audit_log wired in Doc 7 phase.)
pub async fn logout(pool: &SqlitePool, session_id: &str) -> Result<(), AuthError> {
    invalidate_session(pool, session_id).await
}

/// Heartbeat — frontend pings this periodically so `last_seen_at` reflects
/// true online status for the §5 online/offline branch logic.
pub async fn touch_session(pool: &SqlitePool, session_id: &str) -> Result<(), AuthError> {
    sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE id = ? AND is_active = 1")
        .bind(Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------
// Doc 9 Sec1: GET /auth/sessions, DELETE /auth/sessions/{id}
// ---------------------------------------------------------
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SessionListRow {
    pub id: String,
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub is_active: i64,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

/// Doc 9 Sec1: "GET /auth/sessions -> { sessions: [...] }" - lets the
/// user see every device that has an active session on their account.
pub async fn list_sessions(pool: &SqlitePool, user_id: &str) -> Result<Vec<SessionListRow>, AuthError> {
    let rows = sqlx::query_as::<_, SessionListRow>(
        "SELECT id, device_fingerprint, ip_address, browser, os, is_active, last_seen_at, created_at
         FROM sessions WHERE user_id = ? AND is_active = 1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Doc 9 Sec1: "DELETE /auth/sessions/{session_id} -> { status: 'revoked' }"
/// - lets the user remotely log out a specific device. Ownership is
/// checked (user_id = ?) so one user can never revoke another's session.
pub async fn revoke_session(pool: &SqlitePool, user_id: &str, session_id: &str) -> Result<(), AuthError> {
    let result = sqlx::query("UPDATE sessions SET is_active = 0 WHERE id = ? AND user_id = ?")
        .bind(session_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AuthError::Unauthorized);
    }
    Ok(())
}
