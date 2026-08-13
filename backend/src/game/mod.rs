pub mod errors;
pub mod engine;
pub mod state;
pub mod matchmaking;
pub mod disconnect;
pub mod finalize;
pub mod websocket;
pub mod custom_invite;
pub mod hint;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};

use crate::AppState;
use crate::auth::jwt::AccessClaims;
use errors::GameError;

// ---------------------------------------------------------
// GET /ws/match  (Doc 7 §2, §3, §5 — the real-time match connection)
// ---------------------------------------------------------
// Wired directly in main.rs (not behind require_auth — WS auth is the
// token query param verified inside websocket::ws_upgrade_handler itself,
// since browsers can't attach an Authorization header to a WS handshake).

// ---------------------------------------------------------
// POST /custom-match/invite  (§6 step 3)
// ---------------------------------------------------------
async fn send_invite_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<custom_invite::SendInviteRequest>,
) -> Result<Json<custom_invite::InviteResponse>, GameError> {
    Ok(Json(custom_invite::send_invite(&state.db, &claims.sub, req).await?))
}

#[derive(serde::Deserialize)]
struct RespondInviteRequest { decision: String } // "accept" | "decline"

#[derive(serde::Serialize)]
struct RespondInviteResponse { status: String, match_id: Option<String> }

/// Doc 9 Sec7: single POST /custom-match/invite/{invite_id}/respond,
/// body { decision: "accept"|"decline"} — consolidates what were two
/// separate endpoints (accept/decline) into the one the API reference
/// documents.
async fn respond_invite_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(invite_id): Path<String>,
    Json(req): Json<RespondInviteRequest>,
) -> Result<Json<RespondInviteResponse>, GameError> {
    match req.decision.as_str() {
        "accept" => {
            let (match_id, _white_id, _black_id) = custom_invite::accept_invite(&state.db, &claims.sub, &invite_id).await?;
            // Both clients now connect to WS /match/{match_id} — the
            // resume-match path registers whichever side connects second
            // into the already-created matches row (the MatchRegistry
            // in-memory entry is lazily created on first WS move, same
            // as any other in-progress match reloaded after a restart).
            Ok(Json(RespondInviteResponse { status: "accepted".to_string(), match_id: Some(match_id) }))
        }
        "decline" => {
            custom_invite::decline_invite(&state.db, &claims.sub, &invite_id).await?;
            Ok(Json(RespondInviteResponse { status: "declined".to_string(), match_id: None }))
        }
        _ => Err(GameError::InviteNotFound),
    }
}

/// Doc 9 Sec7: GET /custom-match/search, query: username — finds a user
/// by (partial) username to send an invite to.
#[derive(serde::Deserialize)]
struct CustomMatchSearchQuery { username: String }

#[derive(serde::Serialize, sqlx::FromRow)]
struct CustomMatchSearchResult { id: String, username: String }

#[derive(serde::Serialize)]
struct CustomMatchSearchResponse { results: Vec<CustomMatchSearchResult> }

async fn custom_match_search_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    axum::extract::Query(q): axum::extract::Query<CustomMatchSearchQuery>,
) -> Result<Json<CustomMatchSearchResponse>, GameError> {
    let pattern = format!("%{}%", q.username.to_lowercase());
    let results = sqlx::query_as::<_, CustomMatchSearchResult>(
        "SELECT id, username FROM users WHERE username_lower LIKE ? AND id != ? LIMIT 20"
    )
    .bind(&pattern)
    .bind(&claims.sub)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(CustomMatchSearchResponse { results }))
}

#[derive(serde::Serialize)]
struct InviteHistoryResponse { invites: Vec<custom_invite::InviteHistoryRow> }

/// Doc 9 Sec7: GET /custom-match/history (renamed from /custom-match/invites).
async fn invite_history_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<InviteHistoryResponse>, GameError> {
    Ok(Json(InviteHistoryResponse { invites: custom_invite::list_invite_history(&state.db, &claims.sub).await? }))
}

// ---------------------------------------------------------
// POST /match/{match_id}/hint  (Doc 9 Sec6, Doc 6 Sec7.3)
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct HintBody { paid_via_ad: bool }

async fn hint_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(match_id): Path<String>,
    Json(body): Json<HintBody>,
) -> Result<Json<hint::HintResponse>, GameError> {
    let req = hint::HintRequest { match_id, paid_via_ad: body.paid_via_ad };
    Ok(Json(hint::request_hint(&state.db, &state.match_registry, &claims.sub, req).await?))
}

// ---------------------------------------------------------
// GET /match/{match_id}  (Doc 9 Sec6: "current match state, for
// reconnect/rehydrate")
// ---------------------------------------------------------
#[derive(serde::Serialize, sqlx::FromRow)]
struct MatchStateRow {
    id: String, player_white_id: String, player_black_id: String,
    match_type: String, status: String, pgn: Option<String>,
    result: Option<String>, result_reason: Option<String>, started_at: String,
}

#[derive(serde::Serialize)]
struct MatchDetailsResponse {
    id: String, match_type: String, status: String, pgn: Option<String>,
    result: Option<String>, result_reason: Option<String>, started_at: String,
    my_color: String, opponent_username: String,
}

async fn get_match_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(match_id): Path<String>,
) -> Result<Json<MatchDetailsResponse>, GameError> {
    let row = sqlx::query_as::<_, MatchStateRow>(
        "SELECT id, player_white_id, player_black_id, match_type, status, pgn, result, result_reason, started_at FROM matches WHERE id = ?"
    )
    .bind(&match_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(GameError::MatchNotFound)?;

    if row.player_white_id != claims.sub && row.player_black_id != claims.sub {
        return Err(GameError::Unauthorized);
    }

    let my_color = if row.player_white_id == claims.sub { "white" } else { "black" };
    let opponent_id = if my_color == "white" { &row.player_black_id } else { &row.player_white_id };
    let opponent: (String,) = sqlx::query_as("SELECT username FROM users WHERE id = ?")
        .bind(opponent_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(MatchDetailsResponse {
        id: row.id, match_type: row.match_type, status: row.status, pgn: row.pgn,
        result: row.result, result_reason: row.result_reason, started_at: row.started_at,
        my_color: my_color.to_string(), opponent_username: opponent.0,
    }))
}

// ---------------------------------------------------------
// POST /reports/voice-abuse  (Doc 9 Sec11, Doc 7 Sec5.4)
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct VoiceReportRequest {
    match_id: String,
    reported_id: String,
    reason: String,
}

async fn voice_report_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<VoiceReportRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO voice_abuse_reports (id, match_id, reporter_id, reported_id, reason, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'open', ?)"
    )
    .bind(&id)
    .bind(&req.match_id)
    .bind(&claims.sub)
    .bind(&req.reported_id)
    .bind(&req.reason)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "status": "submitted" })))
}

// ---------------------------------------------------------
// POST /reports/bug  (Doc 9 Sec11 — doc specifies multipart with an
// optional screenshot file; accepted here as JSON with a screenshot_url
// instead, since the file itself would be uploaded to storage
// separately and only the resulting URL passed through — no object
// storage integration is specified anywhere in Docs 1-9, so a real
// multipart file upload isn't something this backend can wire without
// guessing at infrastructure the spec doesn't define)
// ---------------------------------------------------------
#[derive(serde::Deserialize)]
struct BugReportRequest { title: String, description: Option<String>, screenshot_url: Option<String> }

async fn bug_report_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(req): Json<BugReportRequest>,
) -> Result<Json<serde_json::Value>, GameError> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO bug_reports (id, user_id, title, description, screenshot_url, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'open', ?)"
    )
    .bind(&id)
    .bind(&claims.sub)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.screenshot_url)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "status": "submitted" })))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/custom-match/search", get(custom_match_search_handler))
        .route("/custom-match/invite", post(send_invite_handler))
        .route("/custom-match/invite/:invite_id/respond", post(respond_invite_handler))
        .route("/custom-match/history", get(invite_history_handler))
        .route("/match/:match_id", get(get_match_handler))
        .route("/match/:match_id/hint", post(hint_handler))
        .route("/reports/voice-abuse", post(voice_report_handler))
        .route("/reports/bug", post(bug_report_handler))
}

/// Doc 9 Sec6: three documented WS endpoints (/match/queue,
/// /match/{match_id}, /match/{match_id}/webrtc-signal). This backend's
/// WS handler (websocket.rs) is a single persistent-connection state
/// machine that handles queueing, move play, AND signal relay over ONE
/// socket via a `type` field in each message — a deliberate design
/// choice (avoids a client needing to juggle three separate sockets and
/// keep them in sync). All three documented paths are wired to the SAME
/// handler so the client can connect at whichever URL fits its flow
/// (e.g. connect at /match/queue while queueing, or directly at
/// /match/{match_id} to reconnect) and the message protocol underneath
/// behaves identically either way — see websocket::ws_upgrade_handler
/// for the actual message types.
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/match/queue", get(websocket::ws_upgrade_handler))
        .route("/ws/match/:match_id", get(websocket::ws_upgrade_handler))
        .route("/ws/match/:match_id/webrtc-signal", get(websocket::ws_upgrade_handler))
}
