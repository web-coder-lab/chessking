use sqlx::SqlitePool;

use super::errors::WalletError;
use super::ledger::{apply_ledger_entry, LedgerEntryInput};

#[derive(sqlx::FromRow)]
struct TxRow {
    id: String,
    user_id: String,
    coins_credited: Option<i64>,
}

/// Doc 5 §5, steps 1-5 exactly:
/// - locate original transaction, deduct the coin amount
/// - insert wallet_logs row type=refund
/// - if balance would go negative: ALLOW it (never floor at 0), flag
///   security_events "chargeback_deficit", route to admin review
pub async fn process_refund(pool: &SqlitePool, payment_transaction_id: &str) -> Result<(), WalletError> {
    let tx_row = sqlx::query_as::<_, TxRow>(
        "SELECT id, user_id, coins_credited FROM payment_transactions WHERE id = ?"
    )
    .bind(payment_transaction_id)
    .fetch_optional(pool)
    .await?
    .ok_or(WalletError::TransactionNotFound)?;

    let coins = tx_row.coins_credited.unwrap_or(0);
    if coins <= 0 {
        return Ok(()); // nothing was ever credited, nothing to claw back
    }

    let new_balance = apply_ledger_entry(pool, LedgerEntryInput {
        user_id: &tx_row.user_id,
        log_type: "refund",
        amount: -coins, // §5 step 4: amount = -coins
        reference_id: Some(&tx_row.id),
        ip_address: None,
        device_fingerprint: None,
    }).await?;

    if new_balance < 0 {
        // Doc 8 §1.2: chargeback_deficit canonical severity = 70, routed
        // through anticheat::risk_score::record_event so risk_scores
        // actually updates (the earlier ad-hoc direct INSERT with a
        // made-up severity of 8 never did). Still §5 step 5b/5c: never
        // any automatic punitive action (no auto-ban, no auto-suspend) —
        // record_event only ever raises the score, escalation is a
        // fully separate concern (Doc 8 §17, ban_escalation.rs).
        let _ = crate::anticheat::risk_score::record_event(
            pool, &tx_row.user_id, "chargeback_deficit",
            serde_json::json!({
                "payment_transaction_id": tx_row.id,
                "coins_clawed_back": coins,
                "resulting_balance": new_balance,
            }),
            None, None,
        ).await;

        tracing::warn!(user_id = %tx_row.user_id, resulting_balance = new_balance, "chargeback pushed balance negative — flagged for admin review, no automatic action taken");
    }

    Ok(())
}
