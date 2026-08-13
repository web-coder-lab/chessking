use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::ShopError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ShopItemRow {
    pub id: String,
    pub category: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub icon_emoji: Option<String>,
    pub price_coins: i64,
    pub is_limited_edition: i64,
}

/// Doc 6 §1.2: only is_active = 1 items appear. §1.4: "Shop query filters
/// out items outside their availability window automatically — no manual
/// daily toggling needed from the admin."
pub async fn list_shop_items(pool: &SqlitePool, category: Option<&str>) -> Result<Vec<ShopItemRow>, ShopError> {
    let now = Utc::now().to_rfc3339();

    let rows = if let Some(cat) = category {
        sqlx::query_as::<_, ShopItemRow>(
            "SELECT id, category, name, description, image_url, icon_emoji, price_coins, is_limited_edition
             FROM shop_items
             WHERE is_active = 1
               AND category = ?
               AND (available_from IS NULL OR available_from <= ?)
               AND (available_until IS NULL OR available_until >= ?)
             ORDER BY price_coins ASC"
        )
        .bind(cat)
        .bind(&now)
        .bind(&now)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ShopItemRow>(
            "SELECT id, category, name, description, image_url, icon_emoji, price_coins, is_limited_edition
             FROM shop_items
             WHERE is_active = 1
               AND (available_from IS NULL OR available_from <= ?)
               AND (available_until IS NULL OR available_until >= ?)
             ORDER BY created_at DESC"
        )
        .bind(&now)
        .bind(&now)
        .fetch_all(pool)
        .await?
    };

    Ok(rows)
}

/// Which item ids (from the current listing) does this user already own?
/// Used by the frontend to render the "Owned" ribbon (§1.3 step 4).
pub async fn owned_item_ids(pool: &SqlitePool, user_id: &str) -> Result<Vec<String>, ShopError> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT shop_item_id FROM inventory WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
