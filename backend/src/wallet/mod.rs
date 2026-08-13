pub mod errors;
pub mod config;
pub mod ledger;
pub mod gateway;
pub mod deposit;
pub mod webhook;
pub mod refund;
pub mod audit;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use deposit::{initiate_deposit, list_coin_packages, CoinPackage, InitiateDepositRequest, InitiateDepositResponse};
use errors::WalletError;
use webhook::handle_webhook;

// ---------------------------------------------------------
// GET /wallet/balance
// ---------------------------------------------------------
#[derive(Serialize)]
struct BalanceResponse { coin_balance: i64 }

async fn balance_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<BalanceResponse>, WalletError> {
    let row: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?")
        .bind(&claims.sub)
        .fetch_one(&state.db)
        .await?;
    Ok(Json(BalanceResponse { coin_balance: row.0 }))
}

// ---------------------------------------------------------
// GET /wallet/packages  (§1 — admin-editable, never hardcoded in frontend)
// ---------------------------------------------------------
#[derive(Serialize)]
struct PackagesResponse { packages: Vec<CoinPackage> }

async fn packages_handler(State(state): State<AppState>) -> Result<Json<PackagesResponse>, WalletError> {
    Ok(Json(PackagesResponse { packages: list_coin_packages(&state.db).await? }))
}

// ---------------------------------------------------------
// POST /wallet/deposit/initiate  (§2 steps 1-4)
// ---------------------------------------------------------
async fn initiate_deposit_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<InitiateDepositRequest>,
) -> Result<Json<InitiateDepositResponse>, WalletError> {
    let resp = initiate_deposit(&state.db, &state.wallet_config, &claims.sub, req).await?;
    Ok(Json(resp))
}

// ---------------------------------------------------------
// GET /wallet/deposit/{transaction_id}/status  (§2 step 9: frontend polls)
// ---------------------------------------------------------
#[derive(Serialize)]
struct DepositStatusResponse { status: String, coins_credited: Option<i64> }

async fn deposit_status_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(transaction_id): Path<String>,
) -> Result<Json<DepositStatusResponse>, WalletError> {
    let row: Option<(String, Option<i64>)> = sqlx::query_as(
        "SELECT status, coins_credited FROM payment_transactions WHERE id = ? AND user_id = ?"
    )
    .bind(&transaction_id)
    .bind(&claims.sub)
    .fetch_optional(&state.db)
    .await?;
    let (status, coins_credited) = row.ok_or(WalletError::TransactionNotFound)?;
    Ok(Json(DepositStatusResponse { status, coins_credited }))
}

// ---------------------------------------------------------
// GET /wallet/history  (§9: history with friendly label + icon)
// ---------------------------------------------------------
#[derive(sqlx::FromRow)]
struct RawLogRow {
    id: String,
    r#type: String,
    amount: i64,
    reference_id: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
struct TransactionRow {
    id: String,
    label: String,
    icon: String,
    amount: i64,
    created_at: String,
}

/// Doc 5 §9 mapping table, exactly as specified.
fn label_and_icon(log_type: &str) -> (&'static str, &'static str) {
    match log_type {
        "deposit" => ("Coins Purchased", "wallet_plus"),
        "shop_purchase" => ("Shop Purchase", "bag"),
        "gift_sent" => ("Gift Sent", "gift"),
        "daily_reward" => ("Daily Reward", "calendar"),
        "ad_reward" => ("Watched Ad", "play_video"),
        "referral_reward" => ("Referral Bonus", "people"),
        "admin_adjustment" => ("Account Adjustment", "shield"),
        "refund" => ("Refund", "undo"),
        _ => ("Transaction", "wallet"),
    }
}

#[derive(Serialize)]
struct HistoryResponse { transactions: Vec<TransactionRow> }

#[derive(serde::Deserialize)]
struct HistoryQuery { page: Option<i64>, limit: Option<i64>, r#type: Option<String> }

/// Doc 9 Sec3: GET /wallet/history, query: page, limit, type?
async fn history_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, axum::extract::Query(q): axum::extract::Query<HistoryQuery>) -> Result<Json<HistoryResponse>, WalletError> {
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let offset = (q.page.unwrap_or(1).max(1) - 1) * limit;

    let rows = sqlx::query_as::<_, RawLogRow>(
        "SELECT id, type, amount, reference_id, created_at FROM wallet_logs
         WHERE user_id = ? AND (?2 IS NULL OR type = ?2)
         ORDER BY created_at DESC LIMIT ?3 OFFSET ?4"
    )
    .bind(&claims.sub)
    .bind(&q.r#type)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    // shop_purchase / gift_sent need the item name / recipient interpolated
    // into the label (§9: "Shop Purchase: {item name}", "Gift Sent: {item
    // name} to {username}") — that enrichment reads shop_items/gifts by
    // reference_id, wired once that lookup is added. For now the base
    // label is returned; correct for the other six types already.
    let out = rows.into_iter().map(|r| {
        let (label, icon) = label_and_icon(&r.r#type);
        TransactionRow { id: r.id, label: label.to_string(), icon: icon.to_string(), amount: r.amount, created_at: r.created_at }
    }).collect();

    Ok(Json(HistoryResponse { transactions: out }))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/wallet/balance", get(balance_handler))
        .route("/wallet/packages", get(packages_handler))
        .route("/wallet/deposit/initiate", post(initiate_deposit_handler))
        .route("/wallet/deposit/:transaction_id/status", get(deposit_status_handler))
        .route("/wallet/history", get(history_handler))
}

/// Webhooks are NOT behind require_auth — the gateway is not a logged-in
/// user. Their authenticity comes entirely from signature verification
/// inside handle_webhook (§3 step 1), which is the real gate.
/// Doc 9 Sec3: three explicit named webhook routes (not a generic
/// :gateway wildcard) — each still calls the same handler, with the
/// gateway name baked into the route registration rather than parsed
/// from the URL, since the doc lists them as three distinct endpoints.
async fn jazzcash_webhook_handler(state: State<AppState>, headers: HeaderMap, body: Bytes) -> Result<&'static str, WalletError> {
    webhook_handler_for("jazzcash", state, headers, body).await
}
async fn easypaisa_webhook_handler(state: State<AppState>, headers: HeaderMap, body: Bytes) -> Result<&'static str, WalletError> {
    webhook_handler_for("easypaisa", state, headers, body).await
}
async fn googlepay_webhook_handler(state: State<AppState>, headers: HeaderMap, body: Bytes) -> Result<&'static str, WalletError> {
    webhook_handler_for("googlepay", state, headers, body).await
}

async fn webhook_handler_for(gateway_name: &str, State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Result<&'static str, WalletError> {
    let signature = headers.get("X-Signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    handle_webhook(&state.db, &state.wallet_config, gateway_name, &body, signature, &state.email, &state.config.frontend_base_url).await?;
    Ok("OK")
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/webhooks/jazzcash", post(jazzcash_webhook_handler))
        .route("/webhooks/easypaisa", post(easypaisa_webhook_handler))
        .route("/webhooks/googlepay", post(googlepay_webhook_handler))
}

pub async fn run_periodic_audit(pool: SqlitePool) {
    audit::spawn_periodic_audit(pool);
}
