use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::AdminError;

#[derive(Debug, Serialize)]
pub struct OverviewStats {
    pub total_users: i64,
    pub active_today: i64,
    pub active_this_week: i64,
    pub revenue_by_gateway_pkr: Vec<(String, i64)>,
    pub ad_reward_coins_this_week: i64,
    pub active_matches_now: i64,
    pub open_bug_reports: i64,
    pub pending_risk_review_count: i64,
    pub recent_admin_actions: Vec<RecentActionRow>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecentActionRow { pub admin_id: String, pub action: String, pub created_at: String }

pub async fn get_overview(pool: &SqlitePool) -> Result<OverviewStats, AdminError> {
    let total_users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;

    let today_start = Utc::now().format("%Y-%m-%dT00:00:00").to_string();
    let active_today: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM sessions WHERE last_seen_at > ?"
    ).bind(&today_start).fetch_one(pool).await?;

    let week_start = (Utc::now() - Duration::days(7)).to_rfc3339();
    let active_this_week: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM sessions WHERE last_seen_at > ?"
    ).bind(&week_start).fetch_one(pool).await?;

    let revenue_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT gateway, COALESCE(SUM(amount_pkr), 0) FROM payment_transactions WHERE status = 'success' GROUP BY gateway"
    ).fetch_all(pool).await?;

    let ad_reward_coins: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(coins_awarded), 0) FROM ad_views WHERE created_at > ? AND verified_server_side = 1"
    ).bind(&week_start).fetch_one(pool).await?;

    let active_matches: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matches WHERE status = 'in_progress'").fetch_one(pool).await?;

    let open_bugs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bug_reports WHERE status = 'open'").fetch_one(pool).await?;

    let pending_review: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM risk_scores WHERE score >= 81").fetch_one(pool).await?;

    let recent_actions = sqlx::query_as::<_, RecentActionRow>(
        "SELECT admin_id, action, created_at FROM admin_audit_log ORDER BY created_at DESC LIMIT 10"
    ).fetch_all(pool).await?;

    Ok(OverviewStats {
        total_users: total_users.0,
        active_today: active_today.0,
        active_this_week: active_this_week.0,
        revenue_by_gateway_pkr: revenue_rows,
        ad_reward_coins_this_week: ad_reward_coins.0,
        active_matches_now: active_matches.0,
        open_bug_reports: open_bugs.0,
        pending_risk_review_count: pending_review.0,
        recent_admin_actions: recent_actions,
    })
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DailyTrendRow { pub day: String, pub count: i64 }

/// Doc 9 Sec3: "simple line/bar charts for growth trends (daily signups,
/// daily revenue, daily active matches) over a selectable time range
/// (7/30/90 days)."
pub async fn daily_signups(pool: &SqlitePool, days: i64) -> Result<Vec<DailyTrendRow>, AdminError> {
    let since = (Utc::now() - Duration::days(days)).to_rfc3339();
    let rows = sqlx::query_as::<_, DailyTrendRow>(
        "SELECT date(created_at) AS day, COUNT(*) AS count FROM users WHERE created_at > ? GROUP BY day ORDER BY day"
    ).bind(&since).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn daily_revenue(pool: &SqlitePool, days: i64) -> Result<Vec<DailyTrendRow>, AdminError> {
    let since = (Utc::now() - Duration::days(days)).to_rfc3339();
    let rows = sqlx::query_as::<_, DailyTrendRow>(
        "SELECT date(created_at) AS day, COALESCE(SUM(amount_pkr), 0) AS count FROM payment_transactions
         WHERE status = 'success' AND created_at > ? GROUP BY day ORDER BY day"
    ).bind(&since).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn daily_active_matches(pool: &SqlitePool, days: i64) -> Result<Vec<DailyTrendRow>, AdminError> {
    let since = (Utc::now() - Duration::days(days)).to_rfc3339();
    let rows = sqlx::query_as::<_, DailyTrendRow>(
        "SELECT date(started_at) AS day, COUNT(*) AS count FROM matches WHERE started_at > ? GROUP BY day ORDER BY day"
    ).bind(&since).fetch_all(pool).await?;
    Ok(rows)
}
