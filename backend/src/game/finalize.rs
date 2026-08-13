use chrono::Utc;
use sqlx::SqlitePool;

use super::engine::{calculate_elo, GameOutcome};
use super::state::MatchRegistry;

/// Doc 7 Sec3.1 step 5: "If the move ended the game" — a-d, exactly.
/// Shared by every path that can end a match: checkmate/stalemate (from
/// a move), resignation, and disconnect_timeout (Sec4).
pub async fn finalize_match(
    pool: &SqlitePool,
    registry: &MatchRegistry,
    match_id: &str,
    white_id: &str,
    black_id: &str,
    outcome: GameOutcome,
    result_reason: &str,
    match_type: &str,
) -> Result<(), sqlx::Error> {
    let (result_str, white_rating_before, black_rating_before): (&str, i64, i64) = {
        let rows: (i64,) = sqlx::query_as("SELECT rating FROM users WHERE id = ?").bind(white_id).fetch_one(pool).await?;
        let white_before = rows.0;
        let rows: (i64,) = sqlx::query_as("SELECT rating FROM users WHERE id = ?").bind(black_id).fetch_one(pool).await?;
        let black_before = rows.0;
        let result_str = match outcome {
            GameOutcome::WhiteWins => "white_win",
            GameOutcome::BlackWins => "black_win",
            GameOutcome::Draw => "draw",
        };
        (result_str, white_before, black_before)
    };

    // Doc 7 Sec3.1 step 5b: recalculate ratings ONLY if match_type = ranked.
    let (white_after, black_after) = if match_type == "ranked" {
        let w = calculate_elo(white_rating_before, black_rating_before, outcome, true);
        let b = calculate_elo(black_rating_before, white_rating_before, outcome, false);
        (w, b)
    } else {
        (white_rating_before, black_rating_before)
    };

    let pgn = registry.with_session(match_id, |s| s.game.to_pgn()).await.unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let update_result = sqlx::query(
        "UPDATE matches SET status = 'completed', result = ?, result_reason = ?, pgn = ?,
         white_rating_before = ?, black_rating_before = ?, white_rating_after = ?, black_rating_after = ?, ended_at = ?
         WHERE id = ? AND status != 'completed'"
    )
    .bind(result_str)
    .bind(result_reason)
    .bind(&pgn)
    .bind(white_rating_before)
    .bind(black_rating_before)
    .bind(white_after)
    .bind(black_after)
    .bind(&now)
    .bind(match_id)
    .execute(pool)
    .await?;

    if update_result.rows_affected() == 0 {
        // Lost the race: another trigger (move/resign/disconnect-timeout/
        // anti-cheat) already finalized this exact match. Never apply
        // ratings or broadcast a result twice for the same game.
        return Ok(());
    }

    if match_type == "ranked" {
        sqlx::query("UPDATE users SET rating = ?, updated_at = ? WHERE id = ?")
            .bind(white_after).bind(&now).bind(white_id).execute(pool).await?;
        sqlx::query("UPDATE users SET rating = ?, updated_at = ? WHERE id = ?")
            .bind(black_after).bind(&now).bind(black_id).execute(pool).await?;
    }

    // Broadcast the final state to any still-connected clients so they
    // transition off the board screen immediately.
    if let Some(tx) = registry.with_session(match_id, |s| s.events.clone()).await {
        let _ = tx.send(serde_json::json!({
            "type": "match_ended",
            "result": result_str,
            "result_reason": result_reason,
        }).to_string());
    }

    registry.remove(match_id).await;
    Ok(())
}
