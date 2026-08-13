import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import { useAuth } from '../../context/AuthContext';
import { GameSocket } from '../../services/gameSocket';
import './Play.css';

export default function Play({ user }) {
  const navigate = useNavigate();
  const { accessToken } = useAuth();
  const [phase, setPhase] = useState('choose'); // choose | searching | found
  const [elapsed, setElapsed] = useState(0);
  const [matchInfo, setMatchInfo] = useState(null);
  const socketRef = useRef(null);
  const timerRef = useRef(null);
  const handoffRef = useRef(false); // true once ChessBoard is taking over the live socket

  useEffect(() => () => {
    clearInterval(timerRef.current);
    if (!handoffRef.current) socketRef.current?.close();
  }, []);

  async function startQuickMatch(matchType) {
    setPhase('searching');
    setElapsed(0);
    timerRef.current = setInterval(() => setElapsed((e) => e + 1), 1000);

    const socket = new GameSocket(accessToken);
    socketRef.current = socket;
    await socket.connect('/match/queue');

    socket.on('match_found', (msg) => {
      clearInterval(timerRef.current);
      setMatchInfo(msg);
      setPhase('found');
      setTimeout(() => {
        handoffRef.current = true; // ChessBoard reuses this same connection — don't close it on unmount
        navigate(`/board/${msg.match_id}`, { state: { socket, color: msg.color } });
      }, 1500);
    });

    socket.joinQueue(matchType);
  }

  function cancelSearch() {
    clearInterval(timerRef.current);
    socketRef.current?.close();
    setPhase('choose');
  }

  if (phase === 'searching') {
    return (
      <div className="ck-play-fullscreen">
        <div className="ck-play-radar" aria-hidden="true">
          <span className="ck-play-radar-ring" />
          <span className="ck-play-radar-ring" />
          <span className="ck-play-knight">♞</span>
        </div>
        <p className="page-title" style={{ marginTop: 'var(--space-6)' }}>Finding an opponent...</p>
        <p className="text-secondary tabular-nums">{Math.floor(elapsed / 60)}:{String(elapsed % 60).padStart(2, '0')}</p>
        <Button variant="outline" onClick={cancelSearch} style={{ marginTop: 'var(--space-8)' }}>
          Cancel
        </Button>
      </div>
    );
  }

  if (phase === 'found' && matchInfo) {
    return (
      <div className="ck-play-fullscreen">
        <div className="ck-vs-card">
          <div className="ck-vs-player">
            <img src={user?.avatarUrl} alt="" className="ck-vs-avatar" />
            <span>{user?.username}</span>
          </div>
          <span className="ck-vs-label">VS</span>
          <div className="ck-vs-player">
            <img src={matchInfo.opponent_avatar_url} alt="" className="ck-vs-avatar" />
            <span>{matchInfo.opponent_username || 'Opponent'}</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="ck-play">
      <TopBar avatarUrl={user?.avatarUrl} coinBalance={user?.coin_balance ?? 0} onBellClick={() => navigate('/notifications')} />

      <main className="ck-play__body">
        <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>Play</h1>

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
