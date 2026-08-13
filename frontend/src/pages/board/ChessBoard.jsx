import { useEffect, useRef, useState } from 'react';
import { useParams, useLocation, useNavigate } from 'react-router-dom';
import { Chess } from 'chess.js';
import Board from './Board';
import Button from '../../components/common/Button';
import { useAuth } from '../../context/AuthContext';
import { GameSocket } from '../../services/gameSocket';
import { gameApi, giftsApi } from '../../services/api';
import GiftPicker from '../../components/gifts/GiftPicker';
import GiftAnimation from '../../components/gifts/GiftAnimation';
import Toast from '../../components/common/Toast';
import './ChessBoard.css';

export default function ChessBoard({ user }) {
  const { matchId } = useParams();
  const location = useLocation();
  const navigate = useNavigate();
  const { accessToken } = useAuth();

  const [chess] = useState(() => new Chess());
  const [fen, setFen] = useState(chess.fen());
  const [selectedSquare, setSelectedSquare] = useState(null);
  const [legalTargets, setLegalTargets] = useState([]);
  const [lastMove, setLastMove] = useState(null);
  const [moveHistoryOpen, setMoveHistoryOpen] = useState(false);
  const [opponentDisconnected, setOpponentDisconnected] = useState(false);
  const [disconnectCountdown, setDisconnectCountdown] = useState(60);
  const [matchEnded, setMatchEnded] = useState(null);
  const [color, setColor] = useState(location.state?.color || 'white');
  const [hintMove, setHintMove] = useState(null);
  const [hintLoading, setHintLoading] = useState(false);
  const [toast, setToast] = useState(null);
  const [opponentUsername, setOpponentUsername] = useState(null);
  const [showGiftPicker, setShowGiftPicker] = useState(false);
  const [playingGift, setPlayingGift] = useState(null);

  const socketRef = useRef(location.state?.socket || null);
  const countdownRef = useRef(null);

  useEffect(() => {
    gameApi.getMatch(matchId).then((d) => setOpponentUsername(d.opponent_username)).catch(() => {});
  }, [matchId]);

  async function handleSendGift(item) {
    try {
      await giftsApi.send(opponentUsername, item.id, 'in_match', matchId);
      setShowGiftPicker(false);
      setPlayingGift(item);
    } catch (err) {
      setToast({ message: err.message });
    }
  }

  useEffect(() => {
    async function setup() {
      let socket = socketRef.current;
      if (!socket) {
        socket = new GameSocket(accessToken);
        socketRef.current = socket;
        await socket.connect(`/ws/match/${matchId}`);
        socket.resumeMatch(matchId);
      }

      socket.on('match_found', (msg) => setColor(msg.color));

      socket.on('board_update', (msg) => {
        try {
          if (msg.promotion) chess.move({ from: msg.from, to: msg.to, promotion: msg.promotion });
          else chess.move({ from: msg.from, to: msg.to });
          setFen(chess.fen());
          setLastMove({ from: msg.from, to: msg.to });
        } catch {
          // Server is authoritative — if local replay ever disagrees,
          // trust the server's next message rather than showing a
          // possibly-wrong board.
        }
        setSelectedSquare(null);
        setLegalTargets([]);
        setHintMove(null);
      });

      socket.on('opponent_disconnected', () => {
        setOpponentDisconnected(true);
        setDisconnectCountdown(60);
        countdownRef.current = setInterval(() => {
          setDisconnectCountdown((c) => (c > 0 ? c - 1 : 0));
        }, 1000);
      });

      socket.on('opponent_reconnected', () => {
        setOpponentDisconnected(false);
        clearInterval(countdownRef.current);
      });

      socket.on('match_ended', (msg) => {
        setMatchEnded(msg);
        clearInterval(countdownRef.current);
      });

      socket.on('gift_sent', (msg) => {
        if (msg.sender_username === user?.username) return; // already played locally on send
        setPlayingGift({ name: msg.gift_name, icon_emoji: msg.icon_emoji, price_coins: msg.price_coins });
      });

      socket.on('error', (msg) => {
        if (msg.code === 'illegal_move') {
          setSelectedSquare(null);
          setLegalTargets([]);
        }
      });
    }

    setup();
    return () => clearInterval(countdownRef.current);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function handleSquareTap(square) {
    if (matchEnded) return;

    if (selectedSquare) {
      if (legalTargets.includes(square)) {
        const piece = chess.get(selectedSquare);
        const isPromotion = piece?.type === 'p' && (square[1] === '8' || square[1] === '1');
        socketRef.current?.move(matchId, selectedSquare, square, isPromotion ? 'q' : undefined);
      }
      setSelectedSquare(null);
      setLegalTargets([]);
      return;
    }

    const piece = chess.get(square);
    if (!piece) return;
    const isMine = (color === 'white') === (piece.color === 'w');
    if (!isMine) return;

    const moves = chess.moves({ square, verbose: true });
    setSelectedSquare(square);
    setLegalTargets(moves.map((m) => m.to));
  }

  function handleResign() {
    if (window.confirm('Resign this match?')) {
      socketRef.current?.resign(matchId);
    }
  }

  async function handleHint() {
    if (hintLoading) return;
    setHintLoading(true);
    try {
      const resp = await gameApi.requestHint(matchId, false);
      // UCI format e.g. "e2e4" or "e7e8q" (promotion) — first 2 chars
      // are the origin square, next 2 are the destination.
      const from = resp.move_suggested.slice(0, 2);
      const to = resp.move_suggested.slice(2, 4);
      setHintMove({ from, to });
      setToast({ message: `Hint used — ${resp.coin_balance} coins left` });
    } catch (e) {
      if (e.code === 'insufficient_coins') {
        setToast({ message: 'Not enough coins for a hint', actionLabel: 'Add Coins', action: () => navigate('/wallet') });
      } else if (e.code === 'hint_limit_reached') {
        setToast({ message: 'No more hints this match' });
      } else {
        setToast({ message: e.message || 'Could not get a hint' });
      }
    } finally {
      setHintLoading(false);
    }
  }

  if (matchEnded) {
    return (
      <div className="ck-board-screen ck-board-screen--ended">
        <h1 className="page-title">
          {matchEnded.result === 'draw' ? 'Draw' : matchEnded.result?.startsWith(color) ? 'You Won!' : 'You Lost'}
        </h1>
        <p className="text-secondary">{matchEnded.result_reason}</p>
        <Button onClick={() => navigate('/dashboard')} style={{ marginTop: 'var(--space-6)' }}>
          Back to Dashboard
        </Button>
      </div>
    );
  }

  return (
    <div className="ck-board-screen">
      {opponentDisconnected && (
        <div className="ck-board__disconnect-banner">
          Opponent disconnected — reconnecting... {disconnectCountdown}s
        </div>
      )}

      <div className="ck-board__info-bar">
        <img src="/assets/default-avatar.svg" alt="" className="ck-board__info-avatar" />
        <div className="ck-board__info-text">
          <span className="ck-board__info-username">{opponentUsername || 'Opponent'}</span>
        </div>
        <button className="icon-tap-target" aria-label="Opponent muted" disabled title="Voice chat isn't available yet">🔇</button>
        <button className="icon-tap-target" aria-label="Send gift" onClick={() => setShowGiftPicker(true)} disabled={!opponentUsername}>🎁</button>
      </div>

      <div className="ck-board__actions">
        <button className="ck-board__action-btn" onClick={handleResign}>Resign</button>
        <button className="ck-board__action-btn" onClick={() => socketRef.current?.send({ type: 'offer_draw', match_id: matchId })}>
          Offer Draw
        </button>
        <button className="ck-board__action-btn" onClick={handleHint} disabled={hintLoading}>
          {hintLoading ? '...' : 'Hint'}
        </button>
      </div>

      <Board
        fen={fen}
        orientation={color}
        onMove={handleSquareTap}
        selectedSquare={selectedSquare}
        legalTargets={legalTargets}
        lastMove={lastMove}
        hintMove={hintMove}
      />

      <button className="ck-board__history-strip" onClick={() => setMoveHistoryOpen((o) => !o)}>
        {moveHistoryOpen ? '▾ Hide moves' : '▴ Show moves'}
      </button>
      {moveHistoryOpen && (
        <div className="ck-board__history-list">
          {chess.history().map((move, i) => (
            <span key={i} className="ck-board__history-move">{move}</span>
          ))}
        </div>
      )}

      <div className="ck-board__info-bar">
        <img src={user?.avatarUrl} alt="" className="ck-board__info-avatar" />
        <div className="ck-board__info-text">
          <span className="ck-board__info-username">{user?.username}</span>
        </div>
        <button className="icon-tap-target" aria-label="Mute microphone" disabled title="Voice chat isn't available yet">🎙️</button>
      </div>

      <Toast
        visible={!!toast}
        message={toast?.message}
        actionLabel={toast?.actionLabel}
        onAction={toast?.action}
        onDismiss={() => setToast(null)}
      />

      {showGiftPicker && (
        <GiftPicker
          recipientLabel={opponentUsername}
          onSend={handleSendGift}
          onClose={() => setShowGiftPicker(false)}
        />
      )}

      {playingGift && (
        <GiftAnimation item={playingGift} onComplete={() => setPlayingGift(null)} />
      )}
    </div>
  );
}
