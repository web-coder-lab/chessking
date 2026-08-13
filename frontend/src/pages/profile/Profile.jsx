import { useEffect, useState } from 'react';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import EmptyState from '../../components/common/EmptyState';
import Toast from '../../components/common/Toast';
import GiftPicker from '../../components/gifts/GiftPicker';
import GiftAnimation from '../../components/gifts/GiftAnimation';
import { socialApi, giftsApi } from '../../services/api';
import './Profile.css';

export default function Profile({ user }) {
  const navigate = useNavigate();
  const { username: routeUsername } = useParams();
  const [searchParams] = useSearchParams();
  const username = routeUsername || user?.username;
  const isMe = username === user?.username;

  const [profile, setProfile] = useState(null);
  const [tab, setTab] = useState(searchParams.get('tab') === 'gifts' ? 'gifts' : 'history');
  const [matches, setMatches] = useState([]);
  const [gifts, setGifts] = useState([]);
  const [showGiftPicker, setShowGiftPicker] = useState(false);
  const [playingGift, setPlayingGift] = useState(null);
  const [toast, setToast] = useState(null);

  useEffect(() => {
    if (!username) return;
    socialApi.getPublicProfile(username).then(setProfile);
    socialApi.getMatchHistory(username).then((d) => setMatches(d.matches));
  }, [username]);

  useEffect(() => {
    if (tab === 'gifts' && profile) {
      giftsApi.receivedTally(username).then(setGifts);
    }
  }, [tab, profile, username]);

  async function handleSendGift(item) {
    try {
      await giftsApi.send(username, item.id, 'profile');
      setShowGiftPicker(false);
      setPlayingGift(item);
      if (tab === 'gifts') {
        giftsApi.receivedTally(username).then(setGifts);
      }
    } catch (err) {
      setToast({ message: err.message });
    }
  }

  if (!profile) return null;

  const wins = matches.filter((m) => m.result === 'win').length;
  const losses = matches.filter((m) => m.result === 'loss').length;
  const winRate = matches.length ? Math.round((wins / matches.length) * 100) : 0;

  return (
    <div className="ck-profile">
      <div className="ck-profile__banner" />
      <img src="/assets/default-avatar.png" alt="" className="ck-profile__avatar" />

      {isMe && (
        <>
          <button className="ck-profile__edit-btn icon-tap-target" onClick={() => navigate('/profile/settings')} aria-label="Edit profile">
            ✏️
          </button>
          <button
            className="ck-profile__edit-btn icon-tap-target"
            style={{ right: 'calc(var(--space-4) + 48px)' }}
            onClick={() => navigate('/settings')}
            aria-label="Settings"
          >
            ⚙️
          </button>
        </>
      )}

      <main className="ck-profile__body">
        <h1 className="page-title">{profile.username}</h1>
        {profile.country_code && <span className="text-secondary">{profile.country_code}</span>}
        {profile.bio && <p className="text-secondary">{profile.bio}</p>}

        <div className="ck-profile__stats">
          <Stat label="Wins" value={wins} />
          <Stat label="Losses" value={losses} />
          <Stat label="Win Rate" value={`${winRate}%`} />
          <Stat label="Rating" value={profile.rating} />
        </div>

        {!isMe && (
          <Button onClick={() => setShowGiftPicker(true)} style={{ marginBottom: 'var(--space-4)' }}>
            🎁 Send Gift
          </Button>
        )}

        <div className="ck-shop__segments" role="tablist">
          <button role="tab" aria-selected={tab === 'history'} className={`ck-shop__segment ${tab === 'history' ? 'ck-shop__segment--active' : ''}`} onClick={() => setTab('history')}>
            Match History
          </button>
          <button role="tab" aria-selected={tab === 'gifts'} className={`ck-shop__segment ${tab === 'gifts' ? 'ck-shop__segment--active' : ''}`} onClick={() => setTab('gifts')}>
            Gifts Received
          </button>
        </div>

        {tab === 'history' && (
          matches.length === 0 ? <EmptyState icon="♟️" text="No matches played yet" /> : (
            <div className="ck-profile__match-list">
              {matches.map((m) => (
                <Card key={m.id} className="ck-profile__match-row">
                  <span className="text-secondary">{m.match_type}</span>
                  {m.opponent_username && <span style={{ flex: 1 }}>vs {m.opponent_username}</span>}
                  {!m.opponent_username && <span style={{ flex: 1 }} />}
                  <span className={`ck-dashboard__result-badge ck-dashboard__result-badge--${m.result === 'win' ? 'win' : m.result === 'draw' ? 'draw' : 'loss'}`}>
                    {m.result === 'win' ? 'W' : m.result === 'draw' ? 'D' : 'L'}
                  </span>
                </Card>
              ))}
            </div>
          )
        )}

        {tab === 'gifts' && (
          gifts.length === 0 ? <EmptyState icon="🎁" text="No gifts received yet" /> : (
            <div className="ck-profile__gift-grid">
              {gifts.map((g) => (
                <div key={g.shop_item_id} className="ck-profile__gift-item">
                  <span>🎁</span>
                  <span className="tabular-nums">x{g.count}</span>
                </div>
              ))}
            </div>
          )
        )}
      </main>

      {showGiftPicker && (
        <GiftPicker
          recipientLabel={username}
          onSend={handleSendGift}
          onClose={() => setShowGiftPicker(false)}
        />
      )}

      {playingGift && (
        <GiftAnimation item={playingGift} onComplete={() => setPlayingGift(null)} />
      )}

      {toast && <Toast message={toast.message} visible={!!toast} onDismiss={() => setToast(null)} />}

      <BottomNav />
    </div>
  );
}

function Stat({ label, value }) {
  return (
    <div className="ck-profile__stat">
      <span className="ck-profile__stat-value tabular-nums">{value}</span>
      <span className="ck-profile__stat-label text-secondary">{label}</span>
    </div>
  );
}
