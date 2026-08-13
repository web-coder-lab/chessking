use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::AdminError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: String, pub admin_id: String, pub action: String,
    pub old_value: Option<String>, pub new_value: Option<String>,
    pub ip_address: Option<String>, pub created_at: String,
}

/// Doc 9 Sec11: "Full, searchable, filterable table: admin, action type,
/// timestamp, IP." Read-only - this table cannot be edited or deleted by
/// anyone through the panel UI (append-only). Enforced by absence: there
/// is no update/delete function anywhere in this module for
/// admin_audit_log - only this read path and the write_audit_log helper
/// (rbac.rs) that every other admin action calls.
pub async fn list_audit_log(
    pool: &SqlitePool,
    admin_id_filter: Option<&str>,
    action_filter: Option<&str>,
    page: i64,
) -> Result<Vec<AuditLogRow>, AdminError> {
    let limit = 50i64;
    let offset = (page.max(1) - 1) * limit;

    let rows = sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, admin_id, action, old_value, new_value, ip_address, created_at
         FROM admin_audit_log
         WHERE (?1 IS NULL OR admin_id = ?1)
           AND (?2 IS NULL OR action = ?2)
         ORDER BY created_at DESC LIMIT ?3 OFFSET ?4"
    )
    .bind(admin_id_filter).bind(action_filter).bind(limit).bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
