use chrono::{Duration, Utc};
use sqlx::SqlitePool;

/// Doc 8 §10: "Track accounts-per-IP, logins-per-IP, deposits-per-IP over
/// rolling windows ... mainly used to raise the bar slightly for CAPTCHA
/// triggering and to weight other simultaneous signals more heavily."
/// Deliberately does NOT auto-block on its own — §10 explicitly says
/// known-VPN/proxy IPs are "not auto-blocked."
pub async fn accounts_per_ip_last_24h(pool: &SqlitePool, ip_address: &str) -> Result<i64, sqlx::Error> {
    let window_start = (Utc::now() - Duration::hours(24)).to_rfc3339();
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM sessions WHERE ip_address = ? AND created_at > ?"
    )
    .bind(ip_address)
    .bind(&window_start)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn logins_per_ip_last_hour(pool: &SqlitePool, ip_address: &str) -> Result<i64, sqlx::Error> {
    let window_start = (Utc::now() - Duration::hours(1)).to_rfc3339();
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sessions WHERE ip_address = ? AND created_at > ?"
    )
    .bind(ip_address)
    .bind(&window_start)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// §10: "raise the bar slightly for CAPTCHA triggering" — a simple
/// heuristic threshold the CAPTCHA trigger check (§14) can fold in
/// alongside its other conditions.
pub fn ip_context_is_elevated(accounts_per_ip: i64, logins_per_ip: i64) -> bool {
    accounts_per_ip > 5 || logins_per_ip > 10
}
