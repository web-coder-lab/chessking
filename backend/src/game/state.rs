use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, Mutex};

use super::disconnect::DisconnectTracker;
use super::engine::GameState;

/// Default time control: 10 minutes per side, no increment (Phase 5).
pub const DEFAULT_CLOCK_MS: i64 = 10 * 60 * 1000;

#[derive(Clone)]
pub struct MatchRegistry {
    inner: Arc<Mutex<HashMap<String, MatchSession>>>,
}

pub struct MatchSession {
    pub game: GameState,
    pub white_id: String,
    pub black_id: String,
    pub match_type: String,
    pub events: broadcast::Sender<String>,
    pub disconnect: DisconnectTracker,
    /// Remaining time (ms). Server-authoritative.
    pub white_ms: i64,
    pub black_ms: i64,
    /// When the current side-to-move's clock started ticking.
    pub turn_started_at: Instant,
    /// User id of the side that offered a draw (waiting for reply).
    pub pending_draw_from: Option<String>,
}

impl MatchSession {
    /// Apply elapsed time to the side that was to move. Returns true if that side flagged.
    pub fn tick_clock_for_side(&mut self, white_to_move: bool) -> bool {
        let elapsed = self.turn_started_at.elapsed().as_millis() as i64;
        if white_to_move {
            self.white_ms = (self.white_ms - elapsed).max(0);
            self.white_ms == 0
        } else {
            self.black_ms = (self.black_ms - elapsed).max(0);
            self.black_ms == 0
        }
    }

    pub fn start_opponent_clock(&mut self) {
        self.turn_started_at = Instant::now();
    }
}

impl MatchRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, match_id: String, session: MatchSession) {
        self.inner.lock().await.insert(match_id, session);
    }

    pub async fn remove(&self, match_id: &str) {
        self.inner.lock().await.remove(match_id);
    }

    pub async fn with_session<F, R>(&self, match_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut MatchSession) -> R,
    {
        let mut guard = self.inner.lock().await;
        guard.get_mut(match_id).map(f)
    }

    pub async fn subscribe(&self, match_id: &str) -> Option<broadcast::Receiver<String>> {
        let guard = self.inner.lock().await;
        guard.get(match_id).map(|s| s.events.subscribe())
    }
}
