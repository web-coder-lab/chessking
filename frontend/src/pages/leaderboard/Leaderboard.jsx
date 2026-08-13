import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import { socialApi } from '../../services/api';
import './Leaderboard.css';

const SCOPES = [
  { key: 'global', label: 'Global' },
  { key: 'country', label: 'Country' },
  { key: 'province', label: 'Province' },
];

export default function Leaderboard({ user }) {
  const navigate = useNavigate();
  const [scope, setScope] = useState('global');
  const [rankings, setRankings] = useState([]);
  const [myRank, setMyRank] = useState(null);

  const scopeValue = scope === 'country' ? user?.country_code : scope === 'province' ? user?.province : null;

  useEffect(() => {
    socialApi.getLeaderboard(scope, scopeValue).then((data) => {
      setRankings(data.rankings);
      setMyRank(data.my_rank);
    });
  }, [scope, scopeValue]);

  const podium = rankings.slice(0, 3);
  const rest = rankings.slice(3);

  return (
    <div className="ck-leaderboard">
      <TopBar avatarUrl={user?.avatarUrl} coinBalance={user?.coin_balance ?? 0} onBellClick={() => navigate('/notifications')} />

      <main className="ck-leaderboard__body">
        <div className="ck-shop__segments" role="tablist">
          {SCOPES.map((s) => (
            <button
              key={s.key}
              role="tab"
              aria-selected={scope === s.key}
              className={`ck-shop__segment ${scope === s.key ? 'ck-shop__segment--active' : ''}`}
              onClick={() => setScope(s.key)}
            >
              {s.label}
            </button>
          ))}
        </div>

        {podium.length === 3 && (
          <div className="ck-leaderboard__podium">
            <PodiumSpot row={podium[1]} size="small" ring="#C0C0C0" />
            <PodiumSpot row={podium[0]} size="large" ring="var(--accent-gold)" />
            <PodiumSpot row={podium[2]} size="small" ring="#CD7F32" />
          </div>
        )}

        <div className="ck-leaderboard__list">
          {rest.map((row) => (
            <Card key={row.rank} className="ck-leaderboard__row" onClick={() => navigate(`/profile/${row.username}`)}>
              <span className="ck-leaderboard__rank tabular-nums">{row.rank}</span>
              <img src="/assets/default-avatar.svg" alt="" className="ck-leaderboard__avatar" />
              <span style={{ flex: 1 }}>{row.username}</span>
              <span className="tabular-nums">{row.rating}</span>
            </Card>
          ))}
        </div>
      </main>

      {myRank && (
        <div className="ck-leaderboard__sticky-me">
          <span className="tabular-nums">#{myRank}</span>
          <span style={{ flex: 1 }}>{user?.username} (You)</span>
        </div>
      )}

      <BottomNav />
    </div>
  );
}

function PodiumSpot({ row, size, ring }) {
  const navigate = useNavigate();
  const avatarSize = size === 'large' ? 64 : 48;
  return (
    <div className="ck-leaderboard__podium-spot" onClick={() => row?.username && navigate(`/profile/${row.username}`)}>
      <img
        src="/assets/default-avatar.svg"
        alt=""
        style={{ width: avatarSize, height: avatarSize, border: `3px solid ${ring}` }}
        className="ck-leaderboard__podium-avatar"
      />
      <span className="ck-leaderboard__podium-name">{row?.username}</span>
      <span className="text-secondary tabular-nums">{row?.rating}</span>
    </div>
  );
}
