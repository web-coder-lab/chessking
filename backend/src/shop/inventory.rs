use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::ShopError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InventoryItemRow {
    pub inventory_id: String,
    pub shop_item_id: String,
    pub category: String,
    pub name: String,
    pub image_url: Option<String>,
    pub is_equipped: i64,
    pub acquired_via: String,
}

/// Doc 6 §2.1: all inventory rows for the user, grouped by category tabs
/// (gift excluded — gift items never enter inventory at all, per §3.1).
pub async fn list_inventory(pool: &SqlitePool, user_id: &str) -> Result<Vec<InventoryItemRow>, ShopError> {
    let rows = sqlx::query_as::<_, InventoryItemRow>(
        "SELECT i.id AS inventory_id, i.shop_item_id, s.category, s.name, s.image_url, i.is_equipped, i.acquired_via
         FROM inventory i
         JOIN shop_items s ON s.id = i.shop_item_id
         WHERE i.user_id = ? AND s.category != 'gift'
         ORDER BY s.category, i.acquired_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Doc 6 §2.2, steps 1-3 exactly, ALL as one DB transaction:
/// a. find any other equipped row in the SAME category, auto-unequip it
/// b. equip the newly selected row
/// c. if category is avatar/banner, also update the denormalized
///    users.avatar_id / users.banner_id (Doc 1 users table)
/// Guarantees: never two items equipped in one category simultaneously,
/// never zero once at least one has been equipped (enforced by only ever
/// calling this on an item the user owns — see §2.3 defaults).
pub async fn equip_item(pool: &SqlitePool, user_id: &str, inventory_id: &str) -> Result<(), ShopError> {
    let mut tx = pool.begin().await?;

    #[derive(sqlx::FromRow)]
    struct TargetRow { shop_item_id: String, category: String }

    let target = sqlx::query_as::<_, TargetRow>(
        "SELECT i.shop_item_id, s.category
         FROM inventory i JOIN shop_items s ON s.id = i.shop_item_id
         WHERE i.id = ? AND i.user_id = ?"
    )
    .bind(inventory_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ShopError::ItemNotFound)?;

    // Step a: auto-unequip whatever else is equipped in this category
    sqlx::query(
        "UPDATE inventory SET is_equipped = 0
         WHERE user_id = ? AND is_equipped = 1
           AND shop_item_id IN (SELECT id FROM shop_items WHERE category = ?)"
    )
    .bind(user_id)
    .bind(&target.category)
    .execute(&mut *tx)
    .await?;

    // Step b: equip the newly selected row
    sqlx::query("UPDATE inventory SET is_equipped = 1 WHERE id = ?")
        .bind(inventory_id)
        .execute(&mut *tx)
        .await?;

    // Step c: denormalized users.avatar_id / users.banner_id
    if target.category == "avatar" {
        sqlx::query("UPDATE users SET avatar_id = ?, updated_at = ? WHERE id = ?")
            .bind(&target.shop_item_id)
            .bind(Utc::now().to_rfc3339())
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    } else if target.category == "banner" {
        sqlx::query("UPDATE users SET banner_id = ?, updated_at = ? WHERE id = ?")
            .bind(&target.shop_item_id)
            .bind(Utc::now().to_rfc3339())
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Doc 9 §4: "POST /inventory/{inventory_id}/unequip". Doc 6 §2.2
/// explicitly forbids a category ever reaching zero-equipped once
/// something has been equipped. Both are satisfied by treating "unequip"
/// as "fall back to this category's free default item" (which every
/// user owns per §2.3) rather than truly clearing the category — the
/// user never sees an empty slot, and the endpoint the API reference
/// documents still does something real.
pub async fn unequip_item(pool: &SqlitePool, user_id: &str, inventory_id: &str) -> Result<(), ShopError> {
    #[derive(sqlx::FromRow)]
    struct TargetRow { category: String, is_equipped: i64 }

    let target = sqlx::query_as::<_, TargetRow>(
        "SELECT s.category, i.is_equipped FROM inventory i JOIN shop_items s ON s.id = i.shop_item_id
         WHERE i.id = ? AND i.user_id = ?"
    )
    .bind(inventory_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ShopError::ItemNotFound)?;

    if target.is_equipped == 0 {
        return Ok(()); // nothing to do — wasn't equipped in the first place
    }

    let default_item_id = match target.category.as_str() {
        "board" => "item_default_board",
        "piece_set" => "item_default_pieces",
        "avatar" => "item_default_avatar",
        "banner" => "item_default_banner",
        _ => return Err(ShopError::NotEquippable),
    };

    let default_inventory_id: (String,) = sqlx::query_as(
        "SELECT id FROM inventory WHERE user_id = ? AND shop_item_id = ?"
    )
    .bind(user_id)
    .bind(default_item_id)
    .fetch_one(pool) // every user owns their category defaults per §2.3 — this must exist
    .await?;

    equip_item(pool, user_id, &default_inventory_id.0).await
}


/// item per category (board/piece_set/avatar/banner) at account creation.
/// Doc 6 offers two options for `acquired_via` on these rows: "'purchase'
/// with price_coins = 0 conceptually" OR a dedicated 'default' value.
/// Doc 1's `inventory.acquired_via` CHECK constraint only allows
/// ('purchase', 'gift_received'), so the first option is the one that's
/// actually schema-compatible without an ALTER TABLE migration — used
/// here. Called from auth::register right after the user row is inserted.
pub async fn grant_default_items(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, user_id: &str) -> Result<(), sqlx::Error> {
    const DEFAULTS: [(&str, &str); 4] = [
        ("board", "item_default_board"),
        ("piece_set", "item_default_pieces"),
        ("avatar", "item_default_avatar"),
        ("banner", "item_default_banner"),
    ];

    let now = Utc::now().to_rfc3339();

    for (category, item_id) in DEFAULTS {
        let inv_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO inventory (id, user_id, shop_item_id, is_equipped, acquired_via, acquired_at)
             VALUES (?, ?, ?, 1, 'purchase', ?)"
        )
        .bind(&inv_id)
        .bind(user_id)
        .bind(item_id)
        .bind(&now)
        .execute(&mut **tx)
        .await?;

        if category == "avatar" {
            sqlx::query("UPDATE users SET avatar_id = ? WHERE id = ?")
                .bind(item_id).bind(user_id).execute(&mut **tx).await?;
        } else if category == "banner" {
            sqlx::query("UPDATE users SET banner_id = ? WHERE id = ?")
                .bind(item_id).bind(user_id).execute(&mut **tx).await?;
        }
    }

    Ok(())
}
