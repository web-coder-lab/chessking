use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AntiCheatError {
    CaptchaRequired,
    CaptchaIncorrect,
    RateLimited,
    Banned,
    Internal,
}

impl IntoResponse for AntiCheatError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AntiCheatError::CaptchaRequired => (StatusCode::FORBIDDEN, "captcha_required", "Please complete the verification challenge.".to_string()),
            AntiCheatError::CaptchaIncorrect => (StatusCode::BAD_REQUEST, "captcha_incorrect", "That wasn't quite right — try again.".to_string()),
            AntiCheatError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Too many requests. Please slow down.".to_string()),
            AntiCheatError::Banned => (StatusCode::FORBIDDEN, "account_banned", "This account has been suspended.".to_string()),
            AntiCheatError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for AntiCheatError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in anticheat: {e:?}");
        AntiCheatError::Internal
    }
}
