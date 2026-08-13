use sqlx::SqlitePool;

use crate::auth::password::verify_password;
use super::errors::AdminError;

/// Doc 9 §12: "Sensitive actions (payment config, SMTP config, manual
/// wallet adjustment, role changes) require re-authentication (current
/// password prompt) immediately before the action executes, even within
/// an already-logged-in session." Called at the top of every such
/// handler with the admin's freshly-submitted password.
pub async fn require_reauth(pool: &SqlitePool, admin_id: &str, submitted_password: &str) -> Result<(), AdminError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
        .bind(admin_id)
        .fetch_optional(pool)
        .await?;
    let Some((hash,)) = row else { return Err(AdminError::Unauthorized) };

    if verify_password(submitted_password, &hash) {
        Ok(())
    } else {
        Err(AdminError::ReauthFailed)
    }
}
