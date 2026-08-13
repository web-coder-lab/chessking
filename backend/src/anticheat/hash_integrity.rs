use sqlx::SqlitePool;

use crate::wallet::ledger::compute_row_hash;
use super::risk_score::record_event;

#[derive(sqlx::FromRow)]
struct LedgerRow {
    id: String,
    user_id: String,
    r#type: String,
    amount: i64,
    balance_after: i64,
    prev_hash: Option<String>,
    row_hash: Option<String>,
    created_at: String,
}

/// Doc 8 §12: "A background integrity-check job periodically re-walks
/// the chain for each user ... and verifies every row_hash still matches
/// its recomputed value. If any historical row was edited directly in
/// the database ... the chain breaks at that point and every subsequent
/// hash fails to validate."
///
/// Returns the list of user_ids whose chain failed verification, for the
/// caller to raise the critical admin alert (§12: "this is treated as a
/// security incident, not a normal risk-score matter").
pub async fn verify_all_chains(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let user_ids: Vec<(String,)> = sqlx::query_as("SELECT DISTINCT user_id FROM wallet_logs")
        .fetch_all(pool)
        .await?;

    let mut broken_chains = Vec::new();

    for (user_id,) in user_ids {
        if !verify_user_chain(pool, &user_id).await? {
            broken_chains.push(user_id.clone());
            raise_critical_tamper_alert(pool, &user_id).await?;
        }
    }

    Ok(broken_chains)
}

async fn verify_user_chain(pool: &SqlitePool, user_id: &str) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query_as::<_, LedgerRow>(
        "SELECT id, user_id, type, amount, balance_after, prev_hash, row_hash, created_at
         FROM wallet_logs WHERE user_id = ? ORDER BY created_at ASC, id ASC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut expected_prev = "genesis".to_string();
    for row in rows {
        let Some(stored_prev) = &row.prev_hash else { continue }; // pre-hash-chain legacy rows, if any, are skipped rather than falsely flagged
        let Some(stored_hash) = &row.row_hash else { continue };

        if *stored_prev != expected_prev {
            return Ok(false); // chain link itself doesn't match — tampered or reordered
        }

        let recomputed = compute_row_hash(stored_prev, &row.user_id, &row.r#type, row.amount, row.balance_after, &row.created_at);
        if recomputed != *stored_hash {
            return Ok(false); // row content was altered after the fact
        }

        expected_prev = stored_hash.clone();
    }

    Ok(true)
}

/// Doc 8 §12: "raises a critical, highest-severity security_events row,
/// plus a direct admin alert." Severity 100 (the maximum defined tier,
/// same as `modified_client_payload_detected`) since ledger tampering is
/// the single most severe integrity violation this system can detect.
async fn raise_critical_tamper_alert(pool: &SqlitePool, user_id: &str) -> Result<(), sqlx::Error> {
    tracing::error!(user_id = %user_id, "CRITICAL: wallet_logs hash chain broken — possible direct database tampering");

    // record_event also recalculates risk_scores, but a broken hash chain
    // is explicitly NOT "a normal risk-score matter" per §12 — it's
    // logged through the same event table for a unified audit trail, but
    // the real response (freezing the account, direct admin paging) is
    // an Admin Panel / on-call concern (Doc 9), not something this
    // background job should decide unilaterally.
    let _ = record_event(
        pool,
        user_id,
        "ledger_row_inconsistent",
        serde_json::json!({ "detected_by": "hash_chain_integrity_sweep", "critical": true }),
        None,
        None,
    ).await;

    Ok(())
}

pub fn spawn_periodic_integrity_check(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
        loop {
            interval.tick().await;
            match verify_all_chains(&pool).await {
                Ok(broken) if !broken.is_empty() => {
                    tracing::error!(count = broken.len(), users = ?broken, "hash chain integrity sweep found tampering");
                }
                Ok(_) => {}
                Err(e) => tracing::error!("hash chain integrity sweep failed to run: {e:?}"),
            }
        }
    });
}
