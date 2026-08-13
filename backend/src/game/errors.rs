use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum GameError {
    IllegalMove,
    NotYourTurn,
    MatchNotFound,
    MatchAlreadyEnded,
    HintLimitReached,
    InsufficientCoinsForHint,
    HintNotAllowedInRanked,
    InviteNotFound,
    CannotInviteSelf,
    ReceiverNotFound,
    Unauthorized,
    Internal,
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            // §3.1 step c: illegal move rejected, board state unchanged
            GameError::IllegalMove => (StatusCode::BAD_REQUEST, "illegal_move", "That move is not legal.".to_string()),
            GameError::NotYourTurn => (StatusCode::BAD_REQUEST, "not_your_turn", "It's not your turn.".to_string()),
            GameError::MatchNotFound => (StatusCode::NOT_FOUND, "match_not_found", "Match not found.".to_string()),
            GameError::MatchAlreadyEnded => (StatusCode::BAD_REQUEST, "match_ended", "This match has already ended.".to_string()),
            GameError::HintLimitReached => (StatusCode::BAD_REQUEST, "hint_limit_reached", "You've used both hints for this match.".to_string()),
            GameError::InsufficientCoinsForHint => (StatusCode::PAYMENT_REQUIRED, "insufficient_coins", "Not enough coins for this hint.".to_string()),
            GameError::HintNotAllowedInRanked => (StatusCode::FORBIDDEN, "hint_not_allowed", "Hints are not available in ranked matches.".to_string()),
            GameError::InviteNotFound => (StatusCode::NOT_FOUND, "invite_not_found", "Invite not found.".to_string()),
            GameError::CannotInviteSelf => (StatusCode::BAD_REQUEST, "cannot_invite_self", "You can't invite yourself.".to_string()),
            GameError::ReceiverNotFound => (StatusCode::NOT_FOUND, "receiver_not_found", "This user could not be found.".to_string()),
            GameError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Session expired. Please log in again.".to_string()),
            GameError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Something went wrong. Please try again.".to_string()),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for GameError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("db error in game module: {e:?}");
        GameError::Internal
    }
}
