use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{SqlitePool, Sqlite, Transaction};
use uuid::Uuid;

use super::errors::WalletError;

/// Doc 1 §6 / Doc 7: tamper-proof hash chain. Each wallet_logs row's
/// row_hash commits to the previous row's hash plus this row's own
/// content, so any retroactive edit to a past row breaks every hash after
/// it - detectable by the audit engine. pub(crate) so other modules that
/// write wallet_logs inside their OWN transaction (e.g. shop purchases,
/// gifts) can keep the same chain unbroken instead of bypassing it.
pub(crate) fn compute_row_hash(prev_hash: &str, user_id: &str, log_type: &str, amount: i64, balance_after: i64, created_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(user_id.as_bytes());
    hasher.update(log_type.as_bytes());
    hasher.update(amount.to_le_bytes());
    hasher.update(balance_after.to_le_bytes());
    hasher.update(created_at.as_bytes());
    hex::encode(hasher.finalize())
}

pub(crate) async fn last_row_hash(tx: &mut Transaction<'_, Sqlite>, user_id: &str) -> Result<String, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT row_hash FROM wallet_logs WHERE user_id = ? ORDER BY created_at DESC, id DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.and_then(|(h,)| h).unwrap_or_else(|| "genesis".to_string()))
}

pub struct LedgerEntryInput<'a> {
    pub user_id: &'a str,
    pub log_type: &'a str, // deposit / shop_purchase / gift_sent / daily_reward / ad_reward / referral_reward / admin_adjustment / refund
    pub amount: i64,        // positive = credit, negative = debit (Doc 1 §6)
    pub reference_id: Option<&'a str>,
    pub ip_address: Option<&'a str>,
    pub device_fingerprint: Option<&'a str>,
}

/// Transaction-scoped primitive: updates users.coin_balance and inserts a
/// hash-chained wallet_logs row, WITHOUT beginning or committing its own
/// transaction. Callers (this module's `apply_ledger_entry`, or
/// shop::purchase, shop::gifts) run this inside their own `tx` so the
/// balance update, ledger row, AND any other write (e.g. an inventory
/// insert) commit or roll back together as one atomic unit.
pub async fn apply_ledger_entry_in_tx(tx: &mut Transaction<'_, Sqlite>, input: LedgerEntryInput<'_>) -> Result<i64, sqlx::Error> {
    // Doc 5 §5 refund rule: negative balances are ALLOWED (never silently
    // floored at 0). Callers that must block overspend (e.g. shop
    // purchase) check balance BEFORE calling this, not inside it.
    let (balance_after,): (i64,) = sqlx::query_as(
        "UPDATE users SET coin_balance = coin_balance + ?, updated_at = ?
         WHERE id = ?
         RETURNING coin_balance"
    )
    .bind(input.amount)
    .bind(Utc::now().to_rfc3339())
    .bind(input.user_id)
    .fetch_one(&mut **tx)
    .await?;
    let balance_before = balance_after - input.amount;

    let prev_hash = last_row_hash(tx, input.user_id).await?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let row_hash = compute_row_hash(&prev_hash, input.user_id, input.log_type, input.amount, balance_after, &now);

    sqlx::query(
        "INSERT INTO wallet_logs (id, user_id, type, amount, balance_before, balance_after, reference_id, ip_address, device_fingerprint, status, prev_hash, row_hash, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'success', ?, ?, ?)"
    )
    .bind(&id)
    .bind(input.user_id)
    .bind(input.log_type)
    .bind(input.amount)
    .bind(balance_before)
    .bind(balance_after)
    .bind(input.reference_id)
    .bind(input.ip_address)
    .bind(input.device_fingerprint)
    .bind(&prev_hash)
    .bind(&row_hash)
    .bind(&now)
    .execute(&mut **tx)
    .await?;

    Ok(balance_after)
}

/// Doc 5 §4 steps 1-7: begin -> apply (via the tx-scoped primitive above)
/// -> commit. Rolls back entirely on any failure - a partial credit must
/// never be possible. Used directly by wallet deposits/refunds, where no
/// other table needs to change in the same transaction.
pub async fn apply_ledger_entry(pool: &SqlitePool, input: LedgerEntryInput<'_>) -> Result<i64, WalletError> {
    let mut tx = pool.begin().await?;
    let balance_after = apply_ledger_entry_in_tx(&mut tx, input).await?;
    tx.commit().await?;
    Ok(balance_after)
}

/// Doc 1 §7: coin_rate_pkr is INTEGER-typed, coins are always whole
/// numbers - Doc 5 §4 step 2: "floor rounding, define clearly." Floor
/// division is Rust's default integer division for positive operands.
pub fn compute_coins_credited(amount_pkr: i64, coin_rate_pkr: i64) -> i64 {
    if coin_rate_pkr <= 0 {
        return 0;
    }
    amount_pkr / coin_rate_pkr
}
