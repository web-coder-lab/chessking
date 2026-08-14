//! Phase 6 — security monitoring: structured logs + periodic summary.

use sqlx::SqlitePool;
use tracing::{error, info, warn};

/// Canonical security log lines for operators (Render log stream / grep).
pub fn log_probe_blocked(path: &str) {
    warn!(target: "security", event = "probe_blocked", path = %path, "scanner probe blocked");
}

pub fn log_auth_rate_limited(kind: &str) {
    warn!(target: "security", event = "auth_rate_limited", kind = %kind, "auth velocity exceeded");
}

pub fn log_captcha_required(identifier_hint: &str) {
    info!(target: "security", event = "captcha_required", identifier_prefix = %identifier_hint.chars().take(3).collect::<String>(), "login captcha step-up");
}

pub fn log_captcha_failed() {
    warn!(target: "security", event = "captcha_failed", "captcha answer rejected");
}

pub fn log_lockout(identifier_hint: &str) {
    warn!(target: "security", event = "account_lockout", identifier_prefix = %identifier_hint.chars().take(3).collect::<String>(), "temporary login lockout");
}

pub fn log_github_store_error(op: &str, detail: &str) {
    error!(target: "security", event = "github_store_error", op = %op, detail = %detail, "durable store failure");
}

/// Every 15 minutes: summarize recent high-severity security_events (ephemeral SQL).
pub fn spawn_periodic_security_summary(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(15 * 60));
        loop {
            interval.tick().await;
            if let Err(e) = emit_summary(&pool).await {
                tracing::debug!(target: "security", "summary skipped: {e}");
            }
        }
    });
}

async fn emit_summary(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let since = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();

    let login_fails: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM security_events WHERE event_type = 'login_failed' AND created_at > ?",
    )
    .bind(&since)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    let high_sev: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM security_events WHERE severity >= 8 AND created_at > ?",
    )
    .bind(&since)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    let registers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM security_events WHERE event_type = 'register_attempt' AND created_at > ?",
    )
    .bind(&since)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    info!(
        target: "security",
        event = "hourly_summary",
        login_failed_1h = login_fails.0,
        high_severity_1h = high_sev.0,
        register_attempts_1h = registers.0,
        "security summary (last 1h, this process)"
    );

    if high_sev.0 > 0 {
        warn!(
            target: "security",
            event = "high_severity_alert",
            count = high_sev.0,
            "high-severity security events in last hour — review admin dashboard"
        );
    }
    Ok(())
}
