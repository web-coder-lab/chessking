use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::errors::AdminError;
use super::rbac::write_audit_log;
use super::reauth::require_reauth;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PaymentTxRow {
    pub id: String, pub user_id: String, pub gateway: String, pub amount_pkr: i64,
    pub coins_credited: Option<i64>, pub status: String, pub created_at: String,
}

/// Doc 9 §5: "Full transaction log table, filterable by date range,
/// gateway, status, user." Bounded + paginated per §1's explicit "never
/// an unbounded raw dump."
pub async fn list_transactions(
    pool: &SqlitePool,
    gateway: Option<&str>,
    status: Option<&str>,
    user_id: Option<&str>,
    page: i64,
) -> Result<Vec<PaymentTxRow>, AdminError> {
    let limit = 50i64;
    let offset = (page.max(1) - 1) * limit;

    let rows = sqlx::query_as::<_, PaymentTxRow>(
        "SELECT id, user_id, gateway, amount_pkr, coins_credited, status, created_at
         FROM payment_transactions
         WHERE (?1 IS NULL OR gateway = ?1)
           AND (?2 IS NULL OR status = ?2)
           AND (?3 IS NULL OR user_id = ?3)
         ORDER BY created_at DESC LIMIT ?4 OFFSET ?5"
    )
    .bind(gateway).bind(status).bind(user_id).bind(limit).bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Doc 9 §5: "Pending/stuck transaction alerts (payments stuck in
/// 'pending' beyond a reasonable time window)." 30 minutes is used as
/// that window — payment gateways typically resolve in seconds to a few
/// minutes, so anything still pending after 30 is a genuine anomaly.
const STUCK_THRESHOLD_MINUTES: i64 = 30;

pub async fn list_stuck_transactions(pool: &SqlitePool) -> Result<Vec<PaymentTxRow>, AdminError> {
    let cutoff = (Utc::now() - Duration::minutes(STUCK_THRESHOLD_MINUTES)).to_rfc3339();
    let rows = sqlx::query_as::<_, PaymentTxRow>(
        "SELECT id, user_id, gateway, amount_pkr, coins_credited, status, created_at
         FROM payment_transactions WHERE status = 'pending' AND created_at < ?
         ORDER BY created_at ASC"
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ---------------------------------------------------------
// Payment config editor (§5) — re-auth required, secrets masked in audit
// ---------------------------------------------------------
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentConfigRequest {
    pub admin_password: String, // re-auth, checked first
    pub key: String,            // e.g. "jazzcash_api_key", "coin_rate_pkr"
    pub value: String,
}

const EDITABLE_PAYMENT_KEYS: [&str; 10] = [
    "jazzcash_api_key", "jazzcash_merchant_id", "jazzcash_secret",
    "easypaisa_api_key", "easypaisa_merchant_id", "easypaisa_secret",
    "googlepay_api_key", "googlepay_merchant_id", "googlepay_secret",
    "coin_rate_pkr",
];

pub async fn update_payment_config(
    pool: &SqlitePool,
    config_store: &crate::wallet::config::RuntimeConfigStore,
    admin_id: &str,
    req: UpdatePaymentConfigRequest,
    admin_ip: Option<&str>,
) -> Result<(), AdminError> {
    require_reauth(pool, admin_id, &req.admin_password).await?;

    if !EDITABLE_PAYMENT_KEYS.contains(&req.key.as_str()) {
        return Err(AdminError::ValidationFailed("Unknown config key.".to_string()));
    }

    // §5: "Changes take effect immediately ... and are captured in
    // admin_audit_log (secret values themselves are masked)." Both of
    // those behaviors already live in RuntimeConfigStore::set (Doc 5
    // §8) — reused here rather than reimplemented, so payment config
    // changes from the Admin Panel and from wallet-internal code always
    // go through the exact same cache-invalidation + masked-audit path.
    config_store.set(pool, &req.key, &req.value, admin_id, admin_ip)
        .await
        .map_err(|_| AdminError::Internal)?;

    let _ = admin_ip; // already passed through to config_store.set above
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UpdateCoinPackageRequest {
    pub admin_password: String,
    pub package_id: String,
    pub amount_pkr: i64,
    pub coins: i64,
    pub bonus_label: Option<String>,
    pub is_active: bool,
}

pub async fn update_coin_package(pool: &SqlitePool, admin_id: &str, req: UpdateCoinPackageRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    require_reauth(pool, admin_id, &req.admin_password).await?;

    let old: Option<(i64, i64)> = sqlx::query_as("SELECT amount_pkr, coins FROM coin_packages WHERE id = ?")
        .bind(&req.package_id).fetch_optional(pool).await?;

    sqlx::query(
        "UPDATE coin_packages SET amount_pkr = ?, coins = ?, bonus_label = ?, is_active = ?, updated_at = ? WHERE id = ?"
    )
    .bind(req.amount_pkr).bind(req.coins).bind(&req.bonus_label).bind(req.is_active as i64)
    .bind(Utc::now().to_rfc3339()).bind(&req.package_id)
    .execute(pool).await?;

    write_audit_log(pool, admin_id, "update_coin_package",
        old.map(|(a, c)| serde_json::json!({"amount_pkr": a, "coins": c})),
        Some(serde_json::json!({"amount_pkr": req.amount_pkr, "coins": req.coins, "is_active": req.is_active})),
        admin_ip).await
}
