use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::config::RuntimeConfigStore;
use super::errors::WalletError;
use super::gateway::{GooglePayGateway, EasyPaisaGateway, JazzCashGateway, PaymentGateway};
use super::ledger::compute_coins_credited;

#[derive(Debug, Deserialize)]
pub struct InitiateDepositRequest {
    pub amount_pkr: i64,
    pub gateway: String, // "jazzcash" | "easypaisa" | "googlepay"
    pub idempotency_key: String, // §7: client-generated, prevents double-tap creating two orders
    pub payer_phone: Option<String>, // required for jazzcash/easypaisa - both are phone-number-keyed mobile wallets
}

/// Pakistani mobile format: 03XXXXXXXXX (11 digits) or +923XXXXXXXXX /
/// 923XXXXXXXXX. This is a format check only - it confirms the number
/// looks like a real Pakistani mobile number before it's sent anywhere,
/// it does not confirm the number is actually reachable or actually
/// owns a JazzCash/EasyPaisa account. That confirmation can only happen
/// gateway-side once real API credentials are wired in (see
/// gateway.rs's stub note) - this is the honest limit of what's
/// checkable without one.
fn is_valid_pk_mobile(phone: &str) -> bool {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    match digits.len() {
        11 => digits.starts_with("03"),
        12 => digits.starts_with("923"),
        _ => false,
    }
}

#[derive(Debug, Serialize)]
pub struct InitiateDepositResponse {
    pub payment_transaction_id: String,
    pub redirect_url: String,
}

/// Doc 5 §2 steps 1-4: validates amount/gateway, creates the
/// payment_transactions row (status = pending), calls the gateway to
/// create a session, returns the checkout redirect. Never credits coins
/// here — that only ever happens from a verified webhook (§2 critical
/// rule, enforced structurally by keeping this function gateway-call-only).
pub async fn initiate_deposit(
    pool: &SqlitePool,
    config: &RuntimeConfigStore,
    user_id: &str,
    req: InitiateDepositRequest,
) -> Result<InitiateDepositResponse, WalletError> {
    if req.amount_pkr <= 0 {
        return Err(WalletError::InvalidAmount);
    }

    if req.gateway == "jazzcash" || req.gateway == "easypaisa" {
        match &req.payer_phone {
            Some(phone) if is_valid_pk_mobile(phone) => {}
            _ => return Err(WalletError::InvalidPhone),
        }
    }

    // §7: idempotency — if a transaction was already created for this
    // exact client-generated key, return it instead of creating a new
    // gateway order (handles double-tap on "Pay"). This is a fast-path
    // check; the UNIQUE(user_id, idempotency_key) constraint below is
    // what actually makes this race-safe if two requests with the same
    // key land close enough together that both get past this SELECT.
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM payment_transactions WHERE user_id = ? AND idempotency_key = ?"
    )
    .bind(user_id)
    .bind(&req.idempotency_key)
    .fetch_optional(pool)
    .await?;
    if let Some((existing_id,)) = existing {
        tracing::info!(transaction_id = %existing_id, "duplicate deposit-initiate suppressed by idempotency key");
        return Err(WalletError::DuplicateIdempotencyKey);
    }

    let coin_rate: i64 = config.get_or(pool, "coin_rate_pkr", "2").await.parse().unwrap_or(2);
    let coins_estimate = compute_coins_credited(req.amount_pkr, coin_rate);

    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let insert_result = sqlx::query(
        "INSERT INTO payment_transactions (id, user_id, gateway, idempotency_key, payer_phone, amount_pkr, coins_credited, coin_rate_used, status, webhook_verified, raw_gateway_response, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?, ?)"
    )
    .bind(&transaction_id)
    .bind(user_id)
    .bind(&req.gateway)
    .bind(&req.idempotency_key)
    .bind(&req.payer_phone)
    .bind(req.amount_pkr)
    .bind(coins_estimate) // provisional; re-derived from coin_rate_used again at credit time (§4 step 2)
    .bind(coin_rate)
    .bind(serde_json::json!({ "idempotency_key": req.idempotency_key }).to_string())
    .bind(&now)
    .execute(pool)
    .await;

    if let Err(sqlx::Error::Database(db_err)) = &insert_result {
        if db_err.is_unique_violation() {
            // Lost the race: another request with the same key committed
            // first. Same outcome as the fast-path check above.
            tracing::info!(user_id, "duplicate deposit-initiate suppressed by unique constraint (race)");
            return Err(WalletError::DuplicateIdempotencyKey);
        }
    }
    insert_result?;

    let gateway: Box<dyn PaymentGateway> = build_gateway(pool, config, &req.gateway).await?;
    let (gateway_transaction_id, redirect_url) = gateway.create_session(req.amount_pkr, &transaction_id).await?;

    sqlx::query("UPDATE payment_transactions SET gateway_transaction_id = ? WHERE id = ?")
        .bind(&gateway_transaction_id)
        .bind(&transaction_id)
        .execute(pool)
        .await?;

    Ok(InitiateDepositResponse { payment_transaction_id: transaction_id, redirect_url })
}

async fn build_gateway(pool: &SqlitePool, config: &RuntimeConfigStore, gateway_name: &str) -> Result<Box<dyn PaymentGateway>, WalletError> {
    match gateway_name {
        "jazzcash" => Ok(Box::new(JazzCashGateway {
            merchant_id: config.get_or(pool, "jazzcash_merchant_id", "").await,
            api_key: config.get_or(pool, "jazzcash_api_key", "").await,
            secret: config.get_or(pool, "jazzcash_secret", "").await,
        })),
        "easypaisa" => Ok(Box::new(EasyPaisaGateway {
            merchant_id: config.get_or(pool, "easypaisa_merchant_id", "").await,
            api_key: config.get_or(pool, "easypaisa_api_key", "").await,
            secret: config.get_or(pool, "easypaisa_secret", "").await,
        })),
        "googlepay" => Ok(Box::new(GooglePayGateway {
            merchant_id: config.get_or(pool, "googlepay_merchant_id", "").await,
            api_key: config.get_or(pool, "googlepay_api_key", "").await,
            secret: config.get_or(pool, "googlepay_secret", "").await,
        })),
        _ => Err(WalletError::UnsupportedGateway),
    }
}

pub async fn gateway_for_webhook(pool: &SqlitePool, config: &RuntimeConfigStore, gateway_name: &str) -> Result<Box<dyn PaymentGateway>, WalletError> {
    build_gateway(pool, config, gateway_name).await
}

/// Coin packages for the Wallet screen grid (Doc 5 §1 — "must not be
/// hardcoded in frontend code").
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CoinPackage {
    pub id: String,
    pub amount_pkr: i64,
    pub coins: i64,
    pub bonus_label: Option<String>,
}

pub async fn list_coin_packages(pool: &SqlitePool) -> Result<Vec<CoinPackage>, WalletError> {
    let rows = sqlx::query_as::<_, CoinPackage>(
        "SELECT id, amount_pkr, coins, bonus_label FROM coin_packages WHERE is_active = 1 ORDER BY sort_order"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
