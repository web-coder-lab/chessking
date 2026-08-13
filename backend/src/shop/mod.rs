pub mod errors;
pub mod list;
pub mod purchase;
pub mod inventory;
pub mod gifts;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use errors::ShopError;

// ---------------------------------------------------------
// GET /shop/items?category=board  (§1.2, §1.4)
// ---------------------------------------------------------
#[derive(Deserialize)]
struct ShopQuery { category: Option<String> }

#[derive(Serialize)]
struct ShopItemWithOwned {
    #[serde(flatten)]
    item: list::ShopItemRow,
    owned: bool,
}

#[derive(Serialize)]
struct ShopItemsResponse { items: Vec<ShopItemWithOwned> }

async fn list_shop_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Query(q): Query<ShopQuery>,
) -> Result<Json<ShopItemsResponse>, ShopError> {
    let items = list::list_shop_items(&state.db, q.category.as_deref()).await?;
    let owned_ids = list::owned_item_ids(&state.db, &claims.sub).await?;

    let out = items.into_iter().map(|item| {
        let owned = owned_ids.contains(&item.id);
        ShopItemWithOwned { item, owned }
    }).collect();

    Ok(Json(ShopItemsResponse { items: out }))
}

// ---------------------------------------------------------
// POST /shop/purchase  (§1.3)
// ---------------------------------------------------------
#[derive(Serialize)]
struct PurchaseResponse { status: String, new_balance: i64, inventory_item: Option<inventory::InventoryItemRow> }

async fn purchase_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<purchase::PurchaseRequest>,
) -> Result<Json<PurchaseResponse>, ShopError> {
    let shop_item_id = req.shop_item_id.clone();
    let new_balance = purchase::purchase_item(&state.db, &claims.sub, req).await?;

    let inventory_item = sqlx::query_as::<_, inventory::InventoryItemRow>(
        "SELECT i.id AS inventory_id, i.shop_item_id, s.category, s.name, s.image_url, i.is_equipped, i.acquired_via
         FROM inventory i JOIN shop_items s ON s.id = i.shop_item_id
         WHERE i.user_id = ? AND i.shop_item_id = ?"
    )
    .bind(&claims.sub)
    .bind(&shop_item_id)
    .fetch_optional(&state.db)
    .await
    .map_err(ShopError::from)?;

    Ok(Json(PurchaseResponse { status: "purchased".to_string(), new_balance, inventory_item }))
}

// ---------------------------------------------------------
// GET /inventory  (§2.1)
// ---------------------------------------------------------
#[derive(Serialize)]
struct InventoryResponse { items: Vec<inventory::InventoryItemRow> }

async fn list_inventory_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Query(q): Query<ShopQuery>,
) -> Result<Json<InventoryResponse>, ShopError> {
    let all = inventory::list_inventory(&state.db, &claims.sub).await?;
    let items = match q.category {
        Some(cat) => all.into_iter().filter(|i| i.category == cat).collect(),
        None => all,
    };
    Ok(Json(InventoryResponse { items }))
}

// ---------------------------------------------------------
// POST /inventory/:inventory_id/equip  (§2.2)
// ---------------------------------------------------------
async fn equip_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(inventory_id): Path<String>,
) -> Result<Json<serde_json::Value>, ShopError> {
    inventory::equip_item(&state.db, &claims.sub, &inventory_id).await?;
    Ok(Json(serde_json::json!({ "status": "equipped" })))
}

// ---------------------------------------------------------
// POST /inventory/:inventory_id/unequip  (Doc 9 Sec4)
// ---------------------------------------------------------
async fn unequip_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(inventory_id): Path<String>,
) -> Result<Json<serde_json::Value>, ShopError> {
    inventory::unequip_item(&state.db, &claims.sub, &inventory_id).await?;
    Ok(Json(serde_json::json!({ "status": "unequipped" })))
}

// ---------------------------------------------------------
// POST /gifts/send  (§3.2, §3.3)
// ---------------------------------------------------------
#[derive(Serialize)]
struct SendGiftResponse { status: String, new_balance: i64 }

async fn send_gift_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<gifts::SendGiftRequest>,
) -> Result<Json<SendGiftResponse>, ShopError> {
    let in_match_broadcast = if req.context == "in_match" {
        req.match_id.clone().map(|mid| (mid, req.shop_item_id.clone()))
    } else {
        None
    };

    let balance = gifts::send_gift(&state.db, &claims.sub, req).await?;

    // §3.3: in-match gifts show live on both screens, not just the
    // sender's - the receiver is actively connected to this exact match
    // right now, so there's no reason to make them wait for the
    // notifications drawer to find out.
    if let Some((match_id, shop_item_id)) = in_match_broadcast {
        if let Ok(item) = sqlx::query_as::<_, (String, Option<String>, i64)>(
            "SELECT name, icon_emoji, price_coins FROM shop_items WHERE id = ?"
        )
        .bind(&shop_item_id)
        .fetch_one(&state.db)
        .await
        {
            let sender_username: Option<(String,)> = sqlx::query_as("SELECT username FROM users WHERE id = ?")
                .bind(&claims.sub)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();
            if let Some((sender_username,)) = sender_username {
                let payload = serde_json::json!({
                    "type": "gift_sent",
                    "sender_username": sender_username,
                    "gift_name": item.0,
                    "icon_emoji": item.1,
                    "price_coins": item.2,
                })
                .to_string();
                state.match_registry.with_session(&match_id, |session| {
                    let _ = session.events.send(payload);
                }).await;
            }
        }
    }

    Ok(Json(SendGiftResponse { status: "sent".to_string(), new_balance: balance }))
}

// ---------------------------------------------------------
// GET /gifts/catalog  (Doc 9 Sec5)
// ---------------------------------------------------------
#[derive(Serialize)]
struct GiftCatalogResponse { items: Vec<list::ShopItemRow> }

async fn gift_catalog_handler(State(state): State<AppState>) -> Result<Json<GiftCatalogResponse>, ShopError> {
    let items = list::list_shop_items(&state.db, Some("gift")).await?;
    Ok(Json(GiftCatalogResponse { items }))
}

// ---------------------------------------------------------
// GET /profile/:username/gifts-received  (§3.4)
// ---------------------------------------------------------
async fn gifts_received_handler(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Vec<gifts::GiftTallyRow>>, ShopError> {
    Ok(Json(gifts::gifts_received_tally(&state.db, &username).await?))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/shop/items", get(list_shop_handler))
        .route("/shop/purchase", post(purchase_handler))
        .route("/inventory", get(list_inventory_handler))
        .route("/inventory/:inventory_id/equip", post(equip_handler))
        .route("/inventory/:inventory_id/unequip", post(unequip_handler))
        .route("/gifts/catalog", get(gift_catalog_handler))
        .route("/gifts/send", post(send_gift_handler))
        .route("/profile/:username/gifts-received", get(gifts_received_handler))
}
