use chrono::Utc;
use serde::{Deserialize, Serialize};
use shakmaty::Position;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::wallet::ledger::{apply_ledger_entry_in_tx, LedgerEntryInput};
use super::engine::GameState;
use super::errors::GameError;
use super::state::MatchRegistry;

#[derive(Debug, Deserialize)]
pub struct HintRequest {
    pub match_id: String,
    /// If the player doesn't have enough coins, the frontend instead
    /// completes an ad-view flow first and sets this true (Sec7.2:
    /// "Watch an ad to unlock this hint" as an alternative to coins).
    pub paid_via_ad: bool,
}

#[derive(Debug, Serialize)]
pub struct HintResponse {
    pub move_suggested: String, // UCI, e.g. "e2e4"
    pub coin_balance: i64,
}

/// Doc 7 Sec7.2: 1st use = 1 coin, 2nd use = 2 coins, max 2 uses/match/player.
fn hint_cost(usage_number: i32) -> i64 {
    match usage_number {
        1 => 1,
        2 => 2,
        _ => 0,
    }
}

/// Doc 7 Sec7.3, steps 1-7 exactly.
pub async fn request_hint(pool: &SqlitePool, registry: &MatchRegistry, user_id: &str, req: HintRequest) -> Result<HintResponse, GameError> {
    #[derive(sqlx::FromRow)]
    struct MatchRow { match_type: String, player_white_id: String, player_black_id: String, status: String }

    let m = sqlx::query_as::<_, MatchRow>(
        "SELECT match_type, player_white_id, player_black_id, status FROM matches WHERE id = ?"
    )
    .bind(&req.match_id)
    .fetch_optional(pool)
    .await?
    .ok_or(GameError::MatchNotFound)?;

    if m.status != "in_progress" {
        return Err(GameError::MatchAlreadyEnded);
    }

    // Sec7.1: "Only in casual and custom matches — never in ranked."
    if m.match_type == "ranked" {
        return Err(GameError::HintNotAllowedInRanked);
    }
    if user_id != m.player_white_id && user_id != m.player_black_id {
        return Err(GameError::Unauthorized);
    }

    // Step 2: usage count for (match_id, user_id) must be < 2
    let used_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM match_hint_usage WHERE match_id = ? AND user_id = ?"
    )
    .bind(&req.match_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if used_count.0 >= 2 {
        return Err(GameError::HintLimitReached);
    }
    let usage_number = (used_count.0 + 1) as i32;
    let cost = hint_cost(usage_number);

    let mut tx = pool.begin().await?;

    // Step 3-4: pay via coins OR ad-view completion.
    let coin_balance = if req.paid_via_ad {
        // Ad-view path: no coin deduction. Server-side verification of ad
        // completion (Doc 8 anti-cheat: "verified_server_side" on
        // ad_views) happens before this endpoint is called — the ad_views
        // row itself is written by the ad-reward flow, not here.
        let bal: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?").bind(user_id).fetch_one(&mut *tx).await?;
        bal.0
    } else {
        let current: (i64,) = sqlx::query_as("SELECT coin_balance FROM users WHERE id = ?").bind(user_id).fetch_one(&mut *tx).await?;
        if current.0 < cost {
            return Err(GameError::InsufficientCoinsForHint);
        }
        // Doc 7 Sec7.3 step 4: "deduct via the same pattern as Shop
        // purchases ... reuse 'shop_purchase' — pick one convention and
        // document it." Chosen: reuse 'shop_purchase' (Doc 1's wallet_logs
        // CHECK constraint only allows the 8 types already defined, and
        // adding 'hint_purchase' would require an ALTER TABLE migration
        // just to widen a CHECK — not worth it for a sub-type). The
        // reference_id ("hint:{match_id}:{usage_number}") is what
        // distinguishes a hint purchase from a real shop purchase in
        // wallet_logs for analytics/support lookup.
        apply_ledger_entry_in_tx(&mut tx, LedgerEntryInput {
            user_id,
            log_type: "shop_purchase",
            amount: -cost,
            reference_id: Some(&format!("hint:{}:{}", req.match_id, usage_number)),
            ip_address: None,
            device_fingerprint: None,
        }).await?
    };

    // Step 5: run the chess engine against the CURRENT authoritative
    // board state, get its top suggested move.
    let move_suggested = suggest_move(registry, &req.match_id).await?;

    // Step 7: record match_hint_usage row.
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO match_hint_usage (id, match_id, user_id, usage_number, coins_spent, move_suggested, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.match_id)
    .bind(user_id)
    .bind(usage_number)
    .bind(if req.paid_via_ad { 0 } else { cost })
    .bind(&move_suggested)
    .bind(Utc::now().to_rfc3339())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HintResponse { move_suggested, coin_balance })
}

/// Doc 7 Sec7.3 step 5: "Backend runs a chess engine (e.g. a UCI-
/// compatible engine bundled server-side) against the CURRENT
/// authoritative board state, returns its top suggested move."
///
/// A real UCI engine (e.g. shelling out to a bundled Stockfish binary
/// over stdin/stdout, or the `pleco` crate) is NOT wired here — that
/// requires bundling an actual engine binary/dependency the doc doesn't
/// specify a version of, which would be guessing rather than following
/// spec. This returns shakmaty's first legal move as a structurally-
/// correct placeholder so the rest of the hint flow (payment, limits,
/// audit logging) is real and testable end-to-end; swap this function's
/// body for a real engine call before shipping hints to production.
async fn suggest_move(registry: &MatchRegistry, match_id: &str) -> Result<String, GameError> {
    let uci = registry.with_session(match_id, |session| {
        best_move_placeholder(&session.game)
    }).await.flatten();

    uci.ok_or(GameError::MatchNotFound)
}

fn best_move_placeholder(game: &GameState) -> Option<String> {
    let legal = game.position.legal_moves();
    let first = legal.first()?;
    let uci = shakmaty::uci::Uci::from_standard(first);
    Some(uci.to_string())
}
