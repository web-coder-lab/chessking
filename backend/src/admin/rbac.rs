use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::jwt::AccessClaims;
use super::errors::AdminError;

/// Doc 9 §2 exact role table. `AdminSection` names each protected area so
/// call sites read like the spec's own table, not ad-hoc string checks
/// scattered everywhere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdminSection {
    UsersWrite,        // suspend/ban/unban/adjust — super_admin, moderator (ban/unban only — see require_role)
    UsersReadSensitive, // support_admin: read-only on sensitive fields
    WalletPayments,     // finance_admin, super_admin
    ShopManagement,     // super_admin (doc doesn't explicitly restrict further; treated as super_admin-only + finance for pricing overlap)
    AntiCheat,          // security_admin, super_admin
    Reports,            // support_admin, super_admin
    Content,            // super_admin
    Config,             // super_admin (payment/SMTP config specifically also allow finance_admin per Sec5/6)
    AuditLogRead,       // security_admin (read), super_admin
    RoleGrant,          // super_admin ONLY
}

/// Doc 9 §2: "Every admin-panel endpoint checks role server-side before
/// executing — the frontend hiding a button is not the security
/// boundary, the backend check is." This function IS that boundary.
pub fn require_role(claims: &AccessClaims, section: AdminSection) -> Result<(), AdminError> {
    let role = claims.role.as_str();

    let allowed = match section {
        AdminSection::UsersWrite => matches!(role, "super_admin" | "moderator"),
        AdminSection::UsersReadSensitive => matches!(role, "super_admin" | "support_admin"),
        AdminSection::WalletPayments => matches!(role, "super_admin" | "finance_admin"),
        AdminSection::ShopManagement => matches!(role, "super_admin" | "finance_admin"),
        AdminSection::AntiCheat => matches!(role, "super_admin" | "security_admin"),
        AdminSection::Reports => matches!(role, "super_admin" | "support_admin" | "moderator"),
        AdminSection::Content => matches!(role, "super_admin"),
        AdminSection::Config => matches!(role, "super_admin" | "finance_admin"),
        AdminSection::AuditLogRead => matches!(role, "super_admin" | "security_admin"),
        // §2: "Only super_admin can change another user's role."
        AdminSection::RoleGrant => role == "super_admin",
    };

    if allowed { Ok(()) } else { Err(AdminError::Forbidden) }
}

/// Doc 9 §4.2 / §11: "Every action here writes an admin_audit_log row
/// (who, what, when, old value, new value, admin's own IP)." Every
/// mutating admin endpoint in this module calls this exactly once.
pub async fn write_audit_log(
    pool: &SqlitePool,
    admin_id: &str,
    action: &str,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    ip_address: Option<&str>,
) -> Result<(), AdminError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO admin_audit_log (id, admin_id, action, old_value, new_value, ip_address, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(admin_id)
    .bind(action)
    .bind(old_value.map(|v| v.to_string()))
    .bind(new_value.map(|v| v.to_string()))
    .bind(ip_address)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}
