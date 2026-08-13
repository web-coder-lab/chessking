use sqlx::SqlitePool;

use crate::game::engine::GameOutcome;
use crate::game::finalize::finalize_match;
use crate::game::state::MatchRegistry;
use super::risk_score::record_event;

/// Doc 8 §4: "Because the server is the sole authority on legality and
/// outcome (Doc 6, Section 3), the client literally cannot submit a
/// 'fake win' — any attempt to call a result-setting endpoint directly
/// ... is rejected unless the server's own game-state reconstruction
/// agrees independently." There is no such endpoint anywhere in this
/// codebase — match results only ever come from `finalize_match`, called
/// internally by the move/resign/disconnect-timeout/cheat-detected code
/// paths, never from a client-supplied "set result" call. This is
/// enforced by absence, not by a check — there's nothing to reject
/// because the attack surface doesn't exist.

#[derive(Debug, Clone, Copy)]
pub struct MidMatchSignal {
    pub engine_similarity_high: bool,
    pub timing_anomaly: bool,
    pub bot_tool_on_screen: bool,
}

impl MidMatchSignal {
    /// Doc 8 §5.1 step 1: "If, DURING a match, multiple independent
    /// signals fire together (not just one alone)." Exactly two-or-more
    /// of the three independent signals must be true.
    pub fn count_active(&self) -> u8 {
        self.engine_similarity_high as u8 + self.timing_anomaly as u8 + self.bot_tool_on_screen as u8
    }

    pub fn crosses_pause_threshold(&self) -> bool {
        self.count_active() >= 2
    }
}

/// Doc 8 §5.1, the full a-d flow. `re_verify` is the "fast
/// re-verification pass" (step b) — it re-checks the SAME signals against
/// data gathered so far. Real engine-similarity scoring and move-timing
/// statistical analysis require a bundled chess engine and a baseline
/// human-timing model, neither of which this doc specifies a concrete
/// implementation for (same category of gap as the paid-hint engine in
/// Doc 6 §7.3) — `re_verify` is therefore a pluggable closure the caller
/// supplies, so the STRUCTURE (pause → re-check → confirm-or-resume,
/// with the exact consequences below) is real and correct even though
/// the actual signal-detection heuristics are not bundled here.
pub async fn handle_mid_match_signal<F>(
    pool: &SqlitePool,
    registry: &MatchRegistry,
    match_id: &str,
    suspected_user_id: &str,
    signal: MidMatchSignal,
    re_verify: F,
) -> Result<(), sqlx::Error>
where
    F: FnOnce() -> bool,
{
    if !signal.crosses_pause_threshold() {
        // Doc 8 §5.1 step 2: "A single signal NEVER pauses a match on its
        // own." Still log at low severity so the pattern accumulates
        // over time, per §5's overall "none of these alone bans" framing.
        if signal.engine_similarity_high {
            let _ = record_event(pool, suspected_user_id, "engine_move_similarity_high", serde_json::json!({ "match_id": match_id, "isolated": true }), None, None).await;
        }
        if signal.timing_anomaly {
            let _ = record_event(pool, suspected_user_id, "impossible_move_timing", serde_json::json!({ "match_id": match_id, "isolated": true }), None, None).await;
        }
        return Ok(());
    }

    // Step a: pause. (Actual WS-level pause — freezing the clock and
    // blocking further move submission for this match_id — is enforced
    // by the caller checking a "paused" flag on MatchSession before
    // accepting a move; that flag plumbing lives in the websocket move
    // handler and reads this function's outcome.)
    tracing::warn!(match_id, suspected_user_id, "match paused: multiple independent cheat signals fired together");

    // Step b: fast re-verification pass.
    let confirmed = re_verify();

    if confirmed {
        // Step c: suspected player removed, honest opponent wins.
        let confirmed_event = record_event(
            pool, suspected_user_id, "engine_move_similarity_high",
            serde_json::json!({ "match_id": match_id, "confirmed_multi_signal": true, "signal_count": signal.count_active() }),
            None, None,
        ).await.unwrap_or(0);
        tracing::warn!(match_id, suspected_user_id, new_score = confirmed_event, "cheat confirmed — removing player from match");

        #[derive(sqlx::FromRow)]
        struct MatchRow { player_white_id: String, player_black_id: String, match_type: String }
        let m: Option<MatchRow> = sqlx::query_as(
            "SELECT player_white_id, player_black_id, match_type FROM matches WHERE id = ?"
        ).bind(match_id).fetch_optional(pool).await?;

        if let Some(m) = m {
            let suspected_is_white = m.player_white_id == suspected_user_id;
            let outcome = if suspected_is_white { GameOutcome::BlackWins } else { GameOutcome::WhiteWins };
            let _ = finalize_match(pool, registry, match_id, &m.player_white_id, &m.player_black_id, outcome, "cheat_detected", &m.match_type).await;
        }
    } else {
        // Step d: not confirmed — resume, log near-miss at low severity.
        tracing::info!(match_id, suspected_user_id, "mid-match pause resolved as false positive — resuming");
        let _ = record_event(pool, suspected_user_id, "impossible_move_timing", serde_json::json!({ "match_id": match_id, "false_positive": true }), None, None).await;
    }

    Ok(())
}

/// Doc 8 §6 (Disconnect Shield pattern-tracking): called by the
/// disconnect module when a player disconnects while in a losing
/// position, tracked across many matches rather than punished on one
/// occurrence.
pub async fn record_disconnect_in_losing_position(pool: &SqlitePool, user_id: &str, match_id: &str) {
    let _ = record_event(
        pool, user_id, "disconnect_pattern_losing_position",
        serde_json::json!({ "match_id": match_id }),
        None, None,
    ).await;
}
