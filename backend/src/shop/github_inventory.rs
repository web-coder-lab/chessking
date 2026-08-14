//! Part 4 — durable inventory + equip state on GitHub.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use crate::db::{GitHubStore, StoreError};

use super::errors::ShopError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhInvItem {
    pub inventory_id: String,
    pub shop_item_id: String,
    pub is_equipped: bool,
    pub acquired_via: String,
    pub acquired_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GhInventoryFile {
    pub items: Vec<GhInvItem>,
    pub avatar_id: Option<String>,
    pub banner_id: Option<String>,
}

fn map_err(e: StoreError) -> ShopError {
    tracing::error!("github inventory: {e}");
    ShopError::Internal
}

pub async fn get_file(store: &GitHubStore, user_id: &str) -> Result<GhInventoryFile, ShopError> {
    match store.get_json::<GhInventoryFile>("inventory", user_id).await {
        Ok((f, _)) => Ok(f),
        Err(StoreError::NotFound) => Ok(GhInventoryFile::default()),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn save_file(store: &GitHubStore, user_id: &str, file: &GhInventoryFile) -> Result<(), ShopError> {
    let sha = match store.get_json::<Value>("inventory", user_id).await {
        Ok((_, s)) => Some(s),
        Err(StoreError::NotFound) => None,
        Err(e) => return Err(map_err(e)),
    };
    store
        .put_json(
            "inventory",
            user_id,
            file,
            sha.as_deref(),
            &format!("inventory {}", user_id),
        )
        .await
        .map_err(map_err)?;
    Ok(())
}

/// Snapshot current SQL inventory → GitHub (after equip/purchase/grant).
pub async fn sync_from_sql(store: &GitHubStore, pool: &SqlitePool, user_id: &str) -> Result<(), ShopError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: String,
        shop_item_id: String,
        is_equipped: i64,
        acquired_via: String,
        acquired_at: String,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, shop_item_id, is_equipped, acquired_via, acquired_at FROM inventory WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let avatar: Option<(Option<String>,)> =
        sqlx::query_as("SELECT avatar_id FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    let banner: Option<(Option<String>,)> =
        sqlx::query_as("SELECT banner_id FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    let file = GhInventoryFile {
        items: rows
            .into_iter()
            .map(|r| GhInvItem {
                inventory_id: r.id,
                shop_item_id: r.shop_item_id,
                is_equipped: r.is_equipped == 1,
                acquired_via: r.acquired_via,
                acquired_at: r.acquired_at,
            })
            .collect(),
        avatar_id: avatar.and_then(|a| a.0),
        banner_id: banner.and_then(|b| b.0),
    };
    save_file(store, user_id, &file).await?;

    // Keep GhUser denormalized fields in sync
    if let Ok(Some(mut gu)) = crate::auth::github_users::get_user(store, user_id).await {
        gu.avatar_id = file.avatar_id.clone();
        gu.banner_id = file.banner_id.clone();
        gu.updated_at = Utc::now().to_rfc3339();
        let _ = crate::auth::github_users::save_user(store, &gu).await;
    }
    Ok(())
}

/// After restart: if SQL inventory empty, restore rows from GitHub.
pub async fn hydrate_sql_if_empty(store: &GitHubStore, pool: &SqlitePool, user_id: &str) -> Result<(), ShopError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM inventory WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(());
    }
    let file = get_file(store, user_id).await?;
    if file.items.is_empty() {
        return Ok(());
    }
    for it in &file.items {
        sqlx::query(
            "INSERT OR IGNORE INTO inventory (id, user_id, shop_item_id, is_equipped, acquired_via, acquired_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&it.inventory_id)
        .bind(user_id)
        .bind(&it.shop_item_id)
        .bind(if it.is_equipped { 1 } else { 0 })
        .bind(&it.acquired_via)
        .bind(&it.acquired_at)
        .execute(pool)
        .await?;
    }
    if let Some(aid) = &file.avatar_id {
        let _ = sqlx::query("UPDATE users SET avatar_id = ? WHERE id = ?")
            .bind(aid)
            .bind(user_id)
            .execute(pool)
            .await;
    }
    if let Some(bid) = &file.banner_id {
        let _ = sqlx::query("UPDATE users SET banner_id = ? WHERE id = ?")
            .bind(bid)
            .bind(user_id)
            .execute(pool)
            .await;
    }
    Ok(())
}
