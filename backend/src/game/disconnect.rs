use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::engine::GameOutcome;
use super::state::MatchRegistry;

const GRACE_PERIOD_SECS: u64 = 60;

/// Doc 7 Sec4: per-match disconnect bookkeeping. Tracks which side(s) are
/// currently down, so the "first-returner-waits" rule (Sec4.2) can be
/// applied correctly rather than naively starting a timer per-disconnect.
#[derive(Clone)]
pub struct DisconnectTracker {
    state: Arc<Mutex<TrackerState>>,
}

#[derive(Default)]
struct TrackerState {
    white_connected: bool,
    black_connected: bool,
    /// Generation counter — bumped every time a timer-relevant event
    /// happens (disconnect/reconnect), so an in-flight timer task can
    /// tell if it's now stale and should no-op instead of wrongly ending
    /// the match.
    generation: u64,
}

impl DisconnectTracker {
    pub fn new() -> Self {
        Self { state: Arc::new(Mutex::new(TrackerState { white_connected: true, black_connected: true, generation: 0 })) }
    }

    /// Doc 7 Sec4.1 steps 1-3 / Sec4.2 steps 1-3: called when a side's
    /// WebSocket closes unexpectedly. Records the disconnect, starts the
    /// appropriate grace-period timer per the exact branching the doc
    /// specifies.
    pub async fn on_disconnect(
        &self,
        pool: SqlitePool,
        registry: MatchRegistry,
        match_id: String,
        white_id: String,
        black_id: String,
        is_white: bool,
        match_type: String,
    ) {
        let my_generation;
        let other_already_down;
        {
            let mut s = self.state.lock().await;
            if is_white { s.white_connected = false; } else { s.black_connected = false; }
            other_already_down = if is_white { !s.black_connected } else { !s.white_connected };
            s.generation += 1;
            my_generation = s.generation;
        }

        let _ = record_disconnect_event(&pool, &match_id, if is_white { &white_id } else { &black_id }).await;

        if other_already_down {
            // Sec4.2 step 3: "Whoever reconnects FIRST does not
            // immediately win" — this disconnect is the SECOND one (the
            // other side was already down), so per the doc's exact rule,
            // the timer for the currently-still-missing side started when
            // THEY first went down, not now. Nothing new to start here;
            // on_reconnect handles starting the fresh 60s window once
            // someone comes back first.
            return;
        }

        // Sec4.1 step 3: single-disconnect grace period.
        let tracker = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(GRACE_PERIOD_SECS)).await;
            tracker.expire_if_still_current(pool, registry, match_id, white_id, black_id, is_white, match_type, my_generation).await;
        });
    }

    /// Doc 7 Sec4.1 step 5a / Sec4.2 step 3-4: called when a side's
    /// WebSocket reconnects. Bumps the generation so any in-flight timer
    /// for THIS side's earlier disconnect becomes stale and no-ops.
    /// Sec4.2's "first-returner-waits, fresh timer for the other" rule is
    /// implemented by starting a NEW timer here when the OTHER side is
    /// still down.
    pub async fn on_reconnect(
        &self,
        pool: SqlitePool,
        registry: MatchRegistry,
        match_id: String,
        white_id: String,
        black_id: String,
        is_white: bool,
        match_type: String,
    ) {
        let other_still_down;
        let my_generation;
        {
            let mut s = self.state.lock().await;
            if is_white { s.white_connected = true; } else { s.black_connected = true; }
            other_still_down = if is_white { !s.black_connected } else { !s.white_connected };
            s.generation += 1;
            my_generation = s.generation;
        }

        record_reconnect_event(&pool, &match_id, if is_white { &white_id } else { &black_id }).await.ok();

        if other_still_down {
            // Sec4.2 step 3: fresh 60s timer for the OTHER (still-missing)
            // player starts NOW, from this reconnect moment.
            let tracker = self.clone();
            let is_white_of_missing = !is_white; // the timer now tracks the OTHER side
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(GRACE_PERIOD_SECS)).await;
                tracker.expire_if_still_current(pool, registry, match_id, white_id, black_id, is_white_of_missing, match_type, my_generation).await;
            });
        }
        // If neither side is down anymore, game simply resumes — Sec4.1
        // step 5a / Sec4.2 step 4a: "board state unchanged."
    }

    async fn expire_if_still_current(
        &self,
        pool: SqlitePool,
        registry: MatchRegistry,
        match_id: String,
        white_id: String,
        black_id: String,
        missing_is_white: bool,
        match_type: String,
        generation_at_start: u64,
    ) {
        let still_current = {
            let s = self.state.lock().await;
            s.generation == generation_at_start
        };
        if !still_current {
            return; // the missing side reconnected before the timer fired
        }

        // Sec4.1 step 5b / Sec4.2 step 4b: still-connected player wins.
        let winner_is_white = !missing_is_white;
        let outcome = if winner_is_white { GameOutcome::WhiteWins } else { GameOutcome::BlackWins };

        super::finalize::finalize_match(&pool, &registry, &match_id, &white_id, &black_id, outcome, "disconnect_timeout", &match_type).await.ok();
    }
}

async fn record_disconnect_event(pool: &SqlitePool, match_id: &str, user_id: &str) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO match_disconnect_events (id, match_id, user_id, disconnected_at, grace_period_expired)
         VALUES (?, ?, ?, ?, 0)"
    )
    .bind(&id)
    .bind(match_id)
    .bind(user_id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_reconnect_event(pool: &SqlitePool, match_id: &str, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE match_disconnect_events SET reconnected_at = ?
         WHERE match_id = ? AND user_id = ? AND reconnected_at IS NULL"
    )
    .bind(Utc::now().to_rfc3339())
    .bind(match_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}
