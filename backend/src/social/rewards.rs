use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry, LedgerEntryInput};
use super::errors::SocialError;

const STREAK_COINS: [i64; 7] = [5, 5, 10, 10, 15, 15, 25]; // day 1..7, admin-tunable later via app_config if needed

#[derive(Debug, Serialize)]
pub struct DailyStatusResponse { pub current_streak_day: i64, pub claimed_today: bool, pub next_reward_coins: i64 }

pub async fn daily_status(
    pool: &SqlitePool,
    user_id: &str,
    gh: Option<&crate::db::GitHubStore>,
) -> Result<DailyStatusResponse, SocialError> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let mut claimed_today: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM daily_rewards WHERE user_id = ? AND claim_date = ?")
        .bind(user_id).bind(&today).fetch_one(pool).await?;
    // Part 3: durable claim flag
    if claimed_today.0 == 0 {
        if let Some(store) = gh {
            if crate::auth::github_wallet::claimed_today(store, user_id, &today)
                .await
                .unwrap_or(false)
            {
                claimed_today.0 = 1;
            }
        }
    }

    let last: Option<(i64, String)> = sqlx::query_as(
        "SELECT streak_day, claim_date FROM daily_rewards WHERE user_id = ? ORDER BY claim_date DESC LIMIT 1"
    ).bind(user_id).fetch_optional(pool).await?;

    let yesterday = (Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let mut current_streak_day = match last {
        Some((day, date)) if date == today => day,
        Some((day, date)) if date == yesterday => (day % 7) + 1,
        _ => 1,
    };
    // Durable streak if SQL empty
    if last.is_none() {
        if let Some(store) = gh {
            if let Ok(f) = crate::auth::github_wallet::get_daily(store, user_id).await {
                if let Some(ref ld) = f.last_claim_date {
                    if ld == &today {
                        current_streak_day = f.current_streak_day.max(1);
                    } else if ld == &yesterday {
                        current_streak_day = (f.current_streak_day % 7) + 1;
                    }
                }
            }
        }
    }

    let next_reward_coins = STREAK_COINS[(current_streak_day as usize - 1).min(6)];
    Ok(DailyStatusResponse { current_streak_day, claimed_today: claimed_today.0 > 0, next_reward_coins })
}

#[derive(Debug, Serialize)]
pub struct DailyClaimResponse { pub status: String, pub coins_awarded: i64, pub new_streak_day: i64 }

pub async fn claim_daily(
    pool: &SqlitePool,
    user_id: &str,
    gh: Option<&crate::db::GitHubStore>,
) -> Result<DailyClaimResponse, SocialError> {
    let status = daily_status(pool, user_id, gh).await?;
    if status.claimed_today {
        return Err(SocialError::AlreadyClaimed);
    }

    let coins = status.next_reward_coins;
    let balance_after = apply_ledger_entry(pool, LedgerEntryInput {
        user_id, log_type: "daily_reward", amount: coins,
        reference_id: None, ip_address: None, device_fingerprint: None,
    }).await.map_err(|_| SocialError::Internal)?;

    let id = Uuid::new_v4().to_string();
    let today = Utc::now().format("%Y-%m-%d").to_string();
    sqlx::query(
        "INSERT INTO daily_rewards (id, user_id, streak_day, coins_awarded, claimed_at, claim_date) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id).bind(user_id).bind(status.current_streak_day).bind(coins).bind(Utc::now().to_rfc3339()).bind(&today)
    .execute(pool).await?;

    // Part 3 durable
    if let Some(store) = gh {
        let _ = crate::auth::github_wallet::record_claim(
            store, user_id, &today, status.current_streak_day, coins,
        ).await;
        let _ = crate::auth::github_wallet::sync_balance(store, user_id, balance_after).await;
    }

    Ok(DailyClaimResponse { status: "claimed".to_string(), coins_awarded: coins, new_streak_day: status.current_streak_day })
}

#[derive(Debug, Serialize)]
pub struct AdsStatusResponse { pub ads_watched_today: i64, pub daily_cap: i64, pub cooldown_remaining_seconds: i64 }

const AD_DAILY_CAP: i64 = 10;
const AD_COOLDOWN_MINUTES: i64 = 2;

pub async fn ads_status(pool: &SqlitePool, user_id: &str) -> Result<AdsStatusResponse, SocialError> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let watched: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ad_views WHERE user_id = ? AND view_date = ? AND verified_server_side = 1"
    ).bind(user_id).bind(&today).fetch_one(pool).await?;

    let last: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM ad_views WHERE user_id = ? AND verified_server_side = 1 ORDER BY created_at DESC LIMIT 1"
    ).bind(user_id).fetch_optional(pool).await?;

    let cooldown_remaining = last
        .and_then(|(s,)| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| {
            let elapsed = Utc::now().signed_duration_since(dt.with_timezone(&Utc));
            (chrono::Duration::minutes(AD_COOLDOWN_MINUTES) - elapsed).num_seconds().max(0)
        })
        .unwrap_or(0);

    Ok(AdsStatusResponse { ads_watched_today: watched.0, daily_cap: AD_DAILY_CAP, cooldown_remaining_seconds: cooldown_remaining })
}
