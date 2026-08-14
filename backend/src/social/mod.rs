pub mod errors;
pub mod profile;
pub mod referral;
pub mod rewards;
pub mod leaderboard;
pub mod notifications;
pub mod legal;

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use serde::Deserialize;

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use errors::SocialError;

async fn get_my_profile_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<profile::FullProfile>, SocialError> {
    Ok(Json(profile::get_my_profile(&state.db, &claims.sub).await?))
}

async fn get_public_profile_handler(State(state): State<AppState>, Path(username): Path<String>) -> Result<Json<profile::PublicProfile>, SocialError> {
    Ok(Json(profile::get_public_profile(&state.db, &username).await?))
}

async fn update_my_profile_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Json(req): Json<profile::UpdateProfileRequest>) -> Result<Json<profile::FullProfile>, SocialError> {
    Ok(Json(profile::update_my_profile(&state.db, &claims.sub, req).await?))
}

async fn change_email_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Json(req): Json<profile::ChangeEmailRequest>) -> Result<Json<serde_json::Value>, SocialError> {
    profile::request_email_change(&state.db, &claims.sub, req, &state.email, &state.config.frontend_base_url).await?;
    Ok(Json(serde_json::json!({ "status": "verify_new_email_sent" })))
}

async fn change_password_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Json(req): Json<profile::ChangePasswordRequest>) -> Result<Json<serde_json::Value>, SocialError> {
    profile::change_password(&state.db, &claims.sub, req).await?;
    Ok(Json(serde_json::json!({ "status": "password_updated" })))
}

#[derive(Deserialize)]
struct PageQuery { page: Option<i64>, limit: Option<i64> }

#[derive(serde::Serialize)]
struct MatchHistoryResponse { matches: Vec<profile::MatchHistoryRow> }

async fn match_history_handler(State(state): State<AppState>, Path(username): Path<String>, Query(q): Query<PageQuery>) -> Result<Json<MatchHistoryResponse>, SocialError> {
    let matches = profile::match_history(&state.db, &username, q.page.unwrap_or(1), q.limit.unwrap_or(20)).await?;
    Ok(Json(MatchHistoryResponse { matches }))
}

async fn referral_link_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<referral::ReferralLinkResponse>, SocialError> {
    Ok(Json(referral::get_referral_link(&state.db, &claims.sub).await?))
}

#[derive(serde::Serialize)]
struct ReferralProgressResponse { referrals: Vec<referral::ReferralProgressRow> }

async fn referral_progress_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<ReferralProgressResponse>, SocialError> {
    Ok(Json(ReferralProgressResponse { referrals: referral::referral_progress(&state.db, &claims.sub).await? }))
}

async fn claim_referral_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Path(referral_id): Path<String>) -> Result<Json<referral::ClaimReferralResponse>, SocialError> {
    Ok(Json(referral::claim_referral(&state.db, &claims.sub, &referral_id).await?))
}

async fn daily_status_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<rewards::DailyStatusResponse>, SocialError> {
    Ok(Json(rewards::daily_status(&state.db, &claims.sub, state.github_store.as_deref()).await?))
}

async fn daily_claim_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<rewards::DailyClaimResponse>, SocialError> {
    Ok(Json(rewards::claim_daily(&state.db, &claims.sub, state.github_store.as_deref()).await?))
}

async fn ads_status_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>) -> Result<Json<rewards::AdsStatusResponse>, SocialError> {
    Ok(Json(rewards::ads_status(&state.db, &claims.sub).await?))
}

#[derive(Deserialize)]
struct LeaderboardQuery { scope: Option<String>, scope_value: Option<String>, page: Option<i64>, limit: Option<i64> }

async fn leaderboard_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<LeaderboardQuery>) -> Result<Json<leaderboard::LeaderboardResponse>, SocialError> {
    Ok(Json(leaderboard::get_leaderboard(
        &state.db, &claims.sub,
        q.scope.as_deref().unwrap_or("global"), q.scope_value.as_deref(),
        q.page.unwrap_or(1), q.limit.unwrap_or(50),
    ).await?))
}

#[derive(serde::Serialize)]
struct NotificationsResponse { notifications: Vec<notifications::NotificationRow> }

async fn list_notifications_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Query(q): Query<PageQuery>) -> Result<Json<NotificationsResponse>, SocialError> {
    let notifs = notifications::list_notifications(&state.db, &claims.sub, q.page.unwrap_or(1), q.limit.unwrap_or(20)).await?;
    Ok(Json(NotificationsResponse { notifications: notifs }))
}

async fn mark_notification_read_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, SocialError> {
    notifications::mark_read(&state.db, &claims.sub, &id).await?;
    Ok(Json(serde_json::json!({ "status": "read" })))
}

#[derive(Deserialize)]
struct NotifSettingsRequest { enabled: bool }

async fn notification_settings_handler(State(state): State<AppState>, Extension(claims): Extension<AccessClaims>, Json(req): Json<NotifSettingsRequest>) -> Result<Json<serde_json::Value>, SocialError> {
    notifications::update_settings(&state.db, &claims.sub, req.enabled).await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
}

async fn support_info_handler(State(state): State<AppState>) -> Result<Json<legal::SupportInfoResponse>, SocialError> {
    Ok(Json(legal::get_support_info(&state.db).await?))
}

async fn privacy_policy_handler(State(state): State<AppState>) -> Result<Json<legal::ContentResponse>, SocialError> {
    Ok(Json(legal::get_legal_page(&state.db, "privacy_policy").await?))
}

async fn terms_of_service_handler(State(state): State<AppState>) -> Result<Json<legal::ContentResponse>, SocialError> {
    Ok(Json(legal::get_legal_page(&state.db, "terms_of_service").await?))
}

async fn about_handler(State(state): State<AppState>) -> Result<Json<legal::ContentResponse>, SocialError> {
    Ok(Json(legal::get_legal_page(&state.db, "about").await?))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/profile/me", get(get_my_profile_handler).patch(update_my_profile_handler))
        .route("/profile/me/email", post(change_email_handler))
        .route("/profile/me/password", post(change_password_handler))
        .route("/profile/:username", get(get_public_profile_handler))
        .route("/profile/:username/match-history", get(match_history_handler))
        .route("/referral/link", get(referral_link_handler))
        .route("/referral/progress", get(referral_progress_handler))
        .route("/referral/:referral_id/claim", post(claim_referral_handler))
        .route("/rewards/daily-status", get(daily_status_handler))
        .route("/rewards/daily-claim", post(daily_claim_handler))
        .route("/rewards/ads-status", get(ads_status_handler))
        .route("/leaderboard", get(leaderboard_handler))
        .route("/notifications", get(list_notifications_handler))
        .route("/notifications/:id/read", post(mark_notification_read_handler))
        .route("/notifications/settings", patch(notification_settings_handler))
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/support/info", get(support_info_handler))
        .route("/legal/privacy-policy", get(privacy_policy_handler))
        .route("/legal/terms-of-service", get(terms_of_service_handler))
        .route("/legal/about", get(about_handler))
}
