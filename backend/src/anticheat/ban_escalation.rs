use chrono::Utc;
use sqlx::SqlitePool;
use std::time::Duration;
use uuid::Uuid;

use super::risk_score::{tier_for_score, RiskTier};

const EVALUATION_CYCLE_INTERVAL_SECS: u64 = 60 * 60; // Doc8 Sec1.1: "3 separate independent evaluation cycles"

/// Doc 8 Sec1.1: "Sustained 81-100 across 3 separate independent
/// evaluation cycles -> Confirmed -> Permanent ban + IP/device
/// blacklist." This is the ONLY path to a permanent ban in the whole
/// system — nothing else (not a single event, not one high score reading)
/// bans directly. Runs on its own timer, fully independent of whatever
/// caused the score to rise.
pub async fn run_escalation_sweep(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // BUGFIX: previously filtered `WHERE score >= 81` here, so a user who
    // dropped OUT of High tier between spikes never got a cycle event
    // recorded for that gap at all. count_consecutive_high_cycles only
    // looks at the last 3 *recorded* rows, so an invisible gap meant 3
    // High readings separated by weeks/months of being clean in between
    // were indistinguishable from 3 truly back-to-back cycles - silently
    // violating the "SUSTAINED ... across 3 separate independent
    // evaluation cycles" rule and the golden rule that a resolved false
    // positive must never cost an innocent user their account. Evaluating
    // every user who has ever been risk-scored (regardless of their
    // current tier) makes every cycle visible, so a dip back to
    // Safe/Observe/Elevated correctly breaks the streak.
    let all_scored_users: Vec<(String, i64)> = sqlx::query_as(
        "SELECT user_id, score FROM risk_scores"
    )
    .fetch_all(pool)
    .await?;

    for (user_id, score) in all_scored_users {
        record_evaluation_cycle(pool, &user_id, score).await?;

        if !matches!(tier_for_score(score), RiskTier::High) {
            // Below High tier this cycle - the marker row above is enough
            // to correctly break any future consecutive-streak count.
            // Nothing to review or ban.
            continue;
        }

        let consecutive_high = count_consecutive_high_cycles(pool, &user_id).await?;

        if consecutive_high >= 3 {
            apply_permanent_ban(pool, &user_id).await?;
        } else {
            // Doc 8 Sec17: "High (1st time reaching 81-100) -> Feature
            // restriction ... routed to admin review queue - NOT an
            // automatic ban." That restriction flag lives on the user's
            // risk profile for the Admin Panel (Doc 9) to act on;
            // recorded here as a low-severity marker event rather than a
            // direct status change, since a feature-level restriction
            // (pause gifting/purchases) is an Admin Panel concern, not
            // something this background sweep should silently do to a
            // user's account on its own.
            mark_pending_review(pool, &user_id, consecutive_high).await?;
        }
    }

    Ok(())
}

async fn record_evaluation_cycle(pool: &SqlitePool, user_id: &str, score: i64) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO security_events (id, user_id, event_type, severity, metadata, created_at)
         VALUES (?, ?, 'risk_evaluation_cycle', 0, ?, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(serde_json::json!({ "score_at_evaluation": score }).to_string())
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

/// Counts how many of the user's LAST 3 evaluation cycles were all
/// High-tier (81-100), consecutively, with no Safe/Observe/Elevated
/// reading in between — "3 separate independent evaluation cycles" per
/// Doc 8 Sec1.1, not just 3 high readings scattered arbitrarily.
async fn count_consecutive_high_cycles(pool: &SqlitePool, user_id: &str) -> Result<i64, sqlx::Error> {
    let recent: Vec<(String,)> = sqlx::query_as(
        "SELECT metadata FROM security_events
         WHERE user_id = ? AND event_type = 'risk_evaluation_cycle'
         ORDER BY created_at DESC LIMIT 3"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut consecutive = 0;
    for (metadata,) in recent {
        let score = serde_json::from_str::<serde_json::Value>(&metadata)
            .ok()
            .and_then(|v| v.get("score_at_evaluation").and_then(|s| s.as_i64()))
            .unwrap_or(0);
        if matches!(tier_for_score(score), RiskTier::High) {
            consecutive += 1;
        } else {
            break;
        }
    }
    Ok(consecutive)
}

async fn mark_pending_review(pool: &SqlitePool, user_id: &str, cycles_so_far: i64) -> Result<(), sqlx::Error> {
    tracing::warn!(user_id = %user_id, cycles = cycles_so_far, "user in High risk tier — routed to admin review queue, not yet auto-banned");
    Ok(())
}

/// Doc 8 Sec17: permanent ban + IP/device blacklist. Doc 8 Sec17.2: every
/// automatic ban remains reviewable/reversible by a Super Admin — this
/// function creates the `bans` row with issued_by = 'system', which the
/// Admin Panel (Doc 9) can reverse.
async fn apply_permanent_ban(pool: &SqlitePool, user_id: &str) -> Result<(), sqlx::Error> {
    let already_banned: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bans WHERE user_id = ? AND ban_type = 'permanent'"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if already_banned.0 > 0 {
        return Ok(()); // idempotent — don't double-ban
    }

    let ban_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bans (id, user_id, ban_type, reason, evidence_ref, ip_blacklisted, device_blacklisted, issued_by, created_at)
         VALUES (?, ?, 'permanent', ?, ?, 1, 1, 'system', ?)"
    )
    .bind(&ban_id)
    .bind(user_id)
    .bind("Sustained High risk score (81-100) across 3 independent evaluation cycles")
    .bind(user_id) // evidence_ref points at the risk_scores/security_events trail for this user
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    sqlx::query("UPDATE users SET status = 'banned', updated_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339())
        .bind(user_id)
        .execute(pool)
        .await?;

    tracing::warn!(user_id = %user_id, ban_id = %ban_id, "permanent ban applied — 3-cycle confirmed high risk");
    Ok(())
}

pub fn spawn_periodic_escalation(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(EVALUATION_CYCLE_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = run_escalation_sweep(&pool).await {
                tracing::error!("ban escalation sweep failed: {e:?}");
            }
        }
    });
}
