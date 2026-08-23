use std::collections::{HashMap, VecDeque};
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
    /// Recent think times (ms) per side — for anti-cheat timing (Phase 9).
    pub white_move_times_ms: VecDeque<i64>,
    pub black_move_times_ms: VecDeque<i64>,
    pub ply_count: u32,
}

/// Min human-like think time after opening (ms). Below this repeatedly → anomaly.
pub const FAST_MOVE_MS: i64 = 80;
pub const FAST_MOVE_STREAK_FOR_ANOMALY: usize = 4;
pub const TIMING_SAMPLE_CAP: usize = 12;

impl MatchSession {
    /// Apply elapsed time to the side that was to move. Returns (flagged, think_ms).
    pub fn tick_clock_for_side(&mut self, white_to_move: bool) -> (bool, i64) {
        let elapsed = self.turn_started_at.elapsed().as_millis() as i64;
        if white_to_move {
            self.white_ms = (self.white_ms - elapsed).max(0);
            (self.white_ms == 0, elapsed)
        } else {
            self.black_ms = (self.black_ms - elapsed).max(0);
            (self.black_ms == 0, elapsed)
        }
    }

    pub fn start_opponent_clock(&mut self) {
        self.turn_started_at = Instant::now();
    }

    /// Record think time; returns true if timing looks bot-like (streak of ultra-fast moves).
    pub fn record_think_time(&mut self, white_moved: bool, think_ms: i64) -> bool {
        self.ply_count = self.ply_count.saturating_add(1);
        // Opening book can be fast — only score after a few plies
        if self.ply_count <= 4 {
            return false;
        }
        let q = if white_moved {
            &mut self.white_move_times_ms
        } else {
            &mut self.black_move_times_ms
        };
        q.push_back(think_ms);
        while q.len() > TIMING_SAMPLE_CAP {
            q.pop_front();
        }
        if q.len() < FAST_MOVE_STREAK_FOR_ANOMALY {
            return false;
        }
        let streak = q
            .iter()
            .rev()
            .take(FAST_MOVE_STREAK_FOR_ANOMALY)
            .filter(|&&ms| ms < FAST_MOVE_MS)
            .count();
        streak >= FAST_MOVE_STREAK_FOR_ANOMALY
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
