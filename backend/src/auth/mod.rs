pub mod errors;
pub mod validation;
pub mod password;
pub mod jwt;
pub mod session;
pub mod register;
pub mod register_intent;
pub mod login;
pub mod two_fa;
pub mod forgot_password;
pub mod github_users;
pub mod github_sessions;
pub mod github_wallet;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::Serialize;
use serde_json::json;

use crate::AppState;
use errors::AuthError;
use jwt::issue_access_token;
use session::{create_session, list_sessions, revoke_session, logout as session_logout, rotate_refresh_token, NewSessionInput};

#[derive(Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Doc 9 Sec1: several responses embed a `user` object alongside tokens
/// (verify-email, login/2fa). This is that shared shape.
#[derive(Serialize)]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub coin_balance: i64,
    pub rating: i64,
}

async fn fetch_public_user(state: &AppState, user_id: &str) -> Result<PublicUser, AuthError> {
    #[derive(sqlx::FromRow)]
    struct Row { id: String, username: String, email: String, role: String, coin_balance: i64, rating: i64 }
    let row: Row = sqlx::query_as("SELECT id, username, email, role, coin_balance, rating FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(AuthError::from)?;
    Ok(PublicUser { id: row.id, username: row.username, email: row.email, role: row.role, coin_balance: row.coin_balance, rating: row.rating })
}

async fn issue_tokens_for_new_session(state: &AppState, user_id: &str, role: &str, device_fingerprint: Option<&str>, ip: Option<&str>, browser: Option<&str>, os: Option<&str>) -> Result<TokenPair, AuthError> {
    if let Some(fp) = device_fingerprint {
        notify_if_new_device(state, user_id, fp, browser, os).await;
    }
    let issued = create_session(&state.db, NewSessionInput {
        user_id,
        device_fingerprint,
        ip_address: ip,
        browser,
        os,
    }, state.github_store.as_deref()).await?;
    let access = issue_access_token(user_id, role, &issued.session_id, &state.config.jwt_secret)?;
    Ok(TokenPair { access_token: access, refresh_token: issued.refresh_token_plain })
}

/// Doc 2 §5: alert the account owner when a session starts from a
/// device fingerprint that's never been seen for this account before -
/// but not on the account's very first-ever session (that's just
/// registration finishing; nothing to compare against yet, and the
/// welcome email already greets that moment).
async fn notify_if_new_device(state: &AppState, user_id: &str, fingerprint: &str, browser: Option<&str>, os: Option<&str>) {
    let has_any_session: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM sessions WHERE user_id = ? LIMIT 1")
        .bind(user_id).fetch_optional(&state.db).await.unwrap_or(None);
    if has_any_session.is_none() {
        return;
    }
    let seen_before: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM sessions WHERE user_id = ? AND device_fingerprint = ? LIMIT 1")
        .bind(user_id).bind(fingerprint).fetch_optional(&state.db).await.unwrap_or(None);
    if seen_before.is_some() {
        return;
    }
    if let Ok(user) = fetch_public_user(state, user_id).await {
        let approx_time = chrono::Utc::now().format("%b %d, %Y %H:%M UTC").to_string();
        let _ = state.email.send_new_device_login_email(
            &user.email,
            browser.unwrap_or("an unknown browser"),
            os.unwrap_or("an unknown device"),
            &approx_time,
            &state.config.frontend_base_url,
        ).await;
    }
}

// ---------------------------------------------------------
// POST /auth/register  (Doc 9 Sec1: response { status: "verify_email_sent" })
// ---------------------------------------------------------
async fn register_handler(State(state): State<AppState>, Json(req): Json<register::RegisterRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    let store = state.github_store.as_deref();
    let resp = register::register(&state.db, req, &state.email, &state.config.frontend_base_url, store).await?;
    Ok(Json(json!({
        "status": if resp.email_sent { "verify_email_sent" } else { "verify_email_pending" },
        "email_sent": resp.email_sent,
        "message": resp.message,
    })))
}

// ---------------------------------------------------------
// POST /auth/verify-email  (Doc 9 Sec1: response { access_token, refresh_token, user })
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct VerifyEmailRequest { token: String, device_fingerprint: Option<String>, ip_address: Option<String>, browser: Option<String>, os: Option<String> }

#[derive(Serialize)]
struct AuthWithUserResponse { access_token: String, refresh_token: String, user: PublicUser }

async fn verify_email_handler(State(state): State<AppState>, Json(req): Json<VerifyEmailRequest>) -> Result<Json<AuthWithUserResponse>, AuthError> {
    let (user_id, was_email_change) = register::verify_email(&state.db, &req.token).await?;
    let tokens = issue_tokens_for_new_session(&state, &user_id, "user", req.device_fingerprint.as_deref(), req.ip_address.as_deref(), req.browser.as_deref(), req.os.as_deref()).await?;
    let user = fetch_public_user(&state, &user_id).await?;
    if !was_email_change {
        let _ = state.email.send_welcome_email(&user.email, &user.username, &state.config.frontend_base_url).await;
    }
    Ok(Json(AuthWithUserResponse { access_token: tokens.access_token, refresh_token: tokens.refresh_token, user }))
}

// ---------------------------------------------------------
// POST /auth/resend-verification  (Doc 9 Sec1: response { next_resend_available_at })
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct ResendVerificationRequest { email: String }

#[derive(Serialize)]
struct ResendVerificationResponse { next_resend_available_at: Option<String> }

async fn resend_verification_handler(State(state): State<AppState>, Json(req): Json<ResendVerificationRequest>) -> Result<Json<ResendVerificationResponse>, AuthError> {
    let email_lower = req.email.to_lowercase();
    let row: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(&email_lower).fetch_optional(&state.db).await.map_err(AuthError::from)?;
    let Some((user_id,)) = row else {
        // Doc 3 Sec2.5 / Doc 9: never leak whether the email exists.
        return Ok(Json(ResendVerificationResponse { next_resend_available_at: None }));
    };

    // Resend-count tracked as however many verification tokens already
    // exist for this user (each resend issues one, per register.rs).
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_verification_tokens WHERE user_id = ?")
        .bind(&user_id).fetch_one(&state.db).await.map_err(AuthError::from)?;
    let last_sent: Option<(String,)> = sqlx::query_as(
        "SELECT created_at FROM email_verification_tokens WHERE user_id = ? ORDER BY created_at DESC LIMIT 1"
    ).bind(&user_id).fetch_optional(&state.db).await.map_err(AuthError::from)?;
    let last_sent_dt = last_sent.and_then(|(s,)| chrono::DateTime::parse_from_rfc3339(&s).ok()).map(|d| d.with_timezone(&chrono::Utc));

    // Existing tokens minus the one issued at registration itself = how
    // many resends have already happened.
    let resend_count = count.0.saturating_sub(1) as u32;

    register::resend_verification(&state.db, &user_id, last_sent_dt, resend_count, &state.email, &state.config.frontend_base_url).await?;

    let wait_secs = register::resend_backoff_seconds(resend_count).unwrap_or(0);
    let next_available = (chrono::Utc::now() + chrono::Duration::seconds(wait_secs)).to_rfc3339();
    Ok(Json(ResendVerificationResponse { next_resend_available_at: Some(next_available) }))
}

// ---------------------------------------------------------
// POST /auth/login  (Doc 9 Sec1: response
// { requires_2fa, requires_device_approval, access_token?, refresh_token? })
// ---------------------------------------------------------
#[derive(Serialize)]
struct LoginResponse {
    requires_2fa: bool,
    requires_device_approval: bool,
    /// Phase 4: client must solve captcha and resubmit login
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    requires_captcha: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    captcha: Option<crate::anticheat::captcha::CaptchaChallenge>,
    pending_id: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

async fn login_handler(State(state): State<AppState>, Json(req): Json<login::LoginRequest>) -> Result<Json<LoginResponse>, AuthError> {
    let store = state.github_store.as_deref();
    let identifier_lower = req.identifier.to_lowercase();
    let fails = login::count_recent_failures(&state.db, &identifier_lower).await.unwrap_or(0);

    // Phase 4: after 3 fails, require chess CAPTCHA before credential check
    if fails >= login::CAPTCHA_AFTER_FAILURES {
        let has_captcha = req.captcha_challenge_id.as_ref().filter(|s| !s.is_empty()).is_some()
            && req.captcha_answer.as_ref().filter(|s| !s.is_empty()).is_some();
        if !has_captcha {
            let challenge = crate::anticheat::captcha::generate_challenge(&state.db)
                .await
                .map_err(|_| AuthError::Internal)?;
            crate::anticheat::monitor::log_captcha_required(&identifier_lower);
            return Ok(Json(LoginResponse {
                requires_2fa: false,
                requires_device_approval: false,
                requires_captcha: true,
                captcha: Some(challenge),
                pending_id: None,
                access_token: None,
                refresh_token: None,
            }));
        }
        let ok = crate::anticheat::captcha::verify_captcha(
            &state.db,
            crate::anticheat::captcha::VerifyCaptchaRequest {
                challenge_id: req.captcha_challenge_id.clone().unwrap_or_default(),
                answer: req.captcha_answer.clone().unwrap_or_default(),
            },
        )
        .await
        .map_err(|_| AuthError::Internal)?;
        if !ok {
            crate::anticheat::monitor::log_captcha_failed();
            return Err(AuthError::CaptchaRequired);
        }
    }

    let user = login::login_step_1_verify_credentials(&state.db, &req, store).await?;

    if user.two_fa_enabled == 0 {
        let tokens = issue_tokens_for_new_session(&state, &user.id, &user.role, req.device_fingerprint.as_deref(), req.ip_address.as_deref(), req.browser.as_deref(), req.os.as_deref()).await?;
        return Ok(Json(LoginResponse {
            requires_2fa: false, requires_device_approval: false, requires_captcha: false, captcha: None, pending_id: None,
            access_token: Some(tokens.access_token), refresh_token: Some(tokens.refresh_token),
        }));
    }

    let (case, _old_session_id) = two_fa::determine_device_case(&state.db, &user.id).await?;
    match case {
        two_fa::DeviceCase::NoActiveSession | two_fa::DeviceCase::ActiveButOffline => {
            let pending_id = two_fa::create_pending_2fa(&state.db, &user.id, req.device_fingerprint.as_deref(), false).await?;
            Ok(Json(LoginResponse { requires_2fa: true, requires_device_approval: false, requires_captcha: false, captcha: None, pending_id: Some(pending_id), access_token: None, refresh_token: None }))
        }
        two_fa::DeviceCase::ActiveAndOnline => {
            let pending_id = two_fa::create_pending_2fa(&state.db, &user.id, req.device_fingerprint.as_deref(), true).await?;
            Ok(Json(LoginResponse { requires_2fa: true, requires_device_approval: true, requires_captcha: false, captcha: None, pending_id: Some(pending_id), access_token: None, refresh_token: None }))
        }
    }
}

// ---------------------------------------------------------
// POST /auth/login/2fa  (Doc 9 Sec1: body { pending_id, code },
// response { access_token, refresh_token, user })
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct SubmitCodeRequest { pending_id: String, code: String, device_fingerprint: Option<String>, ip_address: Option<String>, browser: Option<String>, os: Option<String> }

async fn login_2fa_handler(State(state): State<AppState>, Json(req): Json<SubmitCodeRequest>) -> Result<Json<AuthWithUserResponse>, AuthError> {
    let pending = two_fa::get_pending_for_approval(&state.db, &req.pending_id).await?;
    two_fa::expire_stale_approval_if_needed(&state.db, &pending).await?;

    let old_session = session::find_active_session(&state.db, &pending.user_id).await?;
    let old_session_id = old_session.map(|s| s.id);

    let user_id = two_fa::submit_2fa_code(&state.db, &req.pending_id, &req.code, old_session_id.as_deref()).await?;

    let user = fetch_public_user(&state, &user_id).await?;
    let tokens = issue_tokens_for_new_session(&state, &user_id, &user.role, req.device_fingerprint.as_deref(), req.ip_address.as_deref(), req.browser.as_deref(), req.os.as_deref()).await?;
    Ok(Json(AuthWithUserResponse { access_token: tokens.access_token, refresh_token: tokens.refresh_token, user }))
}

// ---------------------------------------------------------
// POST /auth/login/device-approval-response  (Doc 9 Sec1: authenticated,
// old device; body { pending_id, decision: "approve"|"deny" })
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct DeviceApprovalRequest { pending_id: String, decision: String }

async fn device_approval_response_handler(State(state): State<AppState>, Json(req): Json<DeviceApprovalRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    let approved = match req.decision.as_str() {
        "approve" => true,
        "deny" => false,
        _ => return Err(AuthError::Unauthorized),
    };
    two_fa::respond_to_device_approval(&state.db, &req.pending_id, approved).await?;
    Ok(Json(json!({ "status": if approved { "approved" } else { "denied" } })))
}

// ---------------------------------------------------------
// GET /auth/login/device-approval-status/:pending_id  (§5 Case C: the
// waiting NEW device polls this to find out once the OLD device has
// responded - unauthenticated, since the new device has no session yet;
// the random, unguessable pending_id itself is the access control).
// ---------------------------------------------------------
async fn device_approval_status_handler(State(state): State<AppState>, Path(pending_id): Path<String>) -> Result<Json<serde_json::Value>, AuthError> {
    let pending = two_fa::get_pending_for_approval(&state.db, &pending_id).await?;
    two_fa::expire_stale_approval_if_needed(&state.db, &pending).await?;
    // Re-fetch in case the expiry check above just changed it.
    let current = two_fa::get_pending_for_approval(&state.db, &pending_id).await?;
    Ok(Json(json!({ "status": current.approval_status })))
}

// ---------------------------------------------------------
// POST /auth/refresh
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct RefreshRequest { refresh_token: String }

async fn refresh_handler(State(state): State<AppState>, Json(req): Json<RefreshRequest>) -> Result<Json<TokenPair>, AuthError> {
    let (user_id, issued) = rotate_refresh_token(&state.db, &req.refresh_token, state.github_store.as_deref()).await?;
    let role = match sqlx::query_as::<_, (String,)>("SELECT role FROM users WHERE id = ?")
        .bind(&user_id).fetch_optional(&state.db).await.map_err(AuthError::from)? {
        Some((r,)) => r,
        None => {
            if let Some(store) = state.github_store.as_deref() {
                github_users::get_user(store, &user_id).await?
                    .map(|u| u.role)
                    .unwrap_or_else(|| "user".into())
            } else {
                "user".into()
            }
        }
    };
    let access = issue_access_token(&user_id, &role, &issued.session_id, &state.config.jwt_secret)?;
    Ok(Json(TokenPair { access_token: access, refresh_token: issued.refresh_token_plain }))
}

// ---------------------------------------------------------
// POST /auth/logout  (Doc 9 Sec1: response { status: "logged_out" })
// ---------------------------------------------------------
async fn logout_handler(State(state): State<AppState>, Extension(claims): Extension<jwt::AccessClaims>) -> Result<Json<serde_json::Value>, AuthError> {
    session_logout(&state.db, &claims.session_id).await?;
    Ok(Json(json!({ "status": "logged_out" })))
}

// ---------------------------------------------------------
// POST /auth/forgot-password  (response { status: "if_registered_email_sent" })
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct ForgotPasswordRequest { email: String }

async fn forgot_password_handler(State(state): State<AppState>, Json(req): Json<ForgotPasswordRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    forgot_password::request_password_reset(&state.db, &req.email, &state.email, &state.config.frontend_base_url).await?;
    Ok(Json(json!({ "status": "if_registered_email_sent" })))
}

// ---------------------------------------------------------
// POST /auth/reset-password  (response { status: "password_updated" })
// ---------------------------------------------------------
async fn reset_password_handler(State(state): State<AppState>, Json(req): Json<forgot_password::ResetPasswordRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    forgot_password::reset_password(&state.db, req).await?;
    Ok(Json(json!({ "status": "password_updated" })))
}

// ---------------------------------------------------------
// POST /auth/2fa/enable  (Doc 9 Sec1: response { status: "enabled" })
// ---------------------------------------------------------
async fn enable_2fa_handler(State(state): State<AppState>, Extension(claims): Extension<jwt::AccessClaims>, Json(req): Json<two_fa::Enable2FaRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    two_fa::enable_2fa(&state.db, &claims.sub, req).await?;
    let user = fetch_public_user(&state, &claims.sub).await?;
    let _ = state.email.send_2fa_status_email(&user.email, true, &state.config.frontend_base_url).await;
    Ok(Json(json!({ "status": "enabled" })))
}

// ---------------------------------------------------------
// POST /auth/2fa/disable  (response { status: "disabled" })
// ---------------------------------------------------------
async fn disable_2fa_handler(State(state): State<AppState>, Extension(claims): Extension<jwt::AccessClaims>, Json(req): Json<two_fa::Disable2FaRequest>) -> Result<Json<serde_json::Value>, AuthError> {
    two_fa::disable_2fa(&state.db, &claims.sub, req).await?;
    let user = fetch_public_user(&state, &claims.sub).await?;
    let _ = state.email.send_2fa_status_email(&user.email, false, &state.config.frontend_base_url).await;
    Ok(Json(json!({ "status": "disabled" })))
}

// ---------------------------------------------------------
// GET /auth/sessions  (response { sessions: [...] })
// ---------------------------------------------------------
#[derive(Serialize)]
struct SessionsResponse { sessions: Vec<session::SessionListRow> }

async fn list_sessions_handler(State(state): State<AppState>, Extension(claims): Extension<jwt::AccessClaims>) -> Result<Json<SessionsResponse>, AuthError> {
    let sessions = list_sessions(&state.db, &claims.sub).await?;
    Ok(Json(SessionsResponse { sessions }))
}

// ---------------------------------------------------------
// DELETE /auth/sessions/{session_id}  (response { status: "revoked" })
// ---------------------------------------------------------
async fn revoke_session_handler(State(state): State<AppState>, Extension(claims): Extension<jwt::AccessClaims>, Path(session_id): Path<String>) -> Result<Json<serde_json::Value>, AuthError> {
    revoke_session(&state.db, &claims.sub, &session_id).await?;
    Ok(Json(json!({ "status": "revoked" })))
}


// ---------------------------------------------------------
// POST /auth/register-intent  (Part 10: email only → complete-signup link)
// ---------------------------------------------------------
async fn register_intent_handler(
    State(state): State<AppState>,
    Json(req): Json<register_intent::RegisterIntentRequest>,
) -> Result<Json<register_intent::RegisterIntentResponse>, AuthError> {
    let resp = register_intent::create_intent(
        &state.email,
        &state.config.frontend_base_url,
        state.github_store.as_deref(),
        req,
    )
    .await?;
    Ok(Json(resp))
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register-intent", post(register_intent_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/verify-email", post(verify_email_handler))
        .route("/auth/resend-verification", post(resend_verification_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/login/2fa", post(login_2fa_handler))
        .route("/auth/login/device-approval-status/:pending_id", get(device_approval_status_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/reset-password", post(reset_password_handler))
}

/// Doc 9 Sec1: "/auth/login/device-approval-response — Authenticated (old
/// device)" — this one IS protected (unlike the rest of the login flow),
/// since it's the already-logged-in old device responding, not an
/// anonymous login attempt.
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login/device-approval-response", post(device_approval_response_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/2fa/enable", post(enable_2fa_handler))
        .route("/auth/2fa/disable", post(disable_2fa_handler))
        .route("/auth/sessions", get(list_sessions_handler))
        .route("/auth/sessions/:session_id", delete(revoke_session_handler))
}
