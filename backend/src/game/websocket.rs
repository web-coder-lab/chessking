use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::AppState;
use crate::auth::jwt::verify_access_token;
use super::disconnect::DisconnectTracker;
use super::engine::GameState;
use super::finalize::finalize_match;
use super::matchmaking::QueuedPlayer;
use super::state::MatchSession;

#[derive(Deserialize)]
pub struct WsAuthQuery {
    token: String,
}

/// Doc 3 §8: WebSocket connections carry the JWT as a query param (the
/// browser WebSocket API can't set custom headers on the handshake) and
/// it's verified here exactly like every REST endpoint — no separate,
/// weaker auth path for real-time features.
pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(auth): Query<WsAuthQuery>,
) -> Response {
    match verify_access_token(&auth.token, &state.config.jwt_secret) {
        Ok(claims) => ws.on_upgrade(move |socket| handle_socket(socket, state, claims.sub)),
        Err(_) => Response::builder().status(401).body("unauthorized".into()).unwrap(),
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    JoinQueue { match_type: String },
    ResumeMatch { match_id: String },
    Move { match_id: String, from: String, to: String, promotion: Option<String> },
    Resign { match_id: String },
    WebrtcSignal { match_id: String, payload: serde_json::Value },
    Heartbeat,
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Phase 1: wait for join_queue or resume_match.
    let mut was_resume = false;
    let (match_id, is_white, is_initiator) = loop {
        let Some(Ok(Message::Text(text))) = receiver.next().await else { return };
        let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) else { continue };

        match msg {
            ClientMessage::JoinQueue { match_type } => {
                match wait_for_match(&state, &user_id, &match_type).await {
                    Some(result) => break result,
                    None => return,
                }
            }
            ClientMessage::ResumeMatch { match_id } => {
                was_resume = true;
                match resume_existing_match(&state, &user_id, &match_id).await {
                    Some(result) => break result,
                    None => { let _ = sender.send(Message::Text(json!({"type":"error","message":"Match not found"}).to_string())).await; continue; }
                }
            }
            _ => continue,
        }
    };

    let opponent_id = { get_opponent_id(&state, &match_id, is_white).await };
    let color = if is_white { "white" } else { "black" };
    let _ = sender.send(Message::Text(json!({
        "type": "match_found", "match_id": match_id, "color": color, "opponent_id": opponent_id, "is_initiator": is_initiator
    }).to_string())).await;

    let mut events_rx = match state.match_registry.subscribe(&match_id).await {
        Some(rx) => rx,
        None => return,
    };

    // Doc 7 §4.1 step 5a: this connection resumed an existing match —
    // clear the disconnect-tracker's timer for this side now that we're
    // back, and notify the opponent's client.
    if was_resume {
        notify_reconnected(&state, &match_id, &user_id, is_white).await;
    }

    // Phase 2: the match loop — forward broadcast events to this client,
    // and handle this client's own move/resign/signal/heartbeat messages.
    loop {
        tokio::select! {
            broadcast_msg = events_rx.recv() => {
                match broadcast_msg {
                    Ok(text) => { if sender.send(Message::Text(text)).await.is_err() { break; } }
                    Err(_) => break,
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    Some(Ok(Message::Text(text))) => {
                        if !handle_client_message(&state, &match_id, &user_id, is_white, &text, &mut sender).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }

    // Connection ended (close frame, network drop, or error) — Doc 7 §4.1
    // step 1: "Backend detects disconnect (WebSocket closed unexpectedly
    // / heartbeat timeout)."
    on_socket_closed(&state, &match_id, &user_id, is_white).await;
}

async fn wait_for_match(state: &AppState, user_id: &str, match_type: &str) -> Option<(String, bool, bool)> {
    let (tx, mut rx) = mpsc::channel(1);
    let rating = get_rating(state, user_id).await;

    // Remove any prior queue entry for this user (re-queue / cancel)
    state.matchmaking.leave(user_id).await;

    state.matchmaking.join(match_type, QueuedPlayer {
        user_id: user_id.to_string(),
        rating,
        joined_at: Instant::now(),
        notify: tx,
    }).await;

    // Cap queue wait; FE also cancels by closing the socket (server cleans via timeout).
    let recv = tokio::time::timeout(std::time::Duration::from_secs(180), rx.recv()).await;
    let Some((match_id, opponent_id, is_initiator)) = (match recv {
        Ok(Some(v)) => Some(v),
        _ => {
            state.matchmaking.leave(user_id).await;
            return None;
        }
    }) else {
        state.matchmaking.leave(user_id).await;
        return None;
    };

    // Doc 7 §2 step 4a-b: create the matches row, randomly assign white/
    // black. Only the initiator persists — the other side waits briefly
    // for the row + MatchRegistry entry to appear.
    let is_white = if is_initiator {
        let is_white = rand_bool();
        create_match_row(state, &match_id, user_id, &opponent_id, is_white, match_type).await;
        register_in_memory(state, &match_id, if is_white { user_id } else { &opponent_id }, if is_white { &opponent_id } else { user_id }, match_type).await;
        is_white
    } else {
        // Poll briefly for the initiator's row to land.
        let mut is_white = false;
        for _ in 0..40 {
            if let Some(row) = fetch_match_row(state, &match_id).await {
                is_white = row.0 == user_id;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        is_white
    };

    Some((match_id, is_white, is_initiator))
}

async fn resume_existing_match(state: &AppState, user_id: &str, match_id: &str) -> Option<(String, bool, bool)> {
    let row = fetch_match_row(state, match_id).await?;
    let is_white = row.0 == user_id;
    if !is_white && row.1 != user_id {
        return None; // not a participant in this match
    }
    Some((match_id.to_string(), is_white, false))
}

async fn fetch_match_row(state: &AppState, match_id: &str) -> Option<(String, String)> {
    sqlx::query_as::<_, (String, String)>("SELECT player_white_id, player_black_id FROM matches WHERE id = ? AND status = 'in_progress'")
        .bind(match_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
}

async fn get_opponent_id(state: &AppState, match_id: &str, is_white: bool) -> String {
    if let Some((w, b)) = fetch_match_row(state, match_id).await {
        if is_white { b } else { w }
    } else {
        String::new()
    }
}

async fn get_rating(state: &AppState, user_id: &str) -> i64 {
    sqlx::query_as::<_, (i64,)>("SELECT rating FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .map(|(r,)| r)
        .unwrap_or(1200)
}

fn rand_bool() -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool(0.5)
}

async fn create_match_row(state: &AppState, match_id: &str, a_id: &str, b_id: &str, a_is_white: bool, match_type: &str) {
    let (white_id, black_id) = if a_is_white { (a_id, b_id) } else { (b_id, a_id) };
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT INTO matches (id, player_white_id, player_black_id, match_type, status, white_rating_before, black_rating_before, started_at)
         SELECT ?, ?, ?, ?, 'in_progress', u1.rating, u2.rating, ?
         FROM users u1, users u2 WHERE u1.id = ? AND u2.id = ?"
    )
    .bind(match_id).bind(white_id).bind(black_id).bind(match_type).bind(&now).bind(white_id).bind(black_id)
    .execute(&state.db)
    .await;
}

async fn register_in_memory(state: &AppState, match_id: &str, white_id: &str, black_id: &str, match_type: &str) {
    let (tx, _rx) = tokio::sync::broadcast::channel(32);
    state.match_registry.insert(match_id.to_string(), MatchSession {
        game: GameState::new(),
        white_id: white_id.to_string(),
        black_id: black_id.to_string(),
        match_type: match_type.to_string(),
        events: tx,
        disconnect: DisconnectTracker::new(),
    }).await;
}

/// Returns false if the connection should be closed.
async fn handle_client_message(
    state: &AppState,
    match_id: &str,
    user_id: &str,
    is_white: bool,
    text: &str,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> bool {
    let Ok(msg) = serde_json::from_str::<ClientMessage>(text) else { return true };

    match msg {
        ClientMessage::Move { match_id: _, from, to, promotion } => {
            handle_move(state, match_id, user_id, is_white, &from, &to, promotion.as_deref(), sender).await;
        }
        ClientMessage::Resign { .. } => {
            handle_resign(state, match_id, is_white).await;
        }
        ClientMessage::WebrtcSignal { payload, .. } => {
            // Doc 7 §5.2 step 2: relay SDP offers/answers/ICE candidates
            // between the two players over the existing WS connection —
            // the backend is a signaling server ONLY, audio never routes
            // through it.
            if let Some(tx) = state.match_registry.with_session(match_id, |s| s.events.clone()).await {
                let _ = tx.send(json!({ "type": "webrtc_signal", "from": user_id, "payload": payload }).to_string());
            }
        }
        ClientMessage::Heartbeat => {
            let _ = sender.send(Message::Text(json!({"type":"heartbeat_ack"}).to_string())).await;
        }
        _ => {}
    }
    true
}

async fn handle_move(
    state: &AppState,
    match_id: &str,
    user_id: &str,
    is_white: bool,
    from: &str,
    to: &str,
    promotion: Option<&str>,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) {
    // Doc 7 §3.1 step 3: turn check happens before touching engine state.
    let turn_ok = state.match_registry.with_session(match_id, |s| {
        let side_to_move = s.game.side_to_move();
        let is_my_turn = (side_to_move == shakmaty::Color::White) == is_white;
        is_my_turn
    }).await;

    if turn_ok != Some(true) {
        let _ = sender.send(Message::Text(json!({"type":"error","code":"not_your_turn"}).to_string())).await;
        return;
    }

    let move_result = state.match_registry.with_session(match_id, |s| {
        s.game.try_apply_move(from, to, promotion)
    }).await;

    match move_result {
        Some(Ok(())) => {}
        _ => {
            // §3.1 step c: illegal → reject, error to submitting client
            // ONLY, board state unchanged (nothing was mutated).
            let _ = sender.send(Message::Text(json!({"type":"error","code":"illegal_move"}).to_string())).await;
            return;
        }
    }

    let (game_end, pgn) = state.match_registry.with_session(match_id, |s| {
        (s.game.check_game_end(), s.game.to_pgn())
    }).await.unwrap_or((None, String::new()));

    // §3.1 step 4: broadcast updated state to BOTH clients.
    let fen = state.match_registry.with_session(match_id, |s| s.game.fen()).await.unwrap_or_default();
    if let Some(tx) = state.match_registry.with_session(match_id, |s| s.events.clone()).await {
        let _ = tx.send(json!({
            "type": "board_update",
            "from": from,
            "to": to,
            "promotion": promotion,
            "pgn": pgn,
            "fen": fen
        }).to_string());
    }

    if let Some(outcome) = game_end {
        let (white_id, black_id, match_type) = state.match_registry.with_session(match_id, |s| {
            (s.white_id.clone(), s.black_id.clone(), s.match_type.clone())
        }).await.unwrap_or_default();

        // Doc 1's matches.result_reason CHECK constraint only defines
        // ('checkmate','resign','disconnect_timeout','cheat_detected',
        // 'agreement') — there's no dedicated value for stalemate/
        // threefold-repetition/50-move/insufficient-material, even
        // though Doc 7 §3.1 step d lists all of those as distinct
        // detectable conditions. Decisive results (win/loss) can only
        // mean checkmate here (since resign/disconnect_timeout are set
        // from their own separate call sites, not this one), so
        // "checkmate" is exact for those. For an engine-detected DRAW,
        // "agreement" is the closest existing value — it's not literally
        // accurate (no one agreed to it), but it's the only draw-shaped
        // reason the schema defines, so it's used with this explicit
        // caveat rather than inserting a value the CHECK constraint would
        // reject.
        let reason = match outcome {
            super::engine::GameOutcome::Draw => "agreement",
            _ => "checkmate",
        };
        let _ = finalize_match(&state.db, &state.match_registry, match_id, &white_id, &black_id, outcome, reason, &match_type).await;
    }
}

async fn handle_resign(state: &AppState, match_id: &str, is_white: bool) {
    let (white_id, black_id, match_type) = state.match_registry.with_session(match_id, |s| {
        (s.white_id.clone(), s.black_id.clone(), s.match_type.clone())
    }).await.unwrap_or_default();

    if white_id.is_empty() { return; }

    let outcome = if is_white { super::engine::GameOutcome::BlackWins } else { super::engine::GameOutcome::WhiteWins };
    let _ = finalize_match(&state.db, &state.match_registry, match_id, &white_id, &black_id, outcome, "resign", &match_type).await;
}

async fn on_socket_closed(state: &AppState, match_id: &str, user_id: &str, is_white: bool) {
    let session_info = state.match_registry.with_session(match_id, |s| {
        (s.white_id.clone(), s.black_id.clone(), s.match_type.clone(), s.disconnect.clone())
    }).await;

    let Some((white_id, black_id, match_type, tracker)) = session_info else { return };
    if white_id.is_empty() { return; }

    // Broadcast the reconnect-banner event immediately (Doc 7 §4.1 step 4).
    if let Some(tx) = state.match_registry.with_session(match_id, |s| s.events.clone()).await {
        let _ = tx.send(json!({ "type": "opponent_disconnected", "user_id": user_id, "grace_period_secs": 60 }).to_string());
    }

    tracker.on_disconnect(state.db.clone(), state.match_registry.clone(), match_id.to_string(), white_id, black_id, is_white, match_type).await;
}

/// Called when a player's NEW WebSocket connection resumes a match they
/// were already in (Doc 7 §4.1 step 5a). Wired into `resume_existing_match`
/// implicitly by the caller re-subscribing to the same broadcast channel;
/// this function additionally clears the disconnect-tracker state so the
/// grace-period timer stops.
pub async fn notify_reconnected(state: &AppState, match_id: &str, user_id: &str, is_white: bool) {
    let session_info = state.match_registry.with_session(match_id, |s| {
        (s.white_id.clone(), s.black_id.clone(), s.match_type.clone(), s.disconnect.clone())
    }).await;
    let Some((white_id, black_id, match_type, tracker)) = session_info else { return };

    if let Some(tx) = state.match_registry.with_session(match_id, |s| s.events.clone()).await {
        let _ = tx.send(json!({ "type": "opponent_reconnected", "user_id": user_id }).to_string());
    }

    tracker.on_reconnect(state.db.clone(), state.match_registry.clone(), match_id.to_string(), white_id, black_id, is_white, match_type).await;
}
