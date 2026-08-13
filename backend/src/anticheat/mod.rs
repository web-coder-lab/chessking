pub mod errors;
pub mod risk_score;
pub mod ban_escalation;
pub mod captcha;
pub mod ad_reward;
pub mod hash_integrity;
pub mod device_fingerprint;
pub mod match_integrity;
pub mod ip_reputation;

use axum::{
    extract::State,
    routing::post,
    Extension, Json, Router,
};

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use errors::AntiCheatError;

// ---------------------------------------------------------
// POST /captcha/generate  (Doc 8 Sec14.2)
// ---------------------------------------------------------
async fn generate_captcha_handler(State(state): State<AppState>) -> Result<Json<captcha::CaptchaChallenge>, AntiCheatError> {
    Ok(Json(captcha::generate_challenge(&state.db).await?))
}

// ---------------------------------------------------------
// POST /captcha/verify
// ---------------------------------------------------------
#[derive(serde::Serialize)]
struct VerifyCaptchaResponse { passed: bool }

async fn verify_captcha_handler(
    State(state): State<AppState>,
    Json(req): Json<captcha::VerifyCaptchaRequest>,
) -> Result<Json<VerifyCaptchaResponse>, AntiCheatError> {
    let passed = captcha::verify_captcha(&state.db, req).await?;
    Ok(Json(VerifyCaptchaResponse { passed }))
}

// ---------------------------------------------------------
// POST /webhooks/ad-reward  (Doc 8 Sec15 - S2S callback FROM the ad
// network, not from our own client; no user JWT, the ad network's own
// signature/shared-secret is the authentication - same pattern as the
// payment gateway webhooks in Doc 5)
// ---------------------------------------------------------
async fn ad_reward_webhook_handler(
    State(state): State<AppState>,
    Json(cb): Json<ad_reward::AdRewardCallback>,
) -> Result<&'static str, AntiCheatError> {
    ad_reward::handle_ad_reward_callback(&state.db, cb).await?;
    Ok("OK")
}

// ---------------------------------------------------------
// POST /security/screen-scan-consent  (Doc 8 Sec16 step 3 - explicit,
// disclosed consent before any ranked match, never assumed)
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct ScreenScanConsentRequest { consented: bool }

async fn screen_scan_consent_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<ScreenScanConsentRequest>,
) -> Result<Json<serde_json::Value>, AntiCheatError> {
    sqlx::query(
        "INSERT INTO app_config (key, value, updated_by, updated_at) VALUES (?, ?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
    )
    .bind(format!("screen_scan_consent:{}", claims.sub))
    .bind(serde_json::json!({ "consented": req.consented, "at": chrono::Utc::now().to_rfc3339() }).to_string())
    .bind(&claims.sub)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await
    .map_err(AntiCheatError::from)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------------------------------------------------------
// POST /security/bot-signal  (Doc 8 Sec14.3)
// ---------------------------------------------------------
async fn bot_signal_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(report): Json<captcha::BotSignalReport>,
) -> Result<Json<serde_json::Value>, AntiCheatError> {
    if captcha::bot_signals_present(&report) {
        risk_score::record_event(
            &state.db, &claims.sub, "bot_tool_detected_on_screen",
            serde_json::json!({ "honeypot": report.honeypot_filled, "mouse": report.mouse_movement_absent, "timing": report.request_too_fast }),
            None, None,
        ).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/captcha/generate", post(generate_captcha_handler))
        .route("/captcha/verify", post(verify_captcha_handler))
        .route("/webhooks/ad-reward", post(ad_reward_webhook_handler))
        // Doc 9 Sec9 documents this path for the same S2S callback.
        .route("/rewards/ad-reward-callback", post(ad_reward_webhook_handler))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/security/screen-scan-consent", post(screen_scan_consent_handler))
        .route("/security/bot-signal", post(bot_signal_handler))
}
