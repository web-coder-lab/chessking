//! Part 3 — durable coin balance + daily claims on GitHub.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::db::{GitHubStore, StoreError};

use super::errors::AuthError;
use super::github_users;

fn map_err(e: StoreError) -> AuthError {
    tracing::error!("github wallet: {e}");
    AuthError::Internal
}

/// Push authoritative balance onto durable user record.
pub async fn sync_balance(store: &GitHubStore, user_id: &str, coin_balance: i64) -> Result<(), AuthError> {
    let Some(mut user) = github_users::get_user(store, user_id).await? else {
        tracing::warn!(user_id, "sync_balance: user missing on GitHub");
        return Ok(());
    };
    user.coin_balance = coin_balance;
    user.updated_at = Utc::now().to_rfc3339();
    github_users::save_user(store, &user).await
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GhDailyFile {
    /// date (YYYY-MM-DD) → claim meta
    pub claims: HashMap<String, GhDailyClaim>,
    pub current_streak_day: i64,
    pub last_claim_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhDailyClaim {
    pub streak_day: i64,
    pub coins_awarded: i64,
    pub claimed_at: String,
}

pub async fn get_daily(store: &GitHubStore, user_id: &str) -> Result<GhDailyFile, AuthError> {
    match store.get_json::<GhDailyFile>("daily_rewards", user_id).await {
        Ok((f, _)) => Ok(f),
        Err(StoreError::NotFound) => Ok(GhDailyFile::default()),
        Err(e) => Err(map_err(e)),
    }
}

pub async fn save_daily(store: &GitHubStore, user_id: &str, file: &GhDailyFile) -> Result<(), AuthError> {
    let sha = match store.get_json::<Value>("daily_rewards", user_id).await {
        Ok((_, s)) => Some(s),
        Err(StoreError::NotFound) => None,
        Err(e) => return Err(map_err(e)),
    };
    store
        .put_json(
            "daily_rewards",
            user_id,
            file,
            sha.as_deref(),
            &format!("daily claim {}", user_id),
        )
        .await
        .map_err(map_err)?;
    Ok(())
}

pub async fn claimed_today(store: &GitHubStore, user_id: &str, today: &str) -> Result<bool, AuthError> {
    let f = get_daily(store, user_id).await?;
    Ok(f.claims.contains_key(today))
}

pub async fn record_claim(
    store: &GitHubStore,
    user_id: &str,
    today: &str,
    streak_day: i64,
    coins: i64,
) -> Result<(), AuthError> {
    let mut f = get_daily(store, user_id).await?;
    f.claims.insert(
        today.to_string(),
        GhDailyClaim {
            streak_day,
            coins_awarded: coins,
            claimed_at: Utc::now().to_rfc3339(),
        },
    );
    f.current_streak_day = streak_day;
    f.last_claim_date = Some(today.to_string());
    save_daily(store, user_id, &f).await
}
