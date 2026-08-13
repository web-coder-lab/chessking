use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum ShopError {
    ItemNotFound,
    ItemNotActive,
    AlreadyOwned,
    InsufficientCoins,
    NotEquippable,
    ReceiverNotFound,
    CannotGiftSelf,
    Unauthorized,
    Internal,
}

impl IntoResponse for ShopError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ShopError::ItemNotFound => (StatusCode::NOT_FOUND, "item_not_found", "This item is not available.".to_string()),
            ShopError::ItemNotActive => (StatusCode::NOT_FOUND, "item_not_active", "This item is not currently available.".to_string()),
            ShopError::AlreadyOwned => (StatusCode::CONFLICT, "already_owned", "You already own this item.".to_string()),
            // §1.3 step 3b: "Not enough coins" toast + shortcut to Wallet
            ShopError::InsufficientCoins => (StatusCode::PAYMENT_REQUIRED, "insufficient_coins", "Not enough coins.".to_string()),
            ShopError::NotEquippable => (StatusCode::BAD_REQUEST, "not_equippable", "This item cannot be equipped.".to_string()),
            ShopError::ReceiverNotFound => (StatusCode::NOT_FOUND, "receiver_not_found", "This user could not be found.".to_string()),
            ShopError::CannotGiftSelf => (StatusCode::BAD_REQUEST, "cannot_gift_self", "You can't send a gift to yourself.".to_string()),
            ShopError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            ShopError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong. Please try again.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for ShopError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in shop: {e:?}");
        ShopError::Internal
    }
}
