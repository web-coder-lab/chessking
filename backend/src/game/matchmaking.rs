use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

/// In-memory matchmaking queues (ranked + casual).
#[derive(Clone)]
pub struct MatchmakingQueue {
    ranked: Arc<Mutex<VecDeque<QueuedPlayer>>>,
    casual: Arc<Mutex<VecDeque<QueuedPlayer>>>,
}

pub struct QueuedPlayer {
    pub user_id: String,
    pub rating: i64,
    pub joined_at: Instant,
    /// (match_id, opponent_id, is_initiator)
    pub notify: mpsc::Sender<(String, String, bool)>,
}

const INITIAL_BAND: i64 = 200;
const BAND_WIDEN_PER_SECOND: i64 = 25;

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self {
            ranked: Arc::new(Mutex::new(VecDeque::new())),
            casual: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn join(&self, match_type: &str, player: QueuedPlayer) {
        let queue = if match_type == "ranked" {
            &self.ranked
        } else {
            &self.casual
        };
        let mut guard = queue.lock().await;

        // Drop zombies (socket gone)
        guard.retain(|other| !other.notify.is_closed());
        // Drop duplicate self
        guard.retain(|other| other.user_id != player.user_id);

        let match_index = if match_type == "ranked" {
            guard.iter().position(|other| {
                let band = current_band(other.joined_at).max(current_band(player.joined_at));
                (other.rating - player.rating).abs() <= band
            })
        } else {
            // Casual: any other waiting player
            guard.iter().position(|_| true)
        };

        if let Some(idx) = match_index {
            let opponent = guard.remove(idx).unwrap();
            drop(guard);
            pair_players(player, opponent).await;
        } else {
            guard.push_back(player);
        }
    }

    pub async fn leave(&self, user_id: &str) {
        for q in [&self.ranked, &self.casual] {
            let mut guard = q.lock().await;
            guard.retain(|p| p.user_id != user_id);
        }
    }

    /// Ranked: widen bands for players already waiting.
    pub async fn sweep_ranked(&self) {
        self.sweep_queue(&self.ranked, true).await;
    }

    /// Casual: pair anyone still waiting (covers rare race where both joined empty).
    pub async fn sweep_casual(&self) {
        self.sweep_queue(&self.casual, false).await;
    }

    async fn sweep_queue(&self, queue: &Arc<Mutex<VecDeque<QueuedPlayer>>>, ranked: bool) {
        let mut guard = queue.lock().await;
        guard.retain(|p| !p.notify.is_closed());

        let mut i = 0;
        while i < guard.len() {
            let mut matched = None;
            for j in (i + 1)..guard.len() {
                if guard[i].user_id == guard[j].user_id {
                    continue;
                }
                if ranked {
                    let band = current_band(guard[i].joined_at).max(current_band(guard[j].joined_at));
                    if (guard[i].rating - guard[j].rating).abs() <= band {
                        matched = Some(j);
                        break;
                    }
                } else {
                    matched = Some(j);
                    break;
                }
            }
            if let Some(j) = matched {
                let p2 = guard.remove(j).unwrap();
                let p1 = guard.remove(i).unwrap();
                drop(guard);
                pair_players(p1, p2).await;
                guard = queue.lock().await;
            } else {
                i += 1;
            }
        }
    }
}

fn current_band(joined_at: Instant) -> i64 {
    let waited_secs = joined_at.elapsed().as_secs() as i64;
    INITIAL_BAND + waited_secs * BAND_WIDEN_PER_SECOND
}

async fn pair_players(a: QueuedPlayer, b: QueuedPlayer) {
    let match_id = uuid::Uuid::new_v4().to_string();
    // Prefer notifying both; if one channel is dead, other still plays? skip pair if either fails
    let r1 = a.notify.send((match_id.clone(), b.user_id.clone(), true)).await;
    let r2 = b.notify.send((match_id, a.user_id.clone(), false)).await;
    if r1.is_err() || r2.is_err() {
        tracing::warn!("matchmaking pair notify failed (one side gone)");
    }
}

pub fn spawn_periodic_matching(queue: MatchmakingQueue) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            queue.sweep_ranked().await;
            queue.sweep_casual().await;
        }
    });
}
