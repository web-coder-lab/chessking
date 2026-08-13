use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum SocialError {
    NotFound,
    Unauthorized,
    ValidationFailed(String),
    AlreadyClaimed,
    NotClaimable,
    DailyCapReached,
    Internal,
}

impl IntoResponse for SocialError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            SocialError::NotFound => (StatusCode::NOT_FOUND, "not_found", "Not found.".to_string()),
            SocialError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            SocialError::ValidationFailed(m) => (StatusCode::BAD_REQUEST, "validation_failed", m),
            SocialError::AlreadyClaimed => (StatusCode::BAD_REQUEST, "already_claimed", "Already claimed today.".to_string()),
            SocialError::NotClaimable => (StatusCode::BAD_REQUEST, "not_claimable", "This reward isn't claimable yet.".to_string()),
            SocialError::DailyCapReached => (StatusCode::TOO_MANY_REQUESTS, "daily_cap_reached", "You've reached today's cap.".to_string()),
            SocialError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for SocialError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in social module: {e:?}");
        SocialError::Internal
    }
}
