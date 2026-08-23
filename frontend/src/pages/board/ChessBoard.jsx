import { useEffect, useRef, useState, useCallback } from 'react';
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

/**
 * Phase 1 — real match UI:
 * - Server is authority (shakmaty). Local chess.js only mirrors.
 * - Moves sent over WS; board updates from server fen / move.
 * - Turn lock on client for UX; server still rejects not_your_turn.
 * - Voice controls are disabled (not fake interactive).
 */
export default function ChessBoard({ user }) {
  const { matchId } = useParams();
  const location = useLocation();
  const navigate = useNavigate();
  const { accessToken } = useAuth();

  const chessRef = useRef(new Chess());
  const [fen, setFen] = useState(() => chessRef.current.fen());
  const [selectedSquare, setSelectedSquare] = useState(null);
  const [legalTargets, setLegalTargets] = useState([]);
  const [lastMove, setLastMove] = useState(null);
  const [moveHistoryOpen, setMoveHistoryOpen] = useState(false);
  const [historySan, setHistorySan] = useState([]);
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
  const [statusLine, setStatusLine] = useState('Connecting…');
  const [matchType, setMatchType] = useState('ranked');

  const socketRef = useRef(location.state?.socket || null);
  const countdownRef = useRef(null);
  const myColorRef = useRef(location.state?.color || 'white');

  const refreshHistory = useCallback(() => {
    setHistorySan([...chessRef.current.history()]);
  }, []);

  const applyServerFen = useCallback((nextFen, from, to) => {
    try {
      if (nextFen) {
        chessRef.current.load(nextFen);
      } else if (from && to) {
        try {
          chessRef.current.move({ from, to, promotion: 'q' });
        } catch {
          // ignore if already applied
        }
      }
      setFen(chessRef.current.fen());
      if (from && to) setLastMove({ from, to });
      refreshHistory();
    } catch (e) {
      console.error('fen apply failed', e);
    }
    setSelectedSquare(null);
    setLegalTargets([]);
    setHintMove(null);
  }, [refreshHistory]);

  // Load match metadata + PGN from REST (authoritative snapshot)
  useEffect(() => {
    if (!matchId || !accessToken) return;
    gameApi
      .getMatch(matchId)
      .then((d) => {
        if (d.my_color) {
          setColor(d.my_color);
          myColorRef.current = d.my_color;
        }
        if (d.opponent_username) setOpponentUsername(d.opponent_username);
        if (d.match_type) setMatchType(d.match_type);
        if (d.status === 'completed' || d.status === 'ended') {
          setMatchEnded({
            result: d.result,
            result_reason: d.result_reason,
          });
        }
        // Replay PGN into local mirror if present
        if (d.pgn && typeof d.pgn === 'string' && d.pgn.trim()) {
          try {
            chessRef.current.loadPgn(d.pgn);
            setFen(chessRef.current.fen());
            refreshHistory();
          } catch {
            // pgn may be UCI list not SAN — try as space-separated UCI
            const c = new Chess();
            const parts = d.pgn.trim().split(/\s+/);
            for (const u of parts) {
              if (u.length < 4) continue;
              try {
                c.move({
                  from: u.slice(0, 2),
                  to: u.slice(2, 4),
                  promotion: u.length > 4 ? u[4] : undefined,
                });
              } catch {
                break;
              }
            }
            chessRef.current = c;
            setFen(c.fen());
            refreshHistory();
          }
        }
      })
      .catch(() => {});
  }, [matchId, accessToken, refreshHistory]);

  // WebSocket
  useEffect(() => {
    if (!matchId || !accessToken) {
      setStatusLine('Waiting for login…');
      return undefined;
    }

    let cancelled = false;

    async function setup() {
      try {
        let socket = socketRef.current;
        if (!socket || !socket.ws || socket.ws.readyState > 1) {
          socket = new GameSocket(accessToken);
          socketRef.current = socket;
          await socket.connect(`/ws/match/${matchId}`);
          if (cancelled) return;
          socket.resumeMatch(matchId);
        }

        setStatusLine('Connected');

        socket.on('match_found', (msg) => {
          if (msg.color) {
            setColor(msg.color);
            myColorRef.current = msg.color;
          }
        });

        socket.on('board_update', (msg) => {
          applyServerFen(msg.fen, msg.from, msg.to);
          setStatusLine('Move applied');
        });

        socket.on('opponent_disconnected', () => {
          setOpponentDisconnected(true);
          setDisconnectCountdown(60);
          clearInterval(countdownRef.current);
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
          if (msg.sender_username === user?.username) return;
          setPlayingGift({
            name: msg.gift_name,
            icon_emoji: msg.icon_emoji,
            price_coins: msg.price_coins,
          });
        });

        socket.on('error', (msg) => {
          if (msg.code === 'illegal_move') {
            setToast({ message: 'Illegal move' });
            setSelectedSquare(null);
            setLegalTargets([]);
          } else if (msg.code === 'not_your_turn') {
            setToast({ message: 'Not your turn' });
            setSelectedSquare(null);
            setLegalTargets([]);
          } else if (msg.message) {
            setToast({ message: msg.message });
          }
        });

        socket.on('__closed', () => {
          if (!cancelled) setStatusLine('Disconnected — reconnect from Play if needed');
        });
      } catch (e) {
        if (!cancelled) {
          setStatusLine(e.message || 'Could not connect');
          setToast({ message: e.message || 'Board connection failed' });
        }
      }
    }

    setup();
    return () => {
      cancelled = true;
      clearInterval(countdownRef.current);
    };
  }, [matchId, accessToken, applyServerFen, user?.username]);

  function isMyTurn() {
    const turn = chessRef.current.turn(); // 'w' | 'b'
    const me = myColorRef.current || color;
    return (me === 'white' && turn === 'w') || (me === 'black' && turn === 'b');
  }

  function handleSquareTap(square) {
    if (matchEnded) return;

    if (selectedSquare) {
      if (legalTargets.includes(square)) {
        if (!isMyTurn()) {
          setToast({ message: 'Not your turn' });
          setSelectedSquare(null);
          setLegalTargets([]);
          return;
        }
        const piece = chessRef.current.get(selectedSquare);
        const isPromotion =
          piece?.type === 'p' && (square[1] === '8' || square[1] === '1');
        // Optimistic highlight only — board waits for server board_update
        setLastMove({ from: selectedSquare, to: square });
        socketRef.current?.move(
          matchId,
          selectedSquare,
          square,
          isPromotion ? 'q' : undefined
        );
        setStatusLine('Sending move…');
      }
      setSelectedSquare(null);
      setLegalTargets([]);
      return;
    }

    if (!isMyTurn()) {
      setToast({ message: 'Wait for opponent' });
      return;
    }

    const piece = chessRef.current.get(square);
    if (!piece) return;
    const me = myColorRef.current || color;
    const isMine = (me === 'white') === (piece.color === 'w');
    if (!isMine) return;

    const moves = chessRef.current.moves({ square, verbose: true });
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
    if (matchType === 'ranked') {
      setToast({ message: 'Hints only in casual matches' });
      return;
    }
    setHintLoading(true);
    try {
      const resp = await gameApi.requestHint(matchId, false);
      const from = resp.move_suggested.slice(0, 2);
      const to = resp.move_suggested.slice(2, 4);
      setHintMove({ from, to });
      setToast({ message: `Hint — ${resp.coin_balance} coins left` });
    } catch (e) {
      setToast({ message: e.message || 'Hint failed' });
    } finally {
      setHintLoading(false);
    }
  }

  async function handleSendGift(item) {
    try {
      if (!opponentUsername) {
        setToast({ message: 'Opponent unknown' });
        return;
      }
      await giftsApi.send(opponentUsername, item.id, 'in_match', matchId);
      setShowGiftPicker(false);
      setPlayingGift(item);
    } catch (err) {
      setToast({ message: err.message });
    }
  }

  const turnLabel = matchEnded
    ? 'Match over'
    : isMyTurn()
      ? 'Your turn'
      : "Opponent's turn";

  if (matchEnded) {
    const won =
      matchEnded.result === 'draw'
        ? null
        : matchEnded.result?.includes(color) ||
          matchEnded.result === color ||
          matchEnded.result?.startsWith(color);
    return (
      <div className="ck-board-screen ck-board-screen--ended">
        <h1 className="page-title">
          {matchEnded.result === 'draw' || matchEnded.result === '1/2-1/2'
            ? 'Draw'
            : won
              ? 'You Won!'
              : 'You Lost'}
        </h1>
        <p className="text-secondary">{matchEnded.result_reason || matchEnded.result}</p>
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
          Opponent disconnected — {disconnectCountdown}s
        </div>
      )}

      <div className="ck-board__info-bar">
        <span className="ck-board__info-avatar" aria-hidden>
          👤
        </span>
        <div style={{ flex: 1 }}>
          <strong>{opponentUsername || 'Opponent'}</strong>
          <div className="text-secondary" style={{ fontSize: 12 }}>
            {turnLabel} · You are {color}
          </div>
        </div>
        <span className="text-secondary" style={{ fontSize: 11 }} title={statusLine}>
          {statusLine}
        </span>
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

      <div className="ck-board__toolbar">
        <button
          type="button"
          className="icon-tap-target"
          aria-label="Voice chat not available"
          disabled
          title="Voice chat — Phase later (WebRTC). Not available yet."
        >
          🎙️
        </button>
        <button type="button" className="ck-board__hint" onClick={handleHint} disabled={hintLoading}>
          {hintLoading ? '…' : 'Hint'}
        </button>
        <button type="button" onClick={() => setShowGiftPicker(true)}>
          Gift
        </button>
        <button type="button" className="ck-board__resign" onClick={handleResign}>
          Resign
        </button>
      </div>

      <button
        type="button"
        className="ck-board__history-toggle"
        onClick={() => setMoveHistoryOpen((o) => !o)}
      >
        {moveHistoryOpen ? '▾ Hide moves' : '▴ Show moves'}
      </button>
      {moveHistoryOpen && (
        <div className="ck-board__history">
          {historySan.length === 0 && <span className="text-secondary">No moves yet</span>}
          {historySan.map((move, i) => (
            <span key={i} className="ck-board__history-move">
              {move}
            </span>
          ))}
        </div>
      )}

      {showGiftPicker && (
        <GiftPicker
          recipientLabel={opponentUsername || 'Opponent'}
          onSend={handleSendGift}
          onClose={() => setShowGiftPicker(false)}
        />
      )}
      {playingGift && (
        <GiftAnimation item={playingGift} onComplete={() => setPlayingGift(null)} />
      )}
      {toast && (
        <Toast message={toast.message} visible={!!toast} onDismiss={() => setToast(null)} />
      )}
    </div>
  );
}
