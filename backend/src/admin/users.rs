use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry, LedgerEntryInput};
use super::errors::AdminError;
use super::rbac::write_audit_log;
use super::reauth::require_reauth;

// ---------------------------------------------------------
// Sec4.1: Search & lookup
// ---------------------------------------------------------
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSearchRow {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
}

pub async fn search_users(pool: &SqlitePool, query: &str) -> Result<Vec<UserSearchRow>, AdminError> {
    let pattern = format!("%{}%", query.to_lowercase());
    let rows = sqlx::query_as::<_, UserSearchRow>(
        "SELECT id, username, email, role, status, created_at FROM users
         WHERE username_lower LIKE ? OR email LIKE ?
         ORDER BY created_at DESC LIMIT 50"
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub profile: UserProfileRow,
    pub wallet_balance: i64,
    pub wallet_logs: Vec<WalletLogRow>,
    pub match_history: Vec<MatchSummaryRow>,
    pub sessions: Vec<SessionRow>,
    pub risk_score: i64,
    pub security_events: Vec<SecurityEventRow>,
    pub referrals_made: Vec<ReferralRow>,
    pub referred_by: Option<ReferralRow>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserProfileRow {
    pub id: String, pub username: String, pub email: String, pub bio: Option<String>,
    pub country_code: Option<String>, pub role: String, pub status: String, pub created_at: String,
}
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WalletLogRow { pub id: String, pub r#type: String, pub amount: i64, pub balance_after: i64, pub created_at: String }
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MatchSummaryRow { pub id: String, pub match_type: String, pub result: Option<String>, pub started_at: String }
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionRow { pub id: String, pub device_fingerprint: Option<String>, pub ip_address: Option<String>, pub browser: Option<String>, pub created_at: String, pub is_active: i64 }
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SecurityEventRow { pub id: String, pub event_type: String, pub severity: i64, pub metadata: Option<String>, pub created_at: String }
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReferralRow { pub inviter_id: String, pub invited_id: String, pub reward_claimed: i64, pub created_at: String }

/// Doc 9 Sec4.1: full user detail view - every listed data source in one call.
pub async fn get_user_detail(pool: &SqlitePool, user_id: &str) -> Result<UserDetail, AdminError> {
    let profile = sqlx::query_as::<_, UserProfileRow>(
        "SELECT id, username, email, bio, country_code, role, status, created_at FROM users WHERE id = ?"
    ).bind(user_id).fetch_optional(pool).await?.ok_or(AdminError::NotFound)?;

    let balance: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?").bind(user_id).fetch_one(pool).await?;

    let wallet_logs = sqlx::query_as::<_, WalletLogRow>(
        "SELECT id, type, amount, balance_after, created_at FROM wallet_logs WHERE user_id = ? ORDER BY created_at DESC LIMIT 100"
    ).bind(user_id).fetch_all(pool).await?;

    let match_history = sqlx::query_as::<_, MatchSummaryRow>(
        "SELECT id, match_type, result, started_at FROM matches WHERE player_white_id = ? OR player_black_id = ? ORDER BY started_at DESC LIMIT 50"
    ).bind(user_id).bind(user_id).fetch_all(pool).await?;

    let sessions = sqlx::query_as::<_, SessionRow>(
        "SELECT id, device_fingerprint, ip_address, browser, created_at, is_active FROM sessions WHERE user_id = ? ORDER BY created_at DESC LIMIT 20"
    ).bind(user_id).fetch_all(pool).await?;

    let risk_score: (i64,) = sqlx::query_as("SELECT COALESCE((SELECT score FROM risk_scores WHERE user_id = ?), 0)")
        .bind(user_id).fetch_one(pool).await?;

    let security_events = sqlx::query_as::<_, SecurityEventRow>(
        "SELECT id, event_type, severity, metadata, created_at FROM security_events WHERE user_id = ? ORDER BY created_at DESC LIMIT 100"
    ).bind(user_id).fetch_all(pool).await?;

    let referrals_made = sqlx::query_as::<_, ReferralRow>(
        "SELECT inviter_id, invited_id, reward_claimed, created_at FROM referrals WHERE inviter_id = ?"
    ).bind(user_id).fetch_all(pool).await?;

    let referred_by = sqlx::query_as::<_, ReferralRow>(
        "SELECT inviter_id, invited_id, reward_claimed, created_at FROM referrals WHERE invited_id = ?"
    ).bind(user_id).fetch_optional(pool).await?;

    Ok(UserDetail {
        profile, wallet_balance: balance.0, wallet_logs, match_history, sessions,
        risk_score: risk_score.0, security_events, referrals_made, referred_by,
    })
}

// ---------------------------------------------------------
// Sec4.2: Actions
// ---------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SuspendRequest { pub reason: String, pub duration_days: Option<i64> }

pub async fn suspend_user(pool: &SqlitePool, admin_id: &str, target_id: &str, req: SuspendRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if req.reason.trim().is_empty() {
        return Err(AdminError::ValidationFailed("Reason is required.".to_string()));
    }
    let ban_id = Uuid::new_v4().to_string();
    let expires_at = req.duration_days.map(|d| (Utc::now() + chrono::Duration::days(d)).to_rfc3339());

    sqlx::query(
        "INSERT INTO bans (id, user_id, ban_type, reason, issued_by, expires_at, created_at)
         VALUES (?, ?, 'temporary', ?, ?, ?, ?)"
    )
    .bind(&ban_id).bind(target_id).bind(&req.reason).bind(admin_id).bind(&expires_at).bind(Utc::now().to_rfc3339())
    .execute(pool).await?;

    sqlx::query("UPDATE users SET status = 'suspended', updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;

    write_audit_log(pool, admin_id, "suspend_user",
        Some(serde_json::json!({"status":"active"})),
        Some(serde_json::json!({"status":"suspended","reason":req.reason,"expires_at":expires_at})),
        admin_ip).await
}

#[derive(Debug, Deserialize)]
pub struct BanRequest { pub reason: String }

pub async fn ban_user(pool: &SqlitePool, admin_id: &str, target_id: &str, req: BanRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if req.reason.trim().is_empty() {
        return Err(AdminError::ValidationFailed("Reason is required.".to_string()));
    }
    let ban_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bans (id, user_id, ban_type, reason, ip_blacklisted, device_blacklisted, issued_by, created_at)
         VALUES (?, ?, 'permanent', ?, 1, 1, ?, ?)"
    )
    .bind(&ban_id).bind(target_id).bind(&req.reason).bind(admin_id).bind(Utc::now().to_rfc3339())
    .execute(pool).await?;

    sqlx::query("UPDATE users SET status = 'banned', updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;

    write_audit_log(pool, admin_id, "ban_user",
        Some(serde_json::json!({"status":"active"})),
        Some(serde_json::json!({"status":"banned","reason":req.reason})),
        admin_ip).await
}

/// Doc 9 Sec8 / Doc8 Sec17.2: reversing a ban - restores the account, and
/// the admin panel is explicitly where this reversal lives.
pub async fn unban_user(pool: &SqlitePool, admin_id: &str, target_id: &str, admin_ip: Option<&str>) -> Result<(), AdminError> {
    sqlx::query("UPDATE bans SET expires_at = ? WHERE user_id = ? AND expires_at IS NULL")
        .bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;

    sqlx::query("UPDATE users SET status = 'active', updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;

    write_audit_log(pool, admin_id, "unban_user", None,
        Some(serde_json::json!({"status":"active"})), admin_ip).await
}

#[derive(Debug, Deserialize)]
pub struct AdjustRiskScoreRequest { pub adjustment: i64, pub justification: String }

/// Doc 9 Sec13: body { adjustment, justification } — a signed DELTA
/// applied to the current score, not an absolute overwrite. Clamped to
/// the valid 0-100 range same as the automatic scoring engine (Doc 8 Sec1).
pub async fn adjust_risk_score(pool: &SqlitePool, admin_id: &str, target_id: &str, req: AdjustRiskScoreRequest, admin_ip: Option<&str>) -> Result<i64, AdminError> {
    if req.justification.trim().is_empty() {
        return Err(AdminError::ValidationFailed("Justification is required.".to_string()));
    }

    let old: (i64,) = sqlx::query_as("SELECT COALESCE((SELECT score FROM risk_scores WHERE user_id = ?), 0)")
        .bind(target_id).fetch_one(pool).await?;
    let new_score = (old.0 + req.adjustment).clamp(0, 100);

    sqlx::query(
        "INSERT INTO risk_scores (id, user_id, score, category_breakdown, last_evaluated_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET score = excluded.score, last_evaluated_at = excluded.last_evaluated_at"
    )
    .bind(Uuid::new_v4().to_string()).bind(target_id).bind(new_score)
    .bind(serde_json::json!({"manual_admin_override": true, "justification": req.justification}).to_string())
    .bind(Utc::now().to_rfc3339())
    .execute(pool).await?;

    write_audit_log(pool, admin_id, "adjust_risk_score",
        Some(serde_json::json!({"score": old.0})),
        Some(serde_json::json!({"score": new_score, "adjustment": req.adjustment, "justification": req.justification})),
        admin_ip).await?;

    Ok(new_score)
}

#[derive(Debug, Deserialize)]
pub struct WalletAdjustmentRequest { pub current_password: String, pub amount: i64, pub justification: String }

/// Doc 9 Sec13: body { amount, justification, current_password }.
pub async fn adjust_wallet(pool: &SqlitePool, admin_id: &str, target_id: &str, req: WalletAdjustmentRequest, admin_ip: Option<&str>) -> Result<i64, AdminError> {
    require_reauth(pool, admin_id, &req.current_password).await?;

    if req.justification.trim().is_empty() {
        return Err(AdminError::ValidationFailed("Justification is required for a manual wallet adjustment.".to_string()));
    }

    let new_balance = apply_ledger_entry(pool, LedgerEntryInput {
        user_id: target_id,
        log_type: "admin_adjustment",
        amount: req.amount,
        reference_id: Some(admin_id),
        ip_address: admin_ip,
        device_fingerprint: None,
    }).await.map_err(|_| AdminError::Internal)?;

    write_audit_log(pool, admin_id, "manual_wallet_adjustment",
        None,
        Some(serde_json::json!({"target_user": target_id, "amount": req.amount, "justification": req.justification, "new_balance": new_balance})),
        admin_ip).await?;

    Ok(new_balance)
}

pub async fn force_logout_all_sessions(pool: &SqlitePool, admin_id: &str, target_id: &str, admin_ip: Option<&str>) -> Result<(), AdminError> {
    sqlx::query("UPDATE sessions SET is_active = 0 WHERE user_id = ? AND is_active = 1")
        .bind(target_id).execute(pool).await?;

    write_audit_log(pool, admin_id, "force_logout_all_sessions", None,
        Some(serde_json::json!({"target_user": target_id})), admin_ip).await
}

// ---------------------------------------------------------
// Doc 9 Sec2 / Sec12: role grant, super_admin only, mandatory-2FA enforced
// ---------------------------------------------------------
#[derive(Debug, Deserialize)]
pub struct GrantRoleRequest { pub current_password: String, pub role: String }

/// Doc 9 Sec12: role changes explicitly require re-authentication
/// immediately before the action executes.
pub async fn grant_role(pool: &SqlitePool, admin_id: &str, target_id: &str, req: GrantRoleRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    require_reauth(pool, admin_id, &req.current_password).await?;

    const VALID_ROLES: [&str; 5] = ["user", "super_admin", "security_admin", "finance_admin", "support_admin"];
    if !VALID_ROLES.contains(&req.role.as_str()) && req.role != "moderator" {
        return Err(AdminError::ValidationFailed("Unknown role.".to_string()));
    }

    let target: (String, i64) = sqlx::query_as("SELECT role, two_fa_enabled FROM users WHERE id = ?")
        .bind(target_id).fetch_optional(pool).await?.ok_or(AdminError::NotFound)?;

    // Doc 9 Sec12: "2FA is mandatory for any account with a role other
    // than `user` (cannot be disabled)." A role grant to anything other
    // than "user" is blocked until the target has already turned 2FA on
    // themselves - the admin panel cannot force-enable it on someone
    // else's behalf (that would require knowing their secret).
    if req.role != "user" && target.1 == 0 {
        return Err(AdminError::ValidationFailed(
            "This user must enable 2FA on their own account before an admin role can be granted.".to_string()
        ));
    }

    sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
        .bind(&req.role).bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;

    write_audit_log(pool, admin_id, "grant_role",
        Some(serde_json::json!({"role": target.0})),
        Some(serde_json::json!({"role": req.role, "target_user": target_id})),
        admin_ip).await
}
