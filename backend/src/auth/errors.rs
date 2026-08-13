use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

/// Every message here is copied verbatim from 02_AUTH_SYSTEM.md so the
/// frontend never has to guess wording. Do not reword these.
#[derive(Debug)]
pub enum AuthError {
    // --- Register (§2.4) ---
    UsernameTaken,
    EmailTaken,
    UsernameFormatInvalid,
    EmailFormatInvalid,
    PasswordTooWeak,
    ReservedUsername,

    // --- Login (§3.2, §3.3) ---
    InvalidCredentials,               // generic — never reveal which field was wrong
    EmailNotVerified,
    AccountLocked { retry_after_secs: i64 },

    // --- 2FA (§4) ---
    TwoFaCodeIncorrect,
    TwoFaLockout { retry_after_secs: i64 },
    ReAuthRequired,                   // wrong password/code when enabling/disabling 2FA
    /// Doc 9 §12: "2FA is mandatory for any account with a role other
    /// than `user` (cannot be disabled)."
    TwoFaMandatoryForRole,

    // --- Device / session (§5) ---
    LoginRequestDenied,               // "Login request denied from your other device."

    // --- Password reset (§6) ---
    ResetTokenInvalidOrExpired,

    // --- Session / JWT (§7, §8) ---
    Unauthorized,
    RefreshTokenReuseDetected,        // stolen-token signal — session chain killed

    // --- Verification email resend (§2.5) ---
    ResendTooSoon { retry_after_secs: i64 },
    AlreadyVerified,
    ResendLimitExceeded,              // "Contact support" state, beyond 4th resend

    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthError::UsernameTaken => (StatusCode::CONFLICT, "username_taken", "This username already exists.".to_string()),
            AuthError::EmailTaken => (StatusCode::CONFLICT, "email_taken", "This email is already registered.".to_string()),
            AuthError::UsernameFormatInvalid => (StatusCode::BAD_REQUEST, "username_invalid", "Invalid username format.".to_string()),
            AuthError::EmailFormatInvalid => (StatusCode::BAD_REQUEST, "email_invalid", "Invalid email format.".to_string()),
            AuthError::PasswordTooWeak => (StatusCode::BAD_REQUEST, "password_weak", "Password is too weak.".to_string()),
            AuthError::ReservedUsername => (StatusCode::BAD_REQUEST, "username_reserved", "This username is not available.".to_string()),

            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials", "Invalid username/email or password.".to_string()),
            AuthError::EmailNotVerified => (StatusCode::FORBIDDEN, "email_not_verified", "Please verify your email before logging in.".to_string()),
            AuthError::AccountLocked { retry_after_secs } => (StatusCode::TOO_MANY_REQUESTS, "account_locked", format!("Too many failed attempts. Try again in {retry_after_secs} seconds.")),

            AuthError::TwoFaCodeIncorrect => (StatusCode::UNAUTHORIZED, "2fa_incorrect", "Incorrect 2FA code.".to_string()),
            AuthError::TwoFaLockout { retry_after_secs } => (StatusCode::TOO_MANY_REQUESTS, "2fa_locked", format!("Too many incorrect codes. Try again in {retry_after_secs} seconds.")),
            AuthError::ReAuthRequired => (StatusCode::UNAUTHORIZED, "reauth_required", "Password or 2FA code incorrect.".to_string()),
            AuthError::TwoFaMandatoryForRole => (StatusCode::FORBIDDEN, "2fa_mandatory", "2FA cannot be disabled for admin accounts.".to_string()),

            AuthError::LoginRequestDenied => (StatusCode::FORBIDDEN, "login_denied", "Login request denied from your other device.".to_string()),

            AuthError::ResetTokenInvalidOrExpired => (StatusCode::BAD_REQUEST, "reset_token_invalid", "This reset link is invalid or has expired.".to_string()),

            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            AuthError::RefreshTokenReuseDetected => (StatusCode::UNAUTHORIZED, "session_revoked", "Suspicious activity detected. Please log in again.".to_string()),

            AuthError::ResendTooSoon { retry_after_secs } => (StatusCode::TOO_MANY_REQUESTS, "resend_too_soon", format!("Please wait {retry_after_secs} seconds before requesting another email.")),
            AuthError::AlreadyVerified => (StatusCode::BAD_REQUEST, "already_verified", "This email is already verified.".to_string()),
            AuthError::ResendLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "resend_limit", "Please contact support to verify your email.".to_string()),

            AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong. Please try again.".to_string()),
        };

        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in auth: {e:?}");
        AuthError::Internal
    }
}
