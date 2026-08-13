use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::errors::AdminError;
use super::rbac::write_audit_log;

#[derive(Debug, Deserialize)]
pub struct CreateShopItemRequest {
    pub category: String, pub name: String, pub description: Option<String>,
    pub image_url: String, pub price_coins: i64,
    pub is_limited_edition: bool, pub available_from: Option<String>, pub available_until: Option<String>,
}

pub async fn create_shop_item(pool: &SqlitePool, admin_id: &str, req: CreateShopItemRequest, admin_ip: Option<&str>) -> Result<String, AdminError> {
    const VALID_CATEGORIES: [&str; 5] = ["board", "piece_set", "avatar", "banner", "gift"];
    if !VALID_CATEGORIES.contains(&req.category.as_str()) {
        return Err(AdminError::ValidationFailed("Invalid category.".to_string()));
    }
    if req.price_coins < 0 {
        return Err(AdminError::ValidationFailed("Price cannot be negative.".to_string()));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO shop_items (id, category, name, description, image_url, price_coins, is_active, is_limited_edition, available_from, available_until, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?)"
    )
    .bind(&id).bind(&req.category).bind(&req.name).bind(&req.description).bind(&req.image_url)
    .bind(req.price_coins).bind(req.is_limited_edition as i64).bind(&req.available_from).bind(&req.available_until)
    .bind(Utc::now().to_rfc3339())
    .execute(pool).await?;

    write_audit_log(pool, admin_id, "create_shop_item", None,
        Some(serde_json::json!({"id": id, "name": req.name, "price_coins": req.price_coins})), admin_ip).await?;

    Ok(id)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateShopItemRequest {
    pub name: Option<String>, pub description: Option<String>, pub price_coins: Option<i64>,
    pub is_active: Option<bool>, pub available_from: Option<String>, pub available_until: Option<String>,
}

pub async fn update_shop_item(pool: &SqlitePool, admin_id: &str, item_id: &str, req: UpdateShopItemRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    #[derive(sqlx::FromRow)]
    struct OldRow { name: String, price_coins: i64, is_active: i64 }
    let old = sqlx::query_as::<_, OldRow>("SELECT name, price_coins, is_active FROM shop_items WHERE id = ?")
        .bind(item_id).fetch_optional(pool).await?.ok_or(AdminError::NotFound)?;

    sqlx::query(
        "UPDATE shop_items SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            price_coins = COALESCE(?, price_coins),
            is_active = COALESCE(?, is_active),
            available_from = COALESCE(?, available_from),
            available_until = COALESCE(?, available_until)
         WHERE id = ?"
    )
    .bind(&req.name).bind(&req.description).bind(req.price_coins)
    .bind(req.is_active.map(|b| b as i64)).bind(&req.available_from).bind(&req.available_until)
    .bind(item_id)
    .execute(pool).await?;

    write_audit_log(pool, admin_id, "update_shop_item",
        Some(serde_json::json!({"name": old.name, "price_coins": old.price_coins, "is_active": old.is_active})),
        Some(serde_json::json!({"item_id": item_id, "changes": req})),
        admin_ip).await
}

pub async fn deactivate_shop_item(pool: &SqlitePool, admin_id: &str, item_id: &str, admin_ip: Option<&str>) -> Result<(), AdminError> {
    sqlx::query("UPDATE shop_items SET is_active = 0 WHERE id = ?").bind(item_id).execute(pool).await?;
    write_audit_log(pool, admin_id, "deactivate_shop_item", None,
        Some(serde_json::json!({"item_id": item_id})), admin_ip).await
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ShopItemPopularityRow {
    pub id: String, pub name: String, pub category: String, pub purchase_count: i64,
}

/// Doc 9 §7: "View purchase counts per item (popularity insight)."
pub async fn item_popularity(pool: &SqlitePool) -> Result<Vec<ShopItemPopularityRow>, AdminError> {
    let rows = sqlx::query_as::<_, ShopItemPopularityRow>(
        "SELECT s.id, s.name, s.category, COUNT(i.id) AS purchase_count
         FROM shop_items s LEFT JOIN inventory i ON i.shop_item_id = s.id AND i.acquired_via = 'purchase'
         GROUP BY s.id ORDER BY purchase_count DESC"
    )
    .fetch_all(pool).await?;
    Ok(rows)
}
