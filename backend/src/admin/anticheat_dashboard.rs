use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::anticheat::ban_escalation::run_escalation_sweep;
use super::errors::AdminError;
use super::rbac::write_audit_log;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RiskTierRow {
    pub user_id: String, pub username: String, pub score: i64, pub last_evaluated_at: String,
}

/// Doc 9 Sec16: "GET /admin/security/risk-queue | query: tier". `tier` is
/// optional - "elevated" (61-80), "high" (81-100), or omitted for both
/// (Doc 9 Sec8's "Elevated and High" default view).
pub async fn list_elevated_and_high_risk(pool: &SqlitePool, tier: Option<&str>) -> Result<Vec<RiskTierRow>, AdminError> {
    let (min, max) = match tier {
        Some("elevated") => (61, 80),
        Some("high") => (81, 100),
        _ => (61, 100),
    };
    let rows = sqlx::query_as::<_, RiskTierRow>(
        "SELECT r.user_id, u.username, r.score, r.last_evaluated_at
         FROM risk_scores r JOIN users u ON u.id = r.user_id
         WHERE r.score BETWEEN ? AND ?
         ORDER BY r.score DESC"
    )
    .bind(min).bind(max)
    .fetch_all(pool).await?;
    Ok(rows)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PendingReviewRow {
    pub user_id: String, pub username: String, pub score: i64, pub consecutive_high_cycles: i64,
}

/// Doc 9 Sec8: "Pending review queue: accounts that hit the High tier
/// and are awaiting the 3-cycle confirmation."
pub async fn pending_review_queue(pool: &SqlitePool) -> Result<Vec<PendingReviewRow>, AdminError> {
    let high_risk: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT r.user_id, u.username, r.score FROM risk_scores r JOIN users u ON u.id = r.user_id WHERE r.score >= 81"
    ).fetch_all(pool).await?;

    let mut out = Vec::new();
    for (user_id, username, score) in high_risk {
        let already_banned: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bans WHERE user_id = ? AND ban_type = 'permanent'")
            .bind(&user_id).fetch_one(pool).await?;
        if already_banned.0 > 0 { continue; }

        let cycles: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM (SELECT created_at FROM security_events WHERE user_id = ? AND event_type = 'risk_evaluation_cycle' ORDER BY created_at DESC LIMIT 3)"
        ).bind(&user_id).fetch_one(pool).await?;

        out.push(PendingReviewRow { user_id, username, score, consecutive_high_cycles: cycles.0 });
    }
    Ok(out)
}

/// Doc 9 Sec8: "Manual override: confirm-and-ban early, or dismiss/clear
/// a flag (with justification, logged)."
#[derive(Debug, Deserialize)]
pub struct ManualOverrideRequest { pub action: String, pub justification: String }

pub async fn manual_override(pool: &SqlitePool, admin_id: &str, target_id: &str, req: ManualOverrideRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if req.justification.trim().is_empty() {
        return Err(AdminError::ValidationFailed("Justification is required.".to_string()));
    }

    match req.action.as_str() {
        "confirm_ban" => {
            let ban_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO bans (id, user_id, ban_type, reason, evidence_ref, ip_blacklisted, device_blacklisted, issued_by, created_at)
                 VALUES (?, ?, 'permanent', ?, ?, 1, 1, ?, ?)"
            )
            .bind(&ban_id).bind(target_id)
            .bind(format!("Manual admin confirm-and-ban: {}", req.justification))
            .bind(target_id).bind(admin_id).bind(Utc::now().to_rfc3339())
            .execute(pool).await?;

            sqlx::query("UPDATE users SET status = 'banned', updated_at = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339()).bind(target_id).execute(pool).await?;
        }
        "dismiss" => {
            sqlx::query(
                "INSERT INTO risk_scores (id, user_id, score, category_breakdown, last_evaluated_at)
                 VALUES (?, ?, 0, ?, ?)
                 ON CONFLICT(user_id) DO UPDATE SET score = 0, last_evaluated_at = excluded.last_evaluated_at"
            )
            .bind(Uuid::new_v4().to_string()).bind(target_id)
            .bind(serde_json::json!({"manually_cleared": true, "justification": req.justification}).to_string())
            .bind(Utc::now().to_rfc3339())
            .execute(pool).await?;
        }
        _ => return Err(AdminError::ValidationFailed("action must be confirm_ban or dismiss".to_string())),
    }

    write_audit_log(pool, admin_id, "risk_manual_override",
        None,
        Some(serde_json::json!({"target_user": target_id, "action": req.action, "justification": req.justification})),
        admin_ip).await
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CheatDetectionLogRow {
    pub match_id: String, pub player_white_id: String, pub player_black_id: String,
    pub result_reason: Option<String>, pub ended_at: Option<String>,
}

/// Doc 9 Sec8: "Match-level cheat-detection log with full signal detail
/// for each case."
pub async fn cheat_detection_log(pool: &SqlitePool) -> Result<Vec<CheatDetectionLogRow>, AdminError> {
    let rows = sqlx::query_as::<_, CheatDetectionLogRow>(
        "SELECT id AS match_id, player_white_id, player_black_id, result_reason, ended_at
         FROM matches WHERE result_reason = 'cheat_detected' ORDER BY ended_at DESC LIMIT 100"
    ).fetch_all(pool).await?;
    Ok(rows)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BlacklistRow { pub id: String, pub user_id: String, pub reason: String, pub created_at: String }

/// Doc 9 Sec8: device/IP blacklist read off `bans` (no separate
/// blacklist table in Doc 1's schema; the ip_blacklisted/device_blacklisted
/// flags on bans ARE the blacklist source of truth).
pub async fn list_blacklist(pool: &SqlitePool) -> Result<Vec<BlacklistRow>, AdminError> {
    let rows = sqlx::query_as::<_, BlacklistRow>(
        "SELECT id, user_id, reason, created_at FROM bans
         WHERE ip_blacklisted = 1 OR device_blacklisted = 1
         ORDER BY created_at DESC"
    ).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn remove_from_blacklist(pool: &SqlitePool, admin_id: &str, ban_id: &str, admin_ip: Option<&str>) -> Result<(), AdminError> {
    sqlx::query("UPDATE bans SET ip_blacklisted = 0, device_blacklisted = 0 WHERE id = ?")
        .bind(ban_id).execute(pool).await?;
    write_audit_log(pool, admin_id, "remove_from_blacklist", None,
        Some(serde_json::json!({"ban_id": ban_id})), admin_ip).await
}

pub async fn trigger_escalation_sweep_now(pool: &SqlitePool) -> Result<(), AdminError> {
    run_escalation_sweep(pool).await.map_err(|_| AdminError::Internal)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SecurityEventRow { pub id: String, pub event_type: String, pub severity: i64, pub metadata: Option<String>, pub created_at: String }

/// Doc 9 Sec16: "GET /admin/security/events/{user_id} — { events: [...] }"
pub async fn security_events_for_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<SecurityEventRow>, AdminError> {
    let rows = sqlx::query_as::<_, SecurityEventRow>(
        "SELECT id, event_type, severity, metadata, created_at FROM security_events
         WHERE user_id = ? ORDER BY created_at DESC LIMIT 200"
    ).bind(user_id).fetch_all(pool).await?;
    Ok(rows)
}
