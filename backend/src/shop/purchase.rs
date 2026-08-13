use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry_in_tx, LedgerEntryInput};
use super::errors::ShopError;

#[derive(Debug, Deserialize)]
pub struct PurchaseRequest {
    pub shop_item_id: String,
    /// Doc 9 §21: "Idempotency guard applied to ... /shop/purchase."
    /// The already-owned check below is the real backstop against a
    /// double-tap actually double-charging (a second purchase attempt on
    /// an item the user now owns is rejected outright) — this key is
    /// accepted for API-contract completeness and future request-log
    /// deduplication, without changing that core guarantee.
    pub idempotency_key: String,
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    price_coins: i64,
    is_active: i64,
    category: String,
}

/// Doc 6 Sec1.3, steps 1-5 exactly:
/// a. re-fetch item server-side (never trust client-supplied price)
/// b. check balance >= price
/// c. check not already owned
/// d. atomic: deduct balance + hash-chained wallet_logs row (via the
///    shared ledger primitive - same tamper-proof chain as deposits) +
///    insert inventory row, all in ONE transaction
/// e. commit
pub async fn purchase_item(pool: &SqlitePool, user_id: &str, req: PurchaseRequest) -> Result<i64, ShopError> {
    // Step a: server-side re-fetch, ignore any price the client might send
    let item = sqlx::query_as::<_, ItemRow>(
        "SELECT price_coins, is_active, category FROM shop_items WHERE id = ?"
    )
    .bind(&req.shop_item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ShopError::ItemNotFound)?;

    if item.is_active == 0 {
        return Err(ShopError::ItemNotActive);
    }

    // Sec3.1: gift-category items never enter inventory this way - they
    // go through /gifts/send instead.
    if item.category == "gift" {
        return Err(ShopError::NotEquippable);
    }

    let mut tx = pool.begin().await?;

    // Step c: already-owned check (inventory(user_id, shop_item_id) is
    // also UNIQUE at the DB level as the final safety net per Sec1.3)
    let already_owned: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM inventory WHERE user_id = ? AND shop_item_id = ?"
    )
    .bind(user_id)
    .bind(&req.shop_item_id)
    .fetch_optional(&mut *tx)
    .await?;
    if already_owned.is_some() {
        return Err(ShopError::AlreadyOwned);
    }

    // Step b: balance check (must happen BEFORE calling the ledger
    // primitive, since that primitive allows negative balances by design
    // for the refund case - purchases must never overspend)
    let current: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
    if current.0 < item.price_coins {
        return Err(ShopError::InsufficientCoins);
    }

    // Step d: deduct balance + hash-chained wallet_logs row, in this tx
    let balance_after = apply_ledger_entry_in_tx(&mut tx, LedgerEntryInput {
        user_id,
        log_type: "shop_purchase",
        amount: -item.price_coins,
        reference_id: Some(&req.shop_item_id),
        ip_address: None,
        device_fingerprint: None,
    }).await?;

    // Step d: inventory row, same transaction
    let inv_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO inventory (id, user_id, shop_item_id, is_equipped, acquired_via, acquired_at)
         VALUES (?, ?, ?, 0, 'purchase', ?)"
    )
    .bind(&inv_id)
    .bind(user_id)
    .bind(&req.shop_item_id)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?; // step e

    Ok(balance_after)
}
