use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;

use super::errors::AdminError;
use super::rbac::write_audit_log;

const VALID_KEYS: [&str; 4] = ["privacy_policy", "about", "support_email", "terms_of_service"];

#[derive(Debug, Deserialize)]
pub struct UpdateStaticPageRequest { pub key: String, pub content: String }

/// Doc 9 §10: "Each save writes an admin_audit_log entry and updates
/// static_pages.updated_by/updated_at ... important given these are
/// legally-relevant documents."
pub async fn update_static_page(pool: &SqlitePool, admin_id: &str, req: UpdateStaticPageRequest, admin_ip: Option<&str>) -> Result<(), AdminError> {
    if !VALID_KEYS.contains(&req.key.as_str()) {
        return Err(AdminError::ValidationFailed("Unknown content key.".to_string()));
    }

    let old: Option<(Option<String>,)> = sqlx::query_as("SELECT content FROM static_pages WHERE key = ?")
        .bind(&req.key).fetch_optional(pool).await?;

    sqlx::query(
        "INSERT INTO static_pages (key, content, updated_by, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET content = excluded.content, updated_by = excluded.updated_by, updated_at = excluded.updated_at"
    )
    .bind(&req.key).bind(&req.content).bind(admin_id).bind(Utc::now().to_rfc3339())
    .execute(pool).await?;

    write_audit_log(pool, admin_id, "update_static_page",
        old.map(|(c,)| serde_json::json!({"content_length": c.map(|s| s.len()).unwrap_or(0)})),
        Some(serde_json::json!({"key": req.key, "content_length": req.content.len()})),
        admin_ip).await
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct StaticPageRow { pub key: String, pub content: Option<String>, pub updated_by: Option<String>, pub updated_at: String }

pub async fn get_static_page(pool: &SqlitePool, key: &str) -> Result<StaticPageRow, AdminError> {
    sqlx::query_as::<_, StaticPageRow>("SELECT key, content, updated_by, updated_at FROM static_pages WHERE key = ?")
        .bind(key).fetch_optional(pool).await?.ok_or(AdminError::NotFound)
}
