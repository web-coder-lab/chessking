use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

/// Doc 7 Sec2 step 3: "in-memory (or Redis-backed, if scaling later)
/// queue bucket." This is the in-memory version — swapping to Redis
/// later means replacing this struct's internals without touching the
/// WebSocket handler's calling code.
#[derive(Clone)]
pub struct MatchmakingQueue {
    ranked: Arc<Mutex<VecDeque<QueuedPlayer>>>,
    casual: Arc<Mutex<VecDeque<QueuedPlayer>>>,
}

pub struct QueuedPlayer {
    pub user_id: String,
    pub rating: i64,
    pub joined_at: Instant,
    /// Fires once this player is paired — carries (match_id, opponent_id,
    /// is_initiator). Exactly one side of every pair gets is_initiator =
    /// true, so exactly one side is responsible for creating the DB row
    /// and MatchRegistry entry — the other side just subscribes to it.
    pub notify: mpsc::Sender<(String, String, bool)>,
}

const INITIAL_BAND: i64 = 150;
const BAND_WIDEN_PER_SECOND: i64 = 10; // widens the longer they wait

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self {
            ranked: Arc::new(Mutex::new(VecDeque::new())),
            casual: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Doc 7 Sec2 step 3b/3c + step 4: attempts an immediate match against
    /// whoever is already waiting; if none fit, joins the queue and waits
    /// to be matched by a LATER joiner's call to this same function (or
    /// by the periodic sweep in `run_periodic_matching` below, which
    /// re-checks widening bands for players who've been waiting a while).
    pub async fn join(&self, match_type: &str, player: QueuedPlayer) {
        let queue = if match_type == "ranked" { &self.ranked } else { &self.casual };
        let mut guard = queue.lock().await;

        let match_index = if match_type == "ranked" {
            guard.iter().position(|other| {
                let band = current_band(other.joined_at);
                (other.rating - player.rating).abs() <= band
            })
        } else {
            // Doc 7 Sec2 step 3c: casual matches on availability alone.
            if guard.is_empty() { None } else { Some(0) }
        };

        if let Some(idx) = match_index {
            let opponent = guard.remove(idx).unwrap();
            drop(guard);
            pair_players(player, opponent).await;
        } else {
            guard.push_back(player);
        }
    }

    /// Doc 7 Sec2 step 3b: "widening the longer they wait, to avoid
    /// indefinite queue times for players at rating extremes." Called on
    /// a timer so two players who are both ALREADY waiting (neither one
    /// triggers the other via `join`) still eventually get matched once
    /// their bands overlap.
    pub async fn sweep_ranked(&self) {
        let mut guard = self.ranked.lock().await;
        let mut i = 0;
        while i < guard.len() {
            let band_i = current_band(guard[i].joined_at);
            let mut matched = None;
            for j in (i + 1)..guard.len() {
                if (guard[i].rating - guard[j].rating).abs() <= band_i {
                    matched = Some(j);
                    break;
                }
            }
            if let Some(j) = matched {
                let p2 = guard.remove(j).unwrap();
                let p1 = guard.remove(i).unwrap();
                drop(guard);
                pair_players(p1, p2).await;
                guard = self.ranked.lock().await;
                // don't advance i — re-check the new element now at index i
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

/// Doc 7 Sec2 step 4: creates the matches row, randomly assigns white/
/// black, notifies both clients. DB row creation happens in the
/// initiator's WS handler (see websocket.rs) — this function only
/// decides pairing + who initiates, keeping this module DB-free.
async fn pair_players(a: QueuedPlayer, b: QueuedPlayer) {
    let match_id = uuid::Uuid::new_v4().to_string();

    let _ = a.notify.send((match_id.clone(), b.user_id.clone(), true)).await;
    let _ = b.notify.send((match_id, a.user_id.clone(), false)).await;
}

/// Spawns the periodic ranked-queue sweep (band widening for players who
/// are BOTH already waiting). A 2-second interval balances responsiveness
/// against CPU cost for a queue that's checked on every join anyway.
pub fn spawn_periodic_matching(queue: MatchmakingQueue) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            queue.sweep_ranked().await;
        }
    });
}
