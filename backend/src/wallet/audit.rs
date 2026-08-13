use sqlx::SqlitePool;

use super::errors::WalletError;

/// Doc 5 §6: "A background job runs periodically (e.g. every 5-15
/// minutes) per active user ... expected_balance = sum of all
/// wallet_logs.amount; actual_balance = users.coin_balance; if mismatch
/// → security_events, severity added to risk_scores, do NOT auto-ban —
/// flag for review only."
///
/// Called on a timer (see spawn_periodic below) — never blocks request
/// handlers, per the spec's "background job" framing.
pub async fn reconcile_all_wallets(pool: &SqlitePool) -> Result<usize, WalletError> {
    #[derive(sqlx::FromRow)]
    struct Mismatch {
        user_id: String,
        expected_balance: i64,
        actual_balance: i64,
    }

    let mismatches = sqlx::query_as::<_, Mismatch>(
        "SELECT u.id AS user_id,
                COALESCE(SUM(w.amount), 0) AS expected_balance,
                u.coin_balance AS actual_balance
         FROM users u
         LEFT JOIN wallet_logs w ON w.user_id = u.id AND w.status = 'success'
         GROUP BY u.id
         HAVING expected_balance != actual_balance"
    )
    .fetch_all(pool)
    .await?;

    for m in &mismatches {
        flag_wallet_mismatch(pool, &m.user_id, m.expected_balance, m.actual_balance).await?;
    }

    if !mismatches.is_empty() {
        tracing::warn!(count = mismatches.len(), "wallet audit found balance mismatches — flagged for review");
    }

    Ok(mismatches.len())
}

async fn flag_wallet_mismatch(pool: &SqlitePool, user_id: &str, expected: i64, actual: i64) -> Result<(), WalletError> {
    // Doc 8 §1.2 defines the canonical severity for this event type (50)
    // — routed through anticheat::risk_score::record_event so it also
    // updates risk_scores, not just security_events (the earlier ad-hoc
    // direct INSERT here, with an invented severity of 6, never touched
    // risk_scores at all — fixed now that the canonical scoring engine
    // exists).
    let _ = crate::anticheat::risk_score::record_event(
        pool, user_id, "wallet_mismatch",
        serde_json::json!({ "expected_balance": expected, "actual_balance": actual, "delta": actual - expected }),
        None, None,
    ).await;

    // §6 explicit: "do NOT auto-ban — flag for review only." No status
    // change to the user row happens here, deliberately.
    Ok(())
}

/// §6 additional fraud check: "Coins spent but balance didn't decrease"
/// — detects a wallet_logs row with a debit (negative amount) whose
/// balance_before/balance_after don't actually reflect the drop, which
/// would indicate a race condition or exploit in the purchase flow.
pub async fn detect_inconsistent_debit_rows(pool: &SqlitePool) -> Result<usize, WalletError> {
    #[derive(sqlx::FromRow)]
    struct BadRow { id: String, user_id: String }

    let bad_rows = sqlx::query_as::<_, BadRow>(
        "SELECT id, user_id FROM wallet_logs
         WHERE amount < 0 AND balance_after != balance_before + amount"
    )
    .fetch_all(pool)
    .await?;

    for row in &bad_rows {
        let _ = crate::anticheat::risk_score::record_event(
            pool, &row.user_id, "ledger_row_inconsistent",
            serde_json::json!({ "wallet_log_id": row.id }),
            None, None,
        ).await;
    }

    Ok(bad_rows.len())
}

/// Wires the periodic job. Doc 5 §6: "every 5-15 minutes" — using 10 as
/// the midpoint default; admin-configurable later via app_config if
/// needed (not specified as a config key in this doc, so left as a
/// constant here).
pub fn spawn_periodic_audit(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            if let Err(e) = reconcile_all_wallets(&pool).await {
                tracing::error!("wallet audit reconciliation failed: {e:?}");
            }
            if let Err(e) = detect_inconsistent_debit_rows(&pool).await {
                tracing::error!("wallet audit debit-consistency check failed: {e:?}");
            }
        }
    });
}
