use serde::Deserialize;
use sqlx::SqlitePool;

use crate::email::EmailClient;
use super::errors::AdminError;
use super::rbac::write_audit_log;
use super::reauth::require_reauth;

#[derive(Debug, Deserialize)]
pub struct UpdateSmtpConfigRequest {
    pub admin_password: String,
    pub smtp_email: String,
    pub app_password: String,
}

pub async fn update_smtp_config(
    pool: &SqlitePool,
    config_store: &crate::wallet::config::RuntimeConfigStore,
    admin_id: &str,
    req: UpdateSmtpConfigRequest,
    admin_ip: Option<&str>,
) -> Result<(), AdminError> {
    require_reauth(pool, admin_id, &req.admin_password).await?;

    config_store.set(pool, "smtp_email", &req.smtp_email, admin_id, admin_ip).await.map_err(|_| AdminError::Internal)?;
    // app_password is sensitive - stored under a key ending in "_secret"
    // so RuntimeConfigStore's existing masking (Doc 5 Sec8) picks it up
    // automatically in the audit log, same as payment gateway secrets.
    config_store.set(pool, "smtp_app_secret", &req.app_password, admin_id, admin_ip).await.map_err(|_| AdminError::Internal)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct SendTestEmailRequest {
    pub admin_password: String,
    pub to: String,
}

/// Doc 9 Sec6: "'Send test email' button to verify config works before
/// saving live." Uses whatever is CURRENTLY saved in app_config.
pub async fn send_test_email(
    pool: &SqlitePool,
    config_store: &crate::wallet::config::RuntimeConfigStore,
    admin_id: &str,
    req: SendTestEmailRequest,
) -> Result<(), AdminError> {
    require_reauth(pool, admin_id, &req.admin_password).await?;

    let smtp_email = config_store.get_or(pool, "smtp_email", "").await;
    let smtp_secret = config_store.get_or(pool, "smtp_app_secret", "").await;

    if smtp_email.is_empty() || smtp_secret.is_empty() {
        return Err(AdminError::ValidationFailed("SMTP config is not set yet.".to_string()));
    }

    let client = EmailClient::new("smtp.gmail.com", &smtp_email, &smtp_secret, None);

    client.send_test_email(&req.to).await
        .map_err(|_| AdminError::ValidationFailed("Test email failed to send - check credentials.".to_string()))?;

    write_audit_log(pool, admin_id, "send_test_email", None,
        Some(serde_json::json!({"to": req.to})), None).await
}
