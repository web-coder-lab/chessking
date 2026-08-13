pub mod errors;
pub mod rbac;
pub mod reauth;
pub mod overview;
pub mod users;
pub mod wallet_admin;
pub mod email_config;
pub mod shop_admin;
pub mod anticheat_dashboard;
pub mod reports;
pub mod content;
pub mod audit_log;

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use serde::Deserialize;

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use errors::AdminError;
use rbac::{require_role, AdminSection};

fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()).map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

async fn overview_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<overview::OverviewStats>, AdminError> {
    require_role(&claims, AdminSection::AuditLogRead)?;
    Ok(Json(overview::get_overview(&state.db).await?))
}

#[derive(Deserialize)]
struct DaysQuery { days: Option<i64> }

async fn trends_signups_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<DaysQuery>) -> Result<Json<Vec<overview::DailyTrendRow>>, AdminError> {
    require_role(&claims, AdminSection::AuditLogRead)?;
    Ok(Json(overview::daily_signups(&state.db, q.days.unwrap_or(30)).await?))
}
async fn trends_revenue_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<DaysQuery>) -> Result<Json<Vec<overview::DailyTrendRow>>, AdminError> {
    require_role(&claims, AdminSection::WalletPayments)?;
    Ok(Json(overview::daily_revenue(&state.db, q.days.unwrap_or(30)).await?))
}
async fn trends_matches_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<DaysQuery>) -> Result<Json<Vec<overview::DailyTrendRow>>, AdminError> {
    require_role(&claims, AdminSection::AuditLogRead)?;
    Ok(Json(overview::daily_active_matches(&state.db, q.days.unwrap_or(30)).await?))
}

#[derive(Deserialize)]
struct SearchQuery { q: String }

async fn search_users_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<SearchQuery>) -> Result<Json<Vec<users::UserSearchRow>>, AdminError> {
    require_role(&claims, AdminSection::UsersReadSensitive)?;
    Ok(Json(users::search_users(&state.db, &q.q).await?))
}

async fn user_detail_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Path(user_id): Path<String>) -> Result<Json<users::UserDetail>, AdminError> {
    require_role(&claims, AdminSection::UsersReadSensitive)?;
    Ok(Json(users::get_user_detail(&state.db, &user_id).await?))
}

async fn suspend_user_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<users::SuspendRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::UsersWrite)?;
    users::suspend_user(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn ban_user_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<users::BanRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::UsersWrite)?;
    users::ban_user(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn unban_user_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::UsersWrite)?;
    users::unban_user(&state.db, &claims.sub, &user_id, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn adjust_risk_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<users::AdjustRiskScoreRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    let new_score = users::adjust_risk_score(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"status": "adjusted", "new_score": new_score})))
}

async fn adjust_wallet_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<users::WalletAdjustmentRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::WalletPayments)?;
    let balance = users::adjust_wallet(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"status": "adjusted", "new_balance": balance})))
}

async fn force_logout_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::UsersWrite)?;
    users::force_logout_all_sessions(&state.db, &claims.sub, &user_id, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn grant_role_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<users::GrantRoleRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::RoleGrant)?;
    users::grant_role(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"status": "role_updated"})))
}

#[derive(Deserialize)]
struct TxQuery { gateway: Option<String>, status: Option<String>, user_id: Option<String>, page: Option<i64> }

async fn list_transactions_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<TxQuery>) -> Result<Json<Vec<wallet_admin::PaymentTxRow>>, AdminError> {
    require_role(&claims, AdminSection::WalletPayments)?;
    Ok(Json(wallet_admin::list_transactions(&state.db, q.gateway.as_deref(), q.status.as_deref(), q.user_id.as_deref(), q.page.unwrap_or(1)).await?))
}

async fn stuck_transactions_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<wallet_admin::PaymentTxRow>>, AdminError> {
    require_role(&claims, AdminSection::WalletPayments)?;
    Ok(Json(wallet_admin::list_stuck_transactions(&state.db).await?))
}

async fn update_payment_config_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<wallet_admin::UpdatePaymentConfigRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Config)?;
    wallet_admin::update_payment_config(&state.db, &state.wallet_config, &claims.sub, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct CoinRateRequest { rate_pkr: i64, current_password: String }

/// Doc 9 Sec14: "PATCH /admin/config/coin-rate | { rate_pkr, current_password }"
/// - a dedicated shortcut over the same generic payment-config write path
/// (which already treats "coin_rate_pkr" as one of its editable keys).
async fn update_coin_rate_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<CoinRateRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Config)?;
    wallet_admin::update_payment_config(&state.db, &state.wallet_config, &claims.sub, wallet_admin::UpdatePaymentConfigRequest {
        admin_password: req.current_password,
        key: "coin_rate_pkr".to_string(),
        value: req.rate_pkr.to_string(),
    }, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn update_coin_package_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<wallet_admin::UpdateCoinPackageRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Config)?;
    wallet_admin::update_coin_package(&state.db, &claims.sub, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn refunds_view_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<wallet_admin::PaymentTxRow>>, AdminError> {
    require_role(&claims, AdminSection::WalletPayments)?;
    Ok(Json(wallet_admin::list_transactions(&state.db, None, Some("refunded"), None, 1).await?))
}

async fn update_smtp_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<email_config::UpdateSmtpConfigRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Config)?;
    email_config::update_smtp_config(&state.db, &state.wallet_config, &claims.sub, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn test_email_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Json(req): Json<email_config::SendTestEmailRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Config)?;
    email_config::send_test_email(&state.db, &state.wallet_config, &claims.sub, req).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn create_shop_item_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<shop_admin::CreateShopItemRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::ShopManagement)?;
    let id = shop_admin::create_shop_item(&state.db, &claims.sub, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"id": id})))
}

async fn update_shop_item_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(item_id): Path<String>, Json(req): Json<shop_admin::UpdateShopItemRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::ShopManagement)?;
    shop_admin::update_shop_item(&state.db, &claims.sub, &item_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn deactivate_shop_item_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(item_id): Path<String>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::ShopManagement)?;
    shop_admin::deactivate_shop_item(&state.db, &claims.sub, &item_id, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn shop_popularity_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<shop_admin::ShopItemPopularityRow>>, AdminError> {
    require_role(&claims, AdminSection::ShopManagement)?;
    Ok(Json(shop_admin::item_popularity(&state.db).await?))
}

#[derive(Deserialize)]
struct TierQuery { tier: Option<String> }

async fn risk_tiers_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<TierQuery>) -> Result<Json<Vec<anticheat_dashboard::RiskTierRow>>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    Ok(Json(anticheat_dashboard::list_elevated_and_high_risk(&state.db, q.tier.as_deref()).await?))
}

async fn pending_review_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<anticheat_dashboard::PendingReviewRow>>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    Ok(Json(anticheat_dashboard::pending_review_queue(&state.db).await?))
}

async fn manual_override_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(user_id): Path<String>, Json(req): Json<anticheat_dashboard::ManualOverrideRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    anticheat_dashboard::manual_override(&state.db, &claims.sub, &user_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn cheat_log_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<anticheat_dashboard::CheatDetectionLogRow>>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    Ok(Json(anticheat_dashboard::cheat_detection_log(&state.db).await?))
}

async fn security_events_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Path(user_id): Path<String>) -> Result<Json<Vec<anticheat_dashboard::SecurityEventRow>>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    Ok(Json(anticheat_dashboard::security_events_for_user(&state.db, &user_id).await?))
}

async fn blacklist_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<Vec<anticheat_dashboard::BlacklistRow>>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    Ok(Json(anticheat_dashboard::list_blacklist(&state.db).await?))
}

async fn remove_blacklist_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(ban_id): Path<String>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::AntiCheat)?;
    anticheat_dashboard::remove_from_blacklist(&state.db, &claims.sub, &ban_id, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct StatusQuery { status: Option<String> }

async fn bug_reports_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<StatusQuery>) -> Result<Json<Vec<reports::BugReportRow>>, AdminError> {
    require_role(&claims, AdminSection::Reports)?;
    Ok(Json(reports::list_bug_reports(&state.db, q.status.as_deref()).await?))
}

async fn update_bug_report_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(report_id): Path<String>, Json(req): Json<reports::UpdateReportStatusRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Reports)?;
    reports::update_bug_report_status(&state.db, &claims.sub, &report_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn voice_reports_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<StatusQuery>) -> Result<Json<Vec<reports::VoiceAbuseReportRow>>, AdminError> {
    require_role(&claims, AdminSection::Reports)?;
    Ok(Json(reports::list_voice_abuse_reports(&state.db, q.status.as_deref()).await?))
}

async fn update_voice_report_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Path(report_id): Path<String>, Json(req): Json<reports::UpdateReportStatusRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Reports)?;
    reports::update_voice_report_status(&state.db, &claims.sub, &report_id, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn get_static_page_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Path(key): Path<String>) -> Result<Json<content::StaticPageRow>, AdminError> {
    require_role(&claims, AdminSection::Content)?;
    Ok(Json(content::get_static_page(&state.db, &key).await?))
}

async fn update_static_page_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, headers: HeaderMap, Json(req): Json<content::UpdateStaticPageRequest>) -> Result<Json<serde_json::Value>, AdminError> {
    require_role(&claims, AdminSection::Content)?;
    content::update_static_page(&state.db, &claims.sub, req, client_ip(&headers).as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct AuditQuery { admin_id: Option<String>, action: Option<String>, page: Option<i64> }

async fn audit_log_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<AuditQuery>) -> Result<Json<Vec<audit_log::AuditLogRow>>, AdminError> {
    require_role(&claims, AdminSection::AuditLogRead)?;
    Ok(Json(audit_log::list_audit_log(&state.db, q.admin_id.as_deref(), q.action.as_deref(), q.page.unwrap_or(1)).await?))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/overview/stats", get(overview_handler))
        .route("/admin/trends/signups", get(trends_signups_handler))
        .route("/admin/trends/revenue", get(trends_revenue_handler))
        .route("/admin/trends/matches", get(trends_matches_handler))
        .route("/admin/users/search", get(search_users_handler))
        .route("/admin/users/:user_id", get(user_detail_handler))
        .route("/admin/users/:user_id/suspend", post(suspend_user_handler))
        .route("/admin/users/:user_id/ban", post(ban_user_handler))
        .route("/admin/users/:user_id/unban", post(unban_user_handler))
        .route("/admin/users/:user_id/risk-score", patch(adjust_risk_handler))
        .route("/admin/users/:user_id/wallet-adjustment", post(adjust_wallet_handler))
        .route("/admin/users/:user_id/force-logout", post(force_logout_handler))
        .route("/admin/users/:user_id/role", patch(grant_role_handler))
        .route("/admin/transactions", get(list_transactions_handler))
        .route("/admin/transactions/stuck", get(stuck_transactions_handler))
        .route("/admin/transactions/refunds", get(refunds_view_handler))
        .route("/admin/config/payment", put(update_payment_config_handler))
        .route("/admin/config/coin-rate", patch(update_coin_rate_handler))
        .route("/admin/config/coin-package", post(update_coin_package_handler))
        .route("/admin/config/smtp", put(update_smtp_handler))
        .route("/admin/config/smtp/test", post(test_email_handler))
        .route("/admin/shop/items", post(create_shop_item_handler))
        .route("/admin/shop/items/:item_id", patch(update_shop_item_handler).delete(deactivate_shop_item_handler))
        .route("/admin/shop/items/:item_id/deactivate", post(deactivate_shop_item_handler))
        .route("/admin/shop/popularity", get(shop_popularity_handler))
        .route("/admin/security/risk-queue", get(risk_tiers_handler))
        .route("/admin/security/events/:user_id", get(security_events_handler))
        .route("/admin/security/pending-review", get(pending_review_handler))
        .route("/admin/security/users/:user_id/override", post(manual_override_handler))
        .route("/admin/security/cheat-log", get(cheat_log_handler))
        .route("/admin/security/blacklist", get(blacklist_handler))
        .route("/admin/security/blacklist/:ban_id", delete(remove_blacklist_handler))
        .route("/admin/reports/bugs", get(bug_reports_handler))
        .route("/admin/reports/bugs/:report_id", patch(update_bug_report_handler))
        .route("/admin/reports/voice-abuse", get(voice_reports_handler))
        .route("/admin/reports/voice-abuse/:report_id", patch(update_voice_report_handler))
        .route("/admin/content/:key", get(get_static_page_handler))
        .route("/admin/content", post(update_static_page_handler))
        .route("/admin/audit-log", get(audit_log_handler))
}
