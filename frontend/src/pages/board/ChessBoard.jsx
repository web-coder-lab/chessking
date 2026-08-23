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
import { soundMove, soundCapture, soundCheck, soundGameEnd, soundDrawOffer } from '../../utils/gameSounds';
import { VoiceChannel } from '../../utils/voiceChannel';

const INITIAL_CLOCK_MS = 10 * 60 * 1000; // 10+0 display (server clocks = Phase 5)
const PROMO_PIECES = ['q', 'r', 'b', 'n'];
const PROMO_LABEL = { q: '♛', r: '♜', b: '♝', n: '♞' };

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
  const [whiteClockMs, setWhiteClockMs] = useState(INITIAL_CLOCK_MS);
  const [blackClockMs, setBlackClockMs] = useState(INITIAL_CLOCK_MS);
  const [checkSquare, setCheckSquare] = useState(null);
  const [pendingPromo, setPendingPromo] = useState(null); // { from, to }
  const [moveAnimKey, setMoveAnimKey] = useState(0);
  const [drawOfferFromOpp, setDrawOfferFromOpp] = useState(false);
  const [drawOfferSent, setDrawOfferSent] = useState(false);
  const [matchType, setMatchType] = useState('ranked');

  const socketRef = useRef(location.state?.socket || null);
  const voiceRef = useRef(null);
  const [micOn, setMicOn] = useState(false);
  const [micMuted, setMicMuted] = useState(false);
  const [micError, setMicError] = useState('');
  const isInitiatorRef = useRef(!!location.state?.socket); // queue handoff side often has socket
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
      if (from && to) { setLastMove({ from, to }); setMoveAnimKey((k) => k + 1); }
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
          const before = chessRef.current.fen();
          let wasCapture = false;
          if (msg.from && msg.to) {
            try {
              const target = chessRef.current.get(msg.to);
              wasCapture = !!target;
            } catch (_) {}
          }
          applyServerFen(msg.fen, msg.from, msg.to);
          if (typeof msg.white_ms === 'number') setWhiteClockMs(msg.white_ms);
          if (typeof msg.black_ms === 'number') setBlackClockMs(msg.black_ms);
          try {
            if (chessRef.current.inCheck()) soundCheck();
            else if (wasCapture) soundCapture();
            else soundMove();
          } catch (_) {
            soundMove();
          }
          setStatusLine('Move applied');
        });

        socket.on('board_sync', (msg) => {
          if (msg.color) {
            setColor(msg.color);
            myColorRef.current = msg.color;
          }
          applyServerFen(msg.fen, null, null);
          if (typeof msg.white_ms === 'number') setWhiteClockMs(msg.white_ms);
          if (typeof msg.black_ms === 'number') setBlackClockMs(msg.black_ms);
          setStatusLine('Board synced');
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
          setDrawOfferFromOpp(false);
          setDrawOfferSent(false);
          soundGameEnd();
        });

        socket.on('draw_offered', (msg) => {
          setDrawOfferFromOpp(true);
          setDrawOfferSent(false);
          setToast({ message: 'Opponent offers a draw' });
          soundDrawOffer();
        });

        socket.on('draw_declined', () => {
          setDrawOfferSent(false);
          setDrawOfferFromOpp(false);
          setToast({ message: 'Draw declined' });
        });

        socket.on('webrtc_signal', async (msg) => {
          if (!voiceRef.current) return;
          try {
            await voiceRef.current.handleSignal(msg.payload);
          } catch (_) {}
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
      voiceRef.current?.stop();
      voiceRef.current = null;
    };
  }, [matchId, accessToken, applyServerFen, user?.username]);

  // Client-side clocks (display). Server authority = Phase 5.
  useEffect(() => {
    if (matchEnded) return undefined;
    const id = setInterval(() => {
      const turn = chessRef.current.turn();
      if (turn === 'w') {
        setWhiteClockMs((ms) => Math.max(0, ms - 250));
      } else {
        setBlackClockMs((ms) => Math.max(0, ms - 250));
      }
    }, 250);
    return () => clearInterval(id);
  }, [matchEnded, fen]);

  function formatClock(ms) {
    const s = Math.max(0, Math.floor(ms / 1000));
    const m = Math.floor(s / 60);
    const r = s % 60;
    return `${m}:${r.toString().padStart(2, '0')}`;
  }

  function updateCheckSquare() {
    try {
      if (chessRef.current.inCheck()) {
        const turn = chessRef.current.turn(); // side in check
        const board = chessRef.current.board();
        for (let r = 0; r < 8; r++) {
          for (let c = 0; c < 8; c++) {
            const p = board[r][c];
            if (p && p.type === 'k' && p.color === turn) {
              const sq = 'abcdefgh'[c] + (8 - r);
              setCheckSquare(sq);
              return;
            }
          }
        }
      }
    } catch (_) {}
    setCheckSquare(null);
  }

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
        if (isPromotion) {
          setPendingPromo({ from: selectedSquare, to: square });
          setSelectedSquare(null);
          setLegalTargets([]);
          return;
        }
        setLastMove({ from: selectedSquare, to: square });
        socketRef.current?.move(matchId, selectedSquare, square, undefined);
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

  function confirmPromo(piece) {
    if (!pendingPromo) return;
    const { from, to } = pendingPromo;
    setPendingPromo(null);
    setLastMove({ from, to });
    socketRef.current?.move(matchId, from, to, piece);
    setStatusLine('Sending promotion…');
  }

  async function handleEnableVoice() {
    setMicError('');
    try {
      const socket = socketRef.current;
      if (!socket) throw new Error('Not connected');
      const vc = new VoiceChannel({
        matchId,
        isInitiator: true,
        sendSignal: (payload) => socket.webrtcSignal(matchId, payload),
      });
      voiceRef.current = vc;
      await vc.start();
      setMicOn(true);
      setMicMuted(false);
      setToast({ message: 'Microphone on — opponent must enable voice too' });
    } catch (e) {
      setMicError(e.message || 'Mic failed');
      setToast({ message: e.message || 'Could not access microphone' });
    }
  }

  function handleToggleMute() {
    if (!voiceRef.current || !micOn) return;
    const next = !micMuted;
    voiceRef.current.setMuted(next);
    setMicMuted(next);
  }

  function handleResign() {
    if (window.confirm('Resign this match?')) {
      socketRef.current?.resign(matchId);
    }
  }

  function handleOfferDraw() {
    if (drawOfferSent || drawOfferFromOpp) return;
    socketRef.current?.offerDraw(matchId);
    setDrawOfferSent(true);
    setToast({ message: 'Draw offer sent' });
  }

  function handleAcceptDraw() {
    socketRef.current?.acceptDraw(matchId);
    setDrawOfferFromOpp(false);
  }

  function handleDeclineDraw() {
    socketRef.current?.declineDraw(matchId);
    setDrawOfferFromOpp(false);
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

      <div className="ck-board__clocks">
        <div className={`ck-board__clock ${color === 'black' ? 'ck-board__clock--active' : ''}`}>
          <span className="text-secondary">Opp</span>
          <span className="tabular-nums">{formatClock(color === 'white' ? blackClockMs : whiteClockMs)}</span>
        </div>
        <div className={`ck-board__clock ${color === 'white' ? 'ck-board__clock--active' : ''}`}>
          <span className="text-secondary">You</span>
          <span className="tabular-nums">{formatClock(color === 'white' ? whiteClockMs : blackClockMs)}</span>
        </div>
      </div>

      <div key={moveAnimKey} className="ck-board-anim-wrap">
        <Board
          fen={fen}
          orientation={color}
          onMove={handleSquareTap}
          selectedSquare={selectedSquare}
          legalTargets={legalTargets}
          lastMove={lastMove}
          hintMove={hintMove}
          checkSquare={checkSquare}
        />
      </div>

      {pendingPromo && (
        <div className="ck-board__promo">
          <p className="text-secondary">Promote to</p>
          <div className="ck-board__promo-row">
            {PROMO_PIECES.map((pc) => (
              <button key={pc} type="button" className="ck-board__promo-btn" onClick={() => confirmPromo(pc)}>
                {PROMO_LABEL[pc]}
              </button>
            ))}
          </div>
          <button type="button" className="text-secondary" onClick={() => setPendingPromo(null)}>
            Cancel
          </button>
        </div>
      )}

      <div className="ck-board__toolbar">
        {!micOn ? (
          <button
            type="button"
            className="icon-tap-target"
            aria-label="Enable voice"
            title="Enable voice chat"
            onClick={handleEnableVoice}
          >
            🎙️
          </button>
        ) : (
          <button
            type="button"
            className="icon-tap-target"
            aria-label={micMuted ? 'Unmute' : 'Mute'}
            title={micMuted ? 'Unmute' : 'Mute'}
            onClick={handleToggleMute}
          >
            {micMuted ? '🔇' : '🎙️'}
          </button>
        )}
        <button type="button" className="ck-board__hint" onClick={handleHint} disabled={hintLoading}>
          {hintLoading ? '…' : 'Hint'}
        </button>
        <button type="button" onClick={() => setShowGiftPicker(true)}>
          Gift
        </button>
        <button type="button" className="ck-board__resign" onClick={handleResign}>
          Resign
        </button>
        <button
          type="button"
          className="ck-board__draw"
          onClick={handleOfferDraw}
          disabled={drawOfferSent || drawOfferFromOpp}
        >
          {drawOfferSent ? 'Draw sent…' : 'Offer draw'}
        </button>
      </div>

      {drawOfferFromOpp && (
        <div className="ck-board__draw-banner">
          <span>Opponent offers a draw</span>
          <button type="button" onClick={handleAcceptDraw}>Accept</button>
          <button type="button" onClick={handleDeclineDraw}>Decline</button>
        </div>
      )}

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
