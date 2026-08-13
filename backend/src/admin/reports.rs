use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::errors::AdminError;
use super::rbac::write_audit_log;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BugReportRow {
    pub id: String, pub user_id: String, pub title: String, pub description: Option<String>,
    pub screenshot_url: Option<String>, pub status: String, pub created_at: String,
}

pub async fn list_bug_reports(pool: &SqlitePool, status: Option<&str>) -> Result<Vec<BugReportRow>, AdminError> {
    let rows = sqlx::query_as::<_, BugReportRow>(
        "SELECT id, user_id, title, description, screenshot_url, status, created_at FROM bug_reports
         WHERE (?1 IS NULL OR status = ?1) ORDER BY created_at DESC LIMIT 100"
    ).bind(status).fetch_all(pool).await?;
    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct UpdateReportStatusRequest { pub status: String }

pub async fn update_bug_report_status(pool: &SqlitePool, admin_id: &str, report_id: &str, req: UpdateReportStatusRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if !["open", "in_review", "resolved"].contains(&req.status.as_str()) {
        return Err(AdminError::ValidationFailed("Invalid status.".to_string()));
    }
    sqlx::query("UPDATE bug_reports SET status = ? WHERE id = ?").bind(&req.status).bind(report_id).execute(pool).await?;
    write_audit_log(pool, admin_id, "update_bug_report_status", None,
        Some(serde_json::json!({"report_id": report_id, "status": req.status})), admin_ip).await
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VoiceAbuseReportRow {
    pub id: String, pub match_id: String, pub reporter_id: String, pub reported_id: String,
    pub reason: Option<String>, pub audio_buffer_url: Option<String>, pub status: String, pub created_at: String,
}

pub async fn list_voice_abuse_reports(pool: &SqlitePool, status: Option<&str>) -> Result<Vec<VoiceAbuseReportRow>, AdminError> {
    let rows = sqlx::query_as::<_, VoiceAbuseReportRow>(
        "SELECT id, match_id, reporter_id, reported_id, reason, audio_buffer_url, status, created_at FROM voice_abuse_reports
         WHERE (?1 IS NULL OR status = ?1) ORDER BY created_at DESC LIMIT 100"
    ).bind(status).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn update_voice_report_status(pool: &SqlitePool, admin_id: &str, report_id: &str, req: UpdateReportStatusRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if !["open", "in_review", "resolved"].contains(&req.status.as_str()) {
        return Err(AdminError::ValidationFailed("Invalid status.".to_string()));
    }
    sqlx::query("UPDATE voice_abuse_reports SET status = ? WHERE id = ?").bind(&req.status).bind(report_id).execute(pool).await?;
    write_audit_log(pool, admin_id, "update_voice_report_status", None,
        Some(serde_json::json!({"report_id": report_id, "status": req.status})), admin_ip).await
}
