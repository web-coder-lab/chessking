use chrono::Utc;
use sqlx::SqlitePool;

use super::engine::{calculate_elo, GameOutcome};
use super::state::MatchRegistry;

/// Minimum displayed Elo after updates (prevents sub-100 collapse).
const RATING_FLOOR: i64 = 100;

/// Shared by checkmate / resign / timeout / draw / cheat paths.
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
    let white_before: i64 = sqlx::query_as::<_, (i64,)>("SELECT rating FROM users WHERE id = ?")
        .bind(white_id)
        .fetch_one(pool)
        .await?
        .0;
    let black_before: i64 = sqlx::query_as::<_, (i64,)>("SELECT rating FROM users WHERE id = ?")
        .bind(black_id)
        .fetch_one(pool)
        .await?
        .0;

    let result_str = match outcome {
        GameOutcome::WhiteWins => "white_win",
        GameOutcome::BlackWins => "black_win",
        GameOutcome::Draw => "draw",
    };

    // Ranked only — casual/custom leave rating untouched
    let (white_after, black_after) = if match_type == "ranked" {
        let w = calculate_elo(white_before, black_before, outcome, true).max(RATING_FLOOR);
        let b = calculate_elo(black_before, white_before, outcome, false).max(RATING_FLOOR);
        (w, b)
    } else {
        (white_before, black_before)
    };

    let white_delta = white_after - white_before;
    let black_delta = black_after - black_before;

    let pgn = registry
        .with_session(match_id, |s| s.game.to_pgn())
        .await
        .unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let update_result = sqlx::query(
        "UPDATE matches SET status = 'completed', result = ?, result_reason = ?, pgn = ?,
         white_rating_before = ?, black_rating_before = ?, white_rating_after = ?, black_rating_after = ?, ended_at = ?
         WHERE id = ? AND status != 'completed'",
    )
    .bind(result_str)
    .bind(result_reason)
    .bind(&pgn)
    .bind(white_before)
    .bind(black_before)
    .bind(white_after)
    .bind(black_after)
    .bind(&now)
    .bind(match_id)
    .execute(pool)
    .await?;

    if update_result.rows_affected() == 0 {
        // Already finalized by another path — no double rating apply
        return Ok(());
    }

    if match_type == "ranked" {
        sqlx::query("UPDATE users SET rating = ?, updated_at = ? WHERE id = ?")
            .bind(white_after)
            .bind(&now)
            .bind(white_id)
            .execute(pool)
            .await?;
        sqlx::query("UPDATE users SET rating = ?, updated_at = ? WHERE id = ?")
            .bind(black_after)
            .bind(&now)
            .bind(black_id)
            .execute(pool)
            .await?;
    }

    if let Some(tx) = registry.with_session(match_id, |s| s.events.clone()).await {
        let _ = tx.send(
            serde_json::json!({
                "type": "match_ended",
                "result": result_str,
                "result_reason": result_reason,
                "match_type": match_type,
                "white_rating_before": white_before,
                "black_rating_before": black_before,
                "white_rating_after": white_after,
                "black_rating_after": black_after,
                "white_delta": white_delta,
                "black_delta": black_delta,
            })
            .to_string(),
        );
    }

    registry.remove(match_id).await;
    Ok(())
}
