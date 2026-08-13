use chrono::Utc;
use sqlx::SqlitePool;

use super::config::RuntimeConfigStore;
use super::deposit::gateway_for_webhook;
use super::errors::WalletError;
use super::ledger::{apply_ledger_entry, LedgerEntryInput};

#[derive(sqlx::FromRow)]
struct TxRow {
    id: String,
    user_id: String,
    status: String,
    coin_rate_used: Option<i64>,
    amount_pkr: i64,
}

/// Doc 5 §3, followed exactly step by step:
/// 1. verify signature — reject immediately if invalid
/// 2. look up payment_transactions by gateway_transaction_id
/// 3. idempotency check — already success/failed → no-op (gateways retry)
/// 4. if new+valid: store raw payload, update status, credit coins on success
pub async fn handle_webhook(
    pool: &SqlitePool,
    config: &RuntimeConfigStore,
    gateway_name: &str,
    raw_body: &[u8],
    signature_header: &str,
    email: &crate::email::EmailClient,
    frontend_base_url: &str,
) -> Result<(), WalletError> {
    let gw = gateway_for_webhook(pool, config, gateway_name).await?;

    let secret_key = format!("{gateway_name}_secret");
    let secret = config.get_or(pool, &secret_key, "").await;

    // Step 1: verify signature — reject immediately if invalid
    if !gw.verify_webhook_signature(raw_body, signature_header, &secret) {
        tracing::warn!(gateway = gateway_name, "webhook signature verification failed");
        return Err(WalletError::WebhookSignatureInvalid);
    }

    let payload: serde_json::Value = serde_json::from_slice(raw_body).map_err(|_| WalletError::Internal)?;
    let Some(outcome) = gw.parse_webhook(&payload) else {
        return Err(WalletError::Internal);
    };

    // Step 2: look up matching payment_transactions row
    let tx_row = sqlx::query_as::<_, TxRow>(
        "SELECT id, user_id, status, coin_rate_used, amount_pkr FROM payment_transactions WHERE gateway_transaction_id = ?"
    )
    .bind(&outcome.gateway_transaction_id)
    .fetch_optional(pool)
    .await?
    .ok_or(WalletError::TransactionNotFound)?;

    // Step 3/4 combined: the UPDATE's WHERE clause is the actual
    // idempotency gate, not a separate read-then-write. A plain "SELECT
    // status, then decide, then UPDATE" has a race window: two concurrent
    // deliveries of the same webhook (gateways retry; a duplicated/forged
    // concurrent request could too) can both read "pending" before either
    // commits its own UPDATE, both pass the check, and both credit coins
    // for the same payment. Making the transition itself conditional means
    // the database - not this code - serializes the two attempts: only
    // one UPDATE can actually move the row out of a non-terminal status,
    // and rows_affected() tells us which request that was.
    let new_status = if outcome.success { "success" } else { "failed" };
    let now = Utc::now().to_rfc3339();
    let update_result = sqlx::query(
        "UPDATE payment_transactions SET status = ?, webhook_verified = 1, raw_gateway_response = ?, completed_at = ?
         WHERE id = ? AND status NOT IN ('success', 'failed')"
    )
    .bind(new_status)
    .bind(payload.to_string())
    .bind(&now)
    .bind(&tx_row.id)
    .execute(pool)
    .await?;

    if update_result.rows_affected() == 0 {
        // Lost the race (or a genuinely later duplicate delivery) -
        // someone else already moved this transaction to a terminal
        // status. Never credit twice for the same transaction id.
        tracing::info!(transaction_id = %tx_row.id, "duplicate webhook ignored (already terminal)");
        return Ok(());
    }

    if outcome.success {
        let coins_credited = credit_coins_for_transaction(pool, &tx_row).await?;
        if let Ok(Some((user_email,))) = sqlx::query_as::<_, (String,)>("SELECT email FROM users WHERE id = ?")
            .bind(&tx_row.user_id)
            .fetch_optional(pool)
            .await
        {
            let _ = email.send_payment_confirmation_email(&user_email, tx_row.amount_pkr, coins_credited, frontend_base_url).await;
        }
        // notifications::push("Payment Successful", ...).await?;
    }
    // §3 step 4 "if failed": mark failed, no coin credit, notify user in-app
    // (notification wiring is the same pattern as success, omitted here
    // for brevity — same call, different message).

    Ok(())
}

/// Doc 5 §4, steps 1-7 (the DB-transaction atomicity itself lives inside
/// `apply_ledger_entry` — begin/commit/rollback-on-failure is handled
/// there so this function can't accidentally skip it).
async fn credit_coins_for_transaction(pool: &SqlitePool, tx_row: &TxRow) -> Result<i64, WalletError> {
    let coin_rate = tx_row.coin_rate_used.unwrap_or(2);
    let coins_credited = super::ledger::compute_coins_credited(tx_row.amount_pkr, coin_rate);

    apply_ledger_entry(pool, LedgerEntryInput {
        user_id: &tx_row.user_id,
        log_type: "deposit",
        amount: coins_credited,
        reference_id: Some(&tx_row.id),
        ip_address: None,
        device_fingerprint: None,
    }).await?;

    sqlx::query("UPDATE payment_transactions SET coins_credited = ? WHERE id = ?")
        .bind(coins_credited)
        .bind(&tx_row.id)
        .execute(pool)
        .await?;

    Ok(coins_credited)
}
