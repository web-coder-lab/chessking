use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AntiCheatError;

/// Doc 8 Sec1.2: "Example event severities (tune during implementation)."
/// The doc explicitly says these are tunable defaults, not fixed law -
/// stored as a Rust function (not hardcoded inline at every call site) so
/// tuning later means editing one table, not hunting through call sites.
pub fn severity_for(event_type: &str) -> i64 {
    match event_type {
        "wallet_mismatch" => 50,
        "fake_or_replayed_request" => 30,
        "invalid_or_tampered_jwt" => 20,
        "impossible_match_result_claim" => 40,
        "modified_client_payload_detected" => 100,
        "repeated_chargeback" => 70,
        "multi_account_same_device" => 25,
        "engine_move_similarity_high" => 35,
        "impossible_move_timing" => 25,
        "disconnect_pattern_losing_position" => 15,
        "ad_verification_failed" => 20,
        "bot_tool_detected_on_screen" => 40,
        "referral_fraud_pattern" => 30,
        // Events already wired in earlier phases with their own severity
        // at the call site (Doc 5 wallet audit, Doc 3 login lockout) map
        // here too, so every security_events row - regardless of which
        // module wrote it - participates in the same scoring system.
        "chargeback_deficit" => 70,
        "ledger_row_inconsistent" => 100,
        "login_failed" => 2,
        "2fa_retry" => 3,
        _ => 5, // unknown event types still count for something, minimally
    }
}

/// Doc 8 Sec1.1 score tiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskTier {
    Safe,      // 0-30
    Observe,   // 31-60
    Elevated,  // 61-80
    High,      // 81-100
}

pub fn tier_for_score(score: i64) -> RiskTier {
    match score {
        0..=30 => RiskTier::Safe,
        31..=60 => RiskTier::Observe,
        61..=80 => RiskTier::Elevated,
        _ => RiskTier::High,
    }
}

/// Doc 8 Sec1: "no single signal ever triggers a permanent, irreversible
/// action on its own." This function ONLY records the event and updates
/// the cumulative score - it never bans, restricts, or otherwise acts.
/// Escalation is entirely the job of ban_escalation::run_escalation_sweep,
/// which reads risk_scores independently on its own schedule.
pub async fn record_event(
    pool: &SqlitePool,
    user_id: &str,
    event_type: &str,
    metadata: serde_json::Value,
    ip_address: Option<&str>,
    device_fingerprint: Option<&str>,
) -> Result<i64, AntiCheatError> {
    let id = Uuid::new_v4().to_string();
    let severity = severity_for(event_type);
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO security_events (id, user_id, event_type, severity, metadata, ip_address, device_fingerprint, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(event_type)
    .bind(severity)
    .bind(metadata.to_string())
    .bind(ip_address)
    .bind(device_fingerprint)
    .bind(&now)
    .execute(pool)
    .await?;

    recalculate_score(pool, user_id).await
}

/// Doc 8 Sec1.1: recalculates risk_scores.score for one user from their
/// recent security_events, capped at 100. Sec1.2: "Scores decay slowly
/// over time for users who stop triggering new events (e.g. -5 per clean
/// week)" - implemented here as: sum severity from the last 90 days,
/// then apply a decay credit for each full clean week since the user's
/// last event.
async fn recalculate_score(pool: &SqlitePool, user_id: &str) -> Result<i64, AntiCheatError> {
    let window_start = (Utc::now() - Duration::days(90)).to_rfc3339();

    let raw: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(severity), 0) FROM security_events WHERE user_id = ? AND created_at > ?"
    )
    .bind(user_id)
    .bind(&window_start)
    .fetch_one(pool)
    .await?;

    let last_event_at: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM security_events WHERE user_id = ? AND severity > 0 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let decay = if let Some((last,)) = last_event_at {
        if let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(&last) {
            let clean_weeks = Utc::now().signed_duration_since(last_dt).num_weeks().max(0);
            clean_weeks * 5
        } else { 0 }
    } else { 0 };

    let score = (raw.0 - decay).clamp(0, 100);

    sqlx::query(
        "INSERT INTO risk_scores (id, user_id, score, category_breakdown, last_evaluated_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET score = excluded.score, last_evaluated_at = excluded.last_evaluated_at"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(score)
    .bind(serde_json::json!({ "raw_90d": raw.0, "decay_applied": decay }).to_string())
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(score)
}

pub async fn get_score(pool: &SqlitePool, user_id: &str) -> Result<i64, AntiCheatError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT score FROM risk_scores WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(s,)| s).unwrap_or(0))
}
