use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AdminError {
    Forbidden,
    ReauthRequired,
    ReauthFailed,
    NotFound,
    ValidationFailed(String),
    Unauthorized,
    Internal,
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            // §2: "Every admin-panel endpoint checks role server-side
            // before executing" — this is what that check returns on failure.
            AdminError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "You don't have permission for this action.".to_string()),
            AdminError::ReauthRequired => (StatusCode::UNAUTHORIZED, "reauth_required", "Please re-enter your password to continue.".to_string()),
            AdminError::ReauthFailed => (StatusCode::UNAUTHORIZED, "reauth_failed", "Incorrect password.".to_string()),
            AdminError::NotFound => (StatusCode::NOT_FOUND, "not_found", "Not found.".to_string()),
            AdminError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, "validation_failed", msg),
            AdminError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            AdminError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for AdminError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in admin module: {e:?}");
        AdminError::Internal
    }
}
