use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum WalletError {
    InvalidAmount,
    InvalidPhone,
    UnsupportedGateway,
    TransactionNotFound,
    WebhookSignatureInvalid,
    DuplicateIdempotencyKey,
    InsufficientBalance,
    Unauthorized,
    Internal,
}

impl IntoResponse for WalletError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            WalletError::InvalidAmount => (StatusCode::BAD_REQUEST, "invalid_amount", "Enter a valid amount.".to_string()),
            WalletError::InvalidPhone => (StatusCode::BAD_REQUEST, "invalid_phone", "Enter a valid Pakistani mobile number for this payment method (e.g. 03XXXXXXXXX).".to_string()),
            WalletError::UnsupportedGateway => (StatusCode::BAD_REQUEST, "unsupported_gateway", "This payment method is not available.".to_string()),
            WalletError::TransactionNotFound => (StatusCode::NOT_FOUND, "transaction_not_found", "Transaction not found.".to_string()),
            // §3 step 1: reject immediately on bad signature — no detail leaked
            WalletError::WebhookSignatureInvalid => (StatusCode::UNAUTHORIZED, "invalid_signature", "Invalid webhook signature.".to_string()),
            WalletError::DuplicateIdempotencyKey => (StatusCode::CONFLICT, "duplicate_request", "This payment is already being processed.".to_string()),
            WalletError::InsufficientBalance => (StatusCode::BAD_REQUEST, "insufficient_balance", "Not enough coins.".to_string()),
            WalletError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            WalletError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong. Please try again.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for WalletError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in wallet: {e:?}");
        WalletError::Internal
    }
}
