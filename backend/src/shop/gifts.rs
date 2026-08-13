use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry_in_tx, LedgerEntryInput};
use super::errors::ShopError;

#[derive(Debug, Deserialize)]
pub struct SendGiftRequest {
    pub receiver_username: String,
    pub shop_item_id: String,
    pub context: String,       // "profile" | "in_match"
    pub match_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct GiftItemRow {
    price_coins: i64,
    is_active: i64,
    category: String,
}

/// Doc 6 §3.3, steps 1-5 exactly, both contexts (profile / in_match) use
/// this identical logic per the doc's explicit note. §3.4/§3.5: the
/// receiver NEVER gets coins, inventory rights, or any redemption path —
/// enforced structurally by this function never touching the receiver's
/// coin_balance or inventory table at all, only inserting a `gifts` row.
pub async fn send_gift(pool: &SqlitePool, sender_id: &str, req: SendGiftRequest) -> Result<i64, ShopError> {
    let receiver: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE username_lower = ?")
        .bind(req.receiver_username.to_lowercase())
        .fetch_optional(pool)
        .await?;
    let receiver_id = receiver.ok_or(ShopError::ReceiverNotFound)?.0;

    if sender_id == receiver_id {
        return Err(ShopError::CannotGiftSelf);
    }

    // Step 2: re-fetch price server-side, never trust the client
    let item = sqlx::query_as::<_, GiftItemRow>(
        "SELECT price_coins, is_active, category FROM shop_items WHERE id = ?"
    )
    .bind(&req.shop_item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ShopError::ItemNotFound)?;

    if item.is_active == 0 {
        return Err(ShopError::ItemNotActive);
    }
    if item.category != "gift" {
        return Err(ShopError::NotEquippable); // only category='gift' items are sendable this way
    }

    let mut tx = pool.begin().await?;

    // Step 3: balance check
    let current: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?")
        .bind(sender_id)
        .fetch_one(&mut *tx)
        .await?;
    if current.0 < item.price_coins {
        return Err(ShopError::InsufficientCoins);
    }

    // Step 4: deduct sender's coins + hash-chained wallet_logs row
    let gift_id = Uuid::new_v4().to_string();
    let balance_after = apply_ledger_entry_in_tx(&mut tx, LedgerEntryInput {
        user_id: sender_id,
        log_type: "gift_sent",
        amount: -item.price_coins,
        reference_id: Some(&gift_id),
        ip_address: None,
        device_fingerprint: None,
    }).await?;

    // Step 4: insert gifts row — this is the ONLY thing the receiver's
    // side gets. No coin_balance change, no inventory row, ever (§3.4/§3.5).
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO gifts (id, sender_id, receiver_id, shop_item_id, coins_spent, context, match_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&gift_id)
    .bind(sender_id)
    .bind(&receiver_id)
    .bind(&req.shop_item_id)
    .bind(item.price_coins)
    .bind(&req.context)
    .bind(&req.match_id)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?; // step 5

    // Step 6: notify receiver — "{sender_username} sent you a X!"
    if let Ok(Some((sender_username,))) = sqlx::query_as::<_, (String,)>("SELECT username FROM users WHERE id = ?")
        .bind(sender_id)
        .fetch_optional(pool)
        .await
    {
        let gift_name: Option<(String,)> = sqlx::query_as("SELECT name FROM shop_items WHERE id = ?")
            .bind(&req.shop_item_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        let gift_label = gift_name.map(|(n,)| n).unwrap_or_else(|| "a gift".to_string());
        let _ = crate::social::notifications::create_notification(
            pool,
            &receiver_id,
            "gift_received",
            "You received a gift!",
            Some(&format!("{sender_username} sent you {gift_label}.")),
            Some(&gift_id),
        ).await;
    }

    Ok(balance_after)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GiftTallyRow {
    pub shop_item_id: String,
    pub name: String,
    pub count: i64,
}

/// Doc 6 §3.4: Profile → "Gifts Received" tab shows a permanent tally per
/// item type (e.g. "🧸 x5"), purely social, zero economic value.
pub async fn gifts_received_tally(pool: &SqlitePool, username: &str) -> Result<Vec<GiftTallyRow>, ShopError> {
    let rows = sqlx::query_as::<_, GiftTallyRow>(
        "SELECT g.shop_item_id, s.name, COUNT(*) AS count
         FROM gifts g
         JOIN shop_items s ON s.id = g.shop_item_id
         JOIN users u ON u.id = g.receiver_id
         WHERE u.username_lower = ?
         GROUP BY g.shop_item_id
         ORDER BY count DESC"
    )
    .bind(username.to_lowercase())
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
