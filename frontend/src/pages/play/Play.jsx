import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import { useAuth } from '../../context/AuthContext';
import { GameSocket } from '../../services/gameSocket';
import './Play.css';

const SEARCH_TIMEOUT_MS = 90_000;

export default function Play({ user }) {
  const navigate = useNavigate();
  const { accessToken } = useAuth();
  const [phase, setPhase] = useState('choose'); // choose | searching | found | error
  const [elapsed, setElapsed] = useState(0);
  const [matchInfo, setMatchInfo] = useState(null);
  const [errorMsg, setErrorMsg] = useState('');
  const socketRef = useRef(null);
  const timerRef = useRef(null);
  const searchTimeoutRef = useRef(null);
  const handoffRef = useRef(false);

  useEffect(() => () => {
    clearInterval(timerRef.current);
    clearTimeout(searchTimeoutRef.current);
    if (!handoffRef.current) socketRef.current?.close();
  }, []);

  function resetToChoose(msg) {
    clearInterval(timerRef.current);
    clearTimeout(searchTimeoutRef.current);
    if (!handoffRef.current) socketRef.current?.close();
    socketRef.current = null;
    setPhase(msg ? 'error' : 'choose');
    setErrorMsg(msg || '');
    setElapsed(0);
    setMatchInfo(null);
  }

  async function startQuickMatch(matchType) {
    if (!accessToken) {
      navigate('/auth', { replace: true });
      return;
    }

    setErrorMsg('');
    setPhase('searching');
    setElapsed(0);
    clearInterval(timerRef.current);
    timerRef.current = setInterval(() => setElapsed((e) => e + 1), 1000);

    clearTimeout(searchTimeoutRef.current);
    searchTimeoutRef.current = setTimeout(() => {
      resetToChoose('No opponent found yet. Try again — or invite a friend (Custom Match).');
    }, SEARCH_TIMEOUT_MS);

    try {
      const socket = new GameSocket(accessToken);
      socketRef.current = socket;
      await socket.connect('/match/queue');

      socket.on('match_found', (msg) => {
        clearInterval(timerRef.current);
        clearTimeout(searchTimeoutRef.current);
        setMatchInfo(msg);
        setPhase('found');
        setTimeout(() => {
          handoffRef.current = true;
          navigate(`/board/${msg.match_id}`, {
            state: {
              socket,
              color: msg.color,
              opponentId: msg.opponent_id,
            },
          });
        }, 1200);
      });

      socket.on('error', (msg) => {
        resetToChoose(msg.message || 'Matchmaking error');
      });

      socket.on('__closed', () => {
        // If we already found a match and handed off, ignore
        if (handoffRef.current) return;
        // Still searching → connection died
        setPhase((p) => {
          if (p === 'searching') {
            clearInterval(timerRef.current);
            clearTimeout(searchTimeoutRef.current);
            setErrorMsg('Connection lost while searching. Try again.');
            return 'error';
          }
          return p;
        });
      });

      socket.joinQueue(matchType);
    } catch (e) {
      resetToChoose(e.message || 'Could not start matchmaking');
    }
  }

  function cancelSearch() {
    handoffRef.current = false;
    resetToChoose('');
  }

  if (phase === 'searching') {
    return (
      <div className="ck-play-fullscreen">
        <div className="ck-play-radar" aria-hidden>♞</div>
        <h1 className="page-title">Searching…</h1>
        <p className="text-secondary" style={{ marginTop: 8 }}>
          Looking for an opponent · {elapsed}s
        </p>
        <p className="text-secondary" style={{ marginTop: 8, fontSize: 13, maxWidth: 280 }}>
          Needs another online player. On free server only you may be online — try Custom Match or open a second account.
        </p>
        <Button variant="outline" onClick={cancelSearch} style={{ marginTop: 'var(--space-8)' }}>
          Cancel
        </Button>
      </div>
    );
  }

  if (phase === 'found' && matchInfo) {
    return (
      <div className="ck-play-fullscreen">
        <h1 className="page-title">Match found!</h1>
        <div className="ck-vs-row" style={{ marginTop: 24, display: 'flex', alignItems: 'center', gap: 16, justifyContent: 'center' }}>
          <div className="ck-vs-player">
            <span style={{ fontSize: 40 }}>👤</span>
            <span>{user?.username || 'You'}</span>
          </div>
          <span className="ck-vs-label">VS</span>
          <div className="ck-vs-player">
            <span style={{ fontSize: 40 }}>👤</span>
            <span>{matchInfo.opponent_username || 'Opponent'}</span>
          </div>
        </div>
        <p className="text-secondary" style={{ marginTop: 12 }}>You play as {matchInfo.color}</p>
      </div>
    );
  }

  return (
    <div className="ck-play">
      <TopBar avatarUser={user} coinBalance={user?.coin_balance ?? 0} onBellClick={() => navigate('/notifications')} />

      <main className="ck-play__body">
        <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>Play</h1>

        {phase === 'error' && errorMsg && (
          <p style={{ color: 'var(--danger-red)', marginBottom: 16, fontSize: 14 }}>{errorMsg}</p>
        )}

        <Card className="ck-play__option-card" onClick={() => startQuickMatch('ranked')}>
          <span className="ck-play__option-icon" aria-hidden="true">⚡</span>
          <div>
            <h2 className="section-heading">Quick Match</h2>
            <p className="text-secondary">Random opponent, rating-based</p>
          </div>
        </Card>

        <Card className="ck-play__option-card" onClick={() => navigate('/custom-match')}>
          <span className="ck-play__option-icon" aria-hidden="true">👥</span>
          <div>
            <h2 className="section-heading">Custom Match</h2>
            <p className="text-secondary">Invite a specific friend</p>
          </div>
        </Card>

        <Card className="ck-play__option-card" onClick={() => startQuickMatch('casual')}>
          <span className="ck-play__option-icon" aria-hidden="true">🎲</span>
          <div>
            <h2 className="section-heading">Casual Match</h2>
            <p className="text-secondary">Unranked — hints allowed</p>
          </div>
        </Card>
      </main>

      <BottomNav />
    </div>
  );
}
