use chrono::Utc;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::wallet::errors::WalletError;

/// §8: "Changing a gateway key does not require a code deploy or server
/// restart — the backend reads config at request time (with a short
/// in-memory cache, invalidated on config update)."
pub struct RuntimeConfigStore {
    cache: RwLock<HashMap<String, (String, Instant)>>,
}

const CACHE_TTL: Duration = Duration::from_secs(30);

/// Keys whose values must never appear in audit logs or API responses —
/// per §8: "old value masked/redacted in logs."
const SENSITIVE_KEY_SUFFIXES: [&str; 3] = ["_api_key", "_secret", "_merchant_id"];

fn is_sensitive(key: &str) -> bool {
    SENSITIVE_KEY_SUFFIXES.iter().any(|suffix| key.ends_with(suffix))
}

impl RuntimeConfigStore {
    pub fn new() -> Self {
        Self { cache: RwLock::new(HashMap::new()) }
    }

    pub async fn get(&self, pool: &SqlitePool, key: &str) -> Result<Option<String>, WalletError> {
        if let Some((value, fetched_at)) = self.cache.read().unwrap().get(key) {
            if fetched_at.elapsed() < CACHE_TTL {
                return Ok(Some(value.clone()));
            }
        }

        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM app_config WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await?;

        if let Some((value,)) = &row {
            self.cache.write().unwrap().insert(key.to_string(), (value.clone(), Instant::now()));
        }
        Ok(row.map(|(v,)| v))
    }

    pub async fn get_or(&self, pool: &SqlitePool, key: &str, default: &str) -> String {
        self.get(pool, key).await.ok().flatten().unwrap_or_else(|| default.to_string())
    }

    /// §8: every config change is written to admin_audit_log, with
    /// sensitive old/new values masked — "store only that *a* change
    /// happened and by whom, not the literal old/new secret values."
    pub async fn set(&self, pool: &SqlitePool, key: &str, value: &str, admin_id: &str, ip_address: Option<&str>) -> Result<(), WalletError> {
        let old_value: Option<(String,)> = sqlx::query_as("SELECT value FROM app_config WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await?;

        sqlx::query(
            "INSERT INTO app_config (key, value, updated_by, updated_at) VALUES (?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_by = excluded.updated_by, updated_at = excluded.updated_at"
        )
        .bind(key)
        .bind(value)
        .bind(admin_id)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

        self.cache.write().unwrap().remove(key); // invalidate cache immediately

        let (old_logged, new_logged) = if is_sensitive(key) {
            (
                serde_json::json!({ "changed": old_value.is_some() }),
                serde_json::json!({ "changed": true }),
            )
        } else {
            (
                serde_json::json!(old_value.map(|(v,)| v)),
                serde_json::json!(value),
            )
        };

        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO admin_audit_log (id, admin_id, action, old_value, new_value, ip_address, created_at)
             VALUES (?, ?, 'update_payment_config', ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(admin_id)
        .bind(old_logged.to_string())
        .bind(new_logged.to_string())
        .bind(ip_address)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

        Ok(())
    }
}
