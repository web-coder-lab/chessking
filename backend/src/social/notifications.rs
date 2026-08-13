use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use super::errors::SocialError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NotificationRow {
    pub id: String, pub r#type: String, pub title: String, pub body: Option<String>,
    pub reference_id: Option<String>, pub is_read: i64, pub created_at: String,
}

pub async fn list_notifications(pool: &SqlitePool, user_id: &str, page: i64, limit: i64) -> Result<Vec<NotificationRow>, SocialError> {
    let limit = limit.clamp(1, 100);
    let offset = (page.max(1) - 1) * limit;
    let rows = sqlx::query_as::<_, NotificationRow>(
        "SELECT id, type, title, body, reference_id, is_read, created_at FROM notifications
         WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    ).bind(user_id).bind(limit).bind(offset).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn mark_read(pool: &SqlitePool, user_id: &str, notification_id: &str) -> Result<(), SocialError> {
    let result = sqlx::query("UPDATE notifications SET is_read = 1 WHERE id = ? AND user_id = ?")
        .bind(notification_id).bind(user_id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(SocialError::NotFound);
    }
    Ok(())
}

pub async fn update_settings(pool: &SqlitePool, user_id: &str, enabled: bool) -> Result<(), SocialError> {
    sqlx::query(
        "INSERT INTO notification_settings (user_id, enabled, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at"
    )
    .bind(user_id).bind(enabled as i64).bind(Utc::now().to_rfc3339())
    .execute(pool).await?;
    Ok(())
}

/// Helper other modules can call to create a notification (gift
/// received, custom-match invite, device-approval request, etc.) —
/// referenced as TODO call-sites in earlier phases, now has a real home.
pub async fn create_notification(pool: &SqlitePool, user_id: &str, notif_type: &str, title: &str, body: Option<&str>, reference_id: Option<&str>) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO notifications (id, user_id, type, title, body, reference_id, is_read, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 0, ?)"
    )
    .bind(&id).bind(user_id).bind(notif_type).bind(title).bind(body).bind(reference_id).bind(Utc::now().to_rfc3339())
    .execute(pool).await?;
    Ok(())
}
