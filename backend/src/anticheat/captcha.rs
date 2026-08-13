use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::errors::AntiCheatError;
use super::risk_score::{get_score, tier_for_score, RiskTier};

/// Doc 8 Sec14: "no third-party CAPTCHA (Turnstile/hCaptcha/reCAPTCHA) -
/// a custom, chess-themed challenge is used instead." This module has
/// zero external dependencies on any CAPTCHA vendor by design.

#[derive(Debug, Serialize)]
pub struct CaptchaChallenge {
    pub challenge_id: String,
    pub kind: String,     // "tap_the_knight" | "move_to_square" | "which_side_in_check"
    pub board_fen: String,
    pub prompt: String,
}

/// Sec14.1: trigger conditions - checked by callers before sensitive
/// actions (login, register, resend-verification).
pub async fn should_trigger_captcha(
    pool: &SqlitePool,
    user_id: Option<&str>,
    recent_attempt_count: i64,
    bot_signals_present: bool,
) -> Result<bool, AntiCheatError> {
    if recent_attempt_count >= 3 {
        return Ok(true);
    }
    if bot_signals_present {
        return Ok(true);
    }
    if let Some(uid) = user_id {
        let score = get_score(pool, uid).await?;
        if matches!(tier_for_score(score), RiskTier::Elevated | RiskTier::High) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Sec14.2: three chess-themed challenge types. Each stores its correct
/// answer server-side only (never trust a client-supplied "I passed"
/// flag) - verified in verify_captcha below.
pub async fn generate_challenge(pool: &SqlitePool) -> Result<CaptchaChallenge, AntiCheatError> {
    // rand::thread_rng() returns a !Send type (ThreadRng). Scoping its use
    // in this block guarantees it drops before the first .await below, so
    // the generated Future for this async fn stays Send (required by
    // axum's Handler trait).
    let kind = {
        let mut rng = rand::thread_rng();
        ["tap_the_knight", "move_to_square", "which_side_in_check"]
            .choose(&mut rng)
            .unwrap()
            .to_string()
    };

    let (board_fen, prompt, answer) = match kind.as_str() {
        "tap_the_knight" => (
            "8/8/3k4/8/3N4/8/3K4/8 w - - 0 1".to_string(),
            "Tap the Knight".to_string(),
            "d4".to_string(),
        ),
        "move_to_square" => (
            "8/8/8/8/8/3K4/8/8 w - - 0 1".to_string(),
            "Move the King to C3".to_string(),
            "c3".to_string(),
        ),
        _ => (
            "4k3/8/8/8/8/8/4Q3/4K3 b - - 0 1".to_string(),
            "Which side is in check?".to_string(),
            "black".to_string(),
        ),
    };

    let challenge_id = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    sqlx::query(
        "INSERT INTO app_config (key, value, updated_at) VALUES (?, ?, ?)"
    )
    .bind(format!("captcha_answer:{challenge_id}"))
    .bind(serde_json::json!({ "answer": answer, "expires_at": expires_at.to_rfc3339() }).to_string())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(CaptchaChallenge { challenge_id, kind, board_fen, prompt })
}

#[derive(Debug, Deserialize)]
pub struct VerifyCaptchaRequest {
    pub challenge_id: String,
    pub answer: String,
}

pub async fn verify_captcha(pool: &SqlitePool, req: VerifyCaptchaRequest) -> Result<bool, AntiCheatError> {
    let key = format!("captcha_answer:{}", req.challenge_id);
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM app_config WHERE key = ?")
        .bind(&key)
        .fetch_optional(pool)
        .await?;

    let Some((stored,)) = row else { return Ok(false) };
    let parsed: serde_json::Value = serde_json::from_str(&stored).map_err(|_| AntiCheatError::Internal)?;

    let expires_at = parsed.get("expires_at").and_then(|v| v.as_str()).unwrap_or("");
    let expired = chrono::DateTime::parse_from_rfc3339(expires_at)
        .map(|dt| chrono::Utc::now() > dt)
        .unwrap_or(true);

    sqlx::query("DELETE FROM app_config WHERE key = ?").bind(&key).execute(pool).await?;

    if expired {
        return Ok(false);
    }

    let correct = parsed.get("answer").and_then(|v| v.as_str()).unwrap_or("");
    Ok(correct.eq_ignore_ascii_case(req.answer.trim()))
}

/// Sec14.3: bot-behavior signals reported by the frontend (mouse-movement
/// absence, keystroke uniformity, honeypot field filled, too-fast
/// requests). The frontend computes these client-side and reports a
/// simple boolean - the SERVER decides what to do with it, never the
/// client.
#[derive(Debug, Deserialize)]
pub struct BotSignalReport {
    pub honeypot_filled: bool,
    pub mouse_movement_absent: bool,
    pub request_too_fast: bool,
}

pub fn bot_signals_present(report: &BotSignalReport) -> bool {
    report.honeypot_filled || report.mouse_movement_absent || report.request_too_fast
}
