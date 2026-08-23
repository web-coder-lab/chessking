use shakmaty::fen::Fen;
use shakmaty::uci::Uci;
use shakmaty::{Chess, Color, Position};

use super::errors::GameError;

/// Doc 7 Sec3: "the server is the sole authority on game state." This
/// struct IS that authority - the client only ever sees the results of
/// calling methods on this, never decides legality itself.
///
/// move_history stores UCI strings (e.g. "e2e4", "e7e8q") in order -
/// this is what gets persisted to matches.pgn and is also what
/// reconstructs the position on server restart (Sec3.1 step a).
pub struct GameState {
    pub position: Chess,
    pub move_history: Vec<String>,
    /// Position-only FEN keys (board + side-to-move + castling +
    /// en-passant, NOT halfmove/fullmove counters) seen so far, for
    /// threefold-repetition detection.
    repetition_log: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameOutcome {
    WhiteWins,
    BlackWins,
    Draw,
}

impl GameState {
    pub fn new() -> Self {
        let position = Chess::default();
        let mut state = Self { position, move_history: Vec::new(), repetition_log: Vec::new() };
        state.log_repetition_key();
        state
    }

    /// Doc 7 Sec3.1 step a: reconstructs authoritative state by replaying
    /// the stored move list - used when a match is loaded from the DB
    /// (e.g. after a server restart) rather than served from the
    /// in-memory cache.
    pub fn from_move_history(moves: &[String]) -> Result<Self, GameError> {
        let mut state = Self::new();
        for uci_str in moves {
            state.apply_uci_str(uci_str)?;
        }
        Ok(state)
    }

    fn log_repetition_key(&mut self) {
        let fen = Fen::from_position(self.position.clone(), shakmaty::EnPassantMode::Legal);
        let full = fen.to_string();
        // Strip the halfmove/fullmove counter fields - repetition counts
        // only board+turn+castling+ep recurring, not the move clocks.
        let key: String = full.split(' ').take(4).collect::<Vec<_>>().join(" ");
        self.repetition_log.push(key);
    }

    fn apply_uci_str(&mut self, uci_str: &str) -> Result<(), GameError> {
        let uci: Uci = uci_str.parse().map_err(|_| GameError::IllegalMove)?;
        let mv = uci.to_move(&self.position).map_err(|_| GameError::IllegalMove)?;
        self.position = self.position.clone().play(&mv).map_err(|_| GameError::IllegalMove)?;
        self.move_history.push(uci_str.to_string());
        self.log_repetition_key();
        Ok(())
    }

    /// Doc 7 Sec3.1 steps b-c: validates and applies a move using
    /// shakmaty's full chess-rules implementation (legality, checks,
    /// pins, castling rights, en passant, promotion) - never hand-rolled
    /// logic. Returns Err(IllegalMove) without mutating state at all if
    /// illegal, exactly per Sec3.1 step c: "board state does NOT change."
    pub fn try_apply_move(&mut self, from: &str, to: &str, promotion: Option<&str>) -> Result<(), GameError> {
        let uci_str = match promotion {
            Some(p) => format!("{from}{to}{p}"),
            None => format!("{from}{to}"),
        };

        let uci: Uci = uci_str.parse().map_err(|_| GameError::IllegalMove)?;
        let mv = uci.to_move(&self.position).map_err(|_| GameError::IllegalMove)?;
        let new_position = self.position.clone().play(&mv).map_err(|_| GameError::IllegalMove)?;

        self.position = new_position;
        self.move_history.push(uci_str);
        self.log_repetition_key();
        Ok(())
    }

    /// Doc 7 Sec3.1 step d: "check for game-ending conditions (checkmate/
    /// stalemate/draw by repetition/50-move rule/insufficient material)."
    pub fn check_game_end(&self) -> Option<GameOutcome> {
        if self.position.is_checkmate() {
            return Some(match self.position.turn() {
                Color::White => GameOutcome::BlackWins,
                Color::Black => GameOutcome::WhiteWins,
            });
        }
        if self.position.is_stalemate() || self.position.is_insufficient_material() {
            return Some(GameOutcome::Draw);
        }
        if self.position.halfmoves() >= 100 {
            return Some(GameOutcome::Draw);
        }
        if let Some(last_key) = self.repetition_log.last() {
            let occurrences = self.repetition_log.iter().filter(|k| *k == last_key).count();
            if occurrences >= 3 {
                return Some(GameOutcome::Draw);
            }
        }
        None
    }

    pub fn fen(&self) -> String {
        Fen::from_position(self.position.clone(), shakmaty::EnPassantMode::Legal).to_string()
    }

    pub fn side_to_move(&self) -> Color {
        self.position.turn()
    }

    pub fn to_pgn(&self) -> String {
        self.move_history.join(" ")
    }
}

/// Doc 7 Sec3.1 step 5b: standard Elo formula, ranked matches only.
/// K-factor = 32 (standard default; not specified numerically in the
/// doc).
pub fn calculate_elo(rating_a: i64, rating_b: i64, outcome: GameOutcome, is_a: bool) -> i64 {
    const K: f64 = 32.0;
    let score_a: f64 = match outcome {
        GameOutcome::WhiteWins if is_a => 1.0,
        GameOutcome::WhiteWins => 0.0,
        GameOutcome::BlackWins if is_a => 0.0,
        GameOutcome::BlackWins => 1.0,
        GameOutcome::Draw => 0.5,
    };

    let expected_a = 1.0 / (1.0 + 10f64.powf((rating_b as f64 - rating_a as f64) / 400.0));
    let new_rating = rating_a as f64 + K * (score_a - expected_a);
    new_rating.round() as i64
}
