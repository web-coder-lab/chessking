use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry_in_tx, LedgerEntryInput};
use super::errors::AntiCheatError;
use super::risk_score::record_event;

const DAILY_CAP: i64 = 10;
const COOLDOWN_MINUTES: i64 = 2;

#[derive(Debug, Deserialize)]
pub struct AdRewardCallback {
    pub user_id: String,
    pub ad_network_transaction_id: String,
}

/// Doc 8 §15, steps 1-5 exactly. This IS the S2S callback endpoint the ad
/// network (AdMob/Unity Ads/etc.) hits directly — the client's own
/// "I watched the ad" signal (step 1) is never trusted alone; only this
/// path can ever credit a coin.
pub async fn handle_ad_reward_callback(pool: &SqlitePool, cb: AdRewardCallback) -> Result<(), AntiCheatError> {
    let mut tx = pool.begin().await?;

    // Step 3a: duplicate-callback protection via unique
    // ad_network_transaction_id.
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM ad_views WHERE ad_network_transaction_id = ?"
    )
    .bind(&cb.ad_network_transaction_id)
    .fetch_optional(&mut *tx)
    .await?;
    if existing.is_some() {
        tracing::info!(txn = %cb.ad_network_transaction_id, "duplicate ad-reward callback ignored");
        return Ok(());
    }

    // Step 3b: daily cap.
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let today_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ad_views WHERE user_id = ? AND view_date = ? AND verified_server_side = 1"
    )
    .bind(&cb.user_id)
    .bind(&today)
    .fetch_one(&mut *tx)
    .await?;
    if today_count.0 >= DAILY_CAP {
        drop(tx);
        record_event(pool, &cb.user_id, "ad_verification_failed", serde_json::json!({ "reason": "daily_cap_exceeded" }), None, None).await?;
        return Ok(());
    }

    // Step 3c: cooldown, server-side timestamp only.
    let last: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM ad_views WHERE user_id = ? AND verified_server_side = 1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&cb.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((last_at,)) = last {
        if let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(&last_at) {
            let elapsed = Utc::now().signed_duration_since(last_dt);
            if elapsed < Duration::minutes(COOLDOWN_MINUTES) {
                drop(tx);
                record_event(pool, &cb.user_id, "ad_verification_failed", serde_json::json!({ "reason": "cooldown_violation" }), None, None).await?;
                return Ok(());
            }
        }
    }

    // Step 4: all checks passed — credit 1 coin via the shared ledger,
    // insert ad_views row marked verified_server_side = 1.
    apply_ledger_entry_in_tx(&mut tx, LedgerEntryInput {
        user_id: &cb.user_id,
        log_type: "ad_reward",
        amount: 1,
        reference_id: Some(&cb.ad_network_transaction_id),
        ip_address: None,
        device_fingerprint: None,
    }).await?;

    let id = Uuid::new_v4().to_string();
    let insert_result = sqlx::query(
        "INSERT INTO ad_views (id, user_id, ad_network_transaction_id, coins_awarded, verified_server_side, view_date, created_at)
         VALUES (?, ?, ?, 1, 1, ?, ?)"
    )
    .bind(&id)
    .bind(&cb.user_id)
    .bind(&cb.ad_network_transaction_id)
    .bind(&today)
    .bind(Utc::now().to_rfc3339())
    .execute(&mut *tx)
    .await;

    // The migration-0005 unique index is the hard backstop against a
    // race between the step-3a check above and this insert (two
    // concurrent callbacks with the same transaction id) — a constraint
    // violation here means we already lost the race harmlessly; the
    // coin credit above would need rolling back in that case, which is
    // exactly what NOT committing the transaction does.
    if let Err(sqlx::Error::Database(db_err)) = &insert_result {
        if db_err.is_unique_violation() {
            tracing::info!(txn = %cb.ad_network_transaction_id, "ad-reward race lost to a concurrent duplicate callback — rolled back");
            return Ok(());
        }
    }
    insert_result?;

    tx.commit().await?;
    Ok(())
}
