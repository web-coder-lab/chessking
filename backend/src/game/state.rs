use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use super::disconnect::DisconnectTracker;
use super::engine::GameState;

/// Doc 7 §3.1 step a: "an in-memory game-state cache keyed by match_id."
/// This is the live authoritative board for every in-progress match. On
/// server restart, a match's state is rebuilt from `matches.pgn` via
/// `GameState::from_move_history` instead of being lost.
#[derive(Clone)]
pub struct MatchRegistry {
    inner: Arc<Mutex<HashMap<String, MatchSession>>>,
}

pub struct MatchSession {
    pub game: GameState,
    pub white_id: String,
    pub black_id: String,
    pub match_type: String, // "ranked" | "casual" | "custom"
    /// Broadcasts board updates / move events / disconnect banners to
    /// both connected clients (Doc 7 §3.1 step 4, §4.1 step 4).
    pub events: broadcast::Sender<String>,
    pub disconnect: DisconnectTracker,
}

impl MatchRegistry {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
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
