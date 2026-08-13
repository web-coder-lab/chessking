use sqlx::SqlitePool;

use super::risk_score::record_event;

/// Doc 8 §9: "Used as a cross-check input across the wallet, referral,
/// and gifting fraud checks above — it's a shared signal, not a
/// standalone system." This module intentionally exposes only small,
/// composable check functions that OTHER modules call at the relevant
/// moment (registration, referral claim, gift send) — it never runs a
/// standalone sweep of its own, matching that framing exactly.

const REASONABLE_ACCOUNT_THRESHOLD: i64 = 2; // §9: "beyond a reasonable threshold, e.g. 2"

/// Called at registration time (and anywhere else that needs it) to
/// check how many OTHER accounts already share this device fingerprint.
/// Doc 8 §1.2: `multi_account_same_device` = +25 per additional account
/// beyond the threshold.
pub async fn check_multi_account_same_device(pool: &SqlitePool, user_id: &str, device_fingerprint: &str) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM sessions WHERE device_fingerprint = ? AND user_id != ?"
    )
    .bind(device_fingerprint)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if count.0 > REASONABLE_ACCOUNT_THRESHOLD {
        let extra = count.0 - REASONABLE_ACCOUNT_THRESHOLD;
        for _ in 0..extra {
            let _ = record_event(
                pool, user_id, "multi_account_same_device",
                serde_json::json!({ "device_fingerprint": device_fingerprint, "other_accounts_seen": count.0 }),
                None, Some(device_fingerprint),
            ).await;
        }
    }

    Ok(count.0)
}

/// Doc 8 §8 (Referral Shield), applied here since it's fundamentally a
/// device/IP correlation check: "Same-device or same-IP-cluster invite
/// chains ... → referral_fraud_pattern event, reward auto-withheld
/// pending review rather than paid out and clawed back later."
pub async fn referral_shares_device_or_ip(
    pool: &SqlitePool,
    inviter_id: &str,
    invited_id: &str,
) -> Result<bool, sqlx::Error> {
    let shared: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sessions s1
         JOIN sessions s2 ON (s1.device_fingerprint = s2.device_fingerprint AND s1.device_fingerprint IS NOT NULL)
                          OR (s1.ip_address = s2.ip_address AND s1.ip_address IS NOT NULL)
         WHERE s1.user_id = ? AND s2.user_id = ?"
    )
    .bind(inviter_id)
    .bind(invited_id)
    .fetch_one(pool)
    .await?;

    if shared.0 > 0 {
        let _ = record_event(
            pool, inviter_id, "referral_fraud_pattern",
            serde_json::json!({ "invited_id": invited_id, "shared_signal_count": shared.0 }),
            None, None,
        ).await;
    }

    Ok(shared.0 > 0)
}
