import { avatarEmoji, bannerEmoji } from '../../utils/avatar';
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
  // Prefer route param; else logged-in user (own profile)
  const username = routeUsername || user?.username || null;
  const isMe = !routeUsername || username === user?.username;

  const [profile, setProfile] = useState(null);
  const [loadError, setLoadError] = useState('');
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState(searchParams.get('tab') === 'gifts' ? 'gifts' : 'history');
  const [matches, setMatches] = useState([]);
  const [gifts, setGifts] = useState([]);
  const [showGiftPicker, setShowGiftPicker] = useState(false);
  const [playingGift, setPlayingGift] = useState(null);
  const [toast, setToast] = useState(null);

  useEffect(() => {
    let cancelled = false;
    setLoadError('');
    setLoading(true);

    async function load() {
      try {
        let p = null;
        let name = username;

        // Own profile: use /profile/me (works even if user prop still hydrating)
        if (!routeUsername) {
          p = await socialApi.getMyProfile();
          name = p.username;
        } else {
          p = await socialApi.getPublicProfile(routeUsername);
          name = routeUsername;
        }
        if (cancelled) return;
        setProfile(p);

        try {
          const d = await socialApi.getMatchHistory(name);
          if (!cancelled) setMatches(d.matches || []);
        } catch {
          if (!cancelled) setMatches([]);
        }
      } catch (e) {
        if (!cancelled) {
          setProfile(null);
          setLoadError(e.message || 'Could not load profile');
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    // Wait briefly for user if on /profile without username yet
    if (!routeUsername && !user?.username) {
      // Still try /profile/me — needs access token from AuthContext
      load();
      return () => {
        cancelled = true;
      };
    }
    load();
    return () => {
      cancelled = true;
    };
  }, [routeUsername, username, user?.username]);

  useEffect(() => {
    if (tab !== 'gifts' || !profile) return;
    const name = profile.username || username;
    if (!name) return;
    giftsApi
      .receivedTally(name)
      .then(setGifts)
      .catch(() => setGifts([]));
  }, [tab, profile, username]);

  async function handleSendGift(item) {
    try {
      const name = profile?.username || username;
      await giftsApi.send(name, item.id, 'profile');
      setShowGiftPicker(false);
      setPlayingGift(item);
      if (tab === 'gifts') {
        giftsApi.receivedTally(name).then(setGifts).catch(() => {});
      }
    } catch (err) {
      setToast({ message: err.message });
    }
  }

  if (loading) {
    return (
      <div className="ck-profile" style={{ padding: 24, textAlign: 'center', color: 'var(--accent-gold)' }}>
        <p>♚ Loading profile…</p>
        <BottomNav />
      </div>
    );
  }

  if (loadError || !profile) {
    return (
      <div className="ck-profile" style={{ padding: 24, textAlign: 'center' }}>
        <p style={{ color: 'var(--danger-red)' }}>{loadError || 'Profile not found'}</p>
        <Button onClick={() => navigate('/dashboard')} style={{ marginTop: 16 }}>
          Go Home
        </Button>
        <BottomNav />
      </div>
    );
  }

  const displayName = profile.username || username || 'Player';
  const wins = matches.filter((m) => m.result === 'win').length;
  const losses = matches.filter((m) => m.result === 'loss').length;
  const winRate = matches.length ? Math.round((wins / matches.length) * 100) : 0;

  return (
    <div className="ck-profile">
      <div
        className="ck-profile__banner"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 48,
          background: 'linear-gradient(135deg,#1A1D23,#252018)',
        }}
      >
        {bannerEmoji(profile.banner_id || user?.banner_id)}
      </div>
      <div
        className="ck-profile__avatar"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 40,
          background: '#1A1D23',
        }}
      >
        {avatarEmoji(profile.avatar_id || user?.avatar_id)}
      </div>

      <main className="ck-profile__body">
        <div className="ck-profile__header-row">
          <h1 className="page-title">{displayName}</h1>
          {isMe && (
            <>
              <button
                className="ck-profile__edit-btn icon-tap-target"
                onClick={() => navigate('/profile/settings')}
                aria-label="Edit profile"
              >
                ✏️
              </button>
              <button
                className="ck-profile__edit-btn icon-tap-target"
                onClick={() => navigate('/settings')}
                aria-label="Settings"
              >
                ⚙️
              </button>
            </>
          )}
        </div>

        {profile.bio && <p className="text-secondary ck-profile__bio">{profile.bio}</p>}

        <div className="ck-profile__stats">
          <Stat label="Rating" value={profile.rating ?? 1200} />
          <Stat label="Wins" value={wins} />
          <Stat label="Losses" value={losses} />
          <Stat label="Win %" value={`${winRate}%`} />
        </div>

        {!isMe && (
          <Button onClick={() => setShowGiftPicker(true)} style={{ marginBottom: 'var(--space-4)' }}>
            Send Gift
          </Button>
        )}

        <div className="ck-shop__segments" style={{ marginBottom: 'var(--space-4)' }}>
          <button
            type="button"
            role="tab"
            aria-selected={tab === 'history'}
            className={`ck-shop__segment ${tab === 'history' ? 'ck-shop__segment--active' : ''}`}
            onClick={() => setTab('history')}
          >
            Match History
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === 'gifts'}
            className={`ck-shop__segment ${tab === 'gifts' ? 'ck-shop__segment--active' : ''}`}
            onClick={() => setTab('gifts')}
          >
            Gifts Received
          </button>
        </div>

        {tab === 'history' &&
          (matches.length === 0 ? (
            <EmptyState icon="♟️" text="No matches played yet" />
          ) : (
            <div className="ck-profile__match-list">
              {matches.map((m) => (
                <Card key={m.id} className="ck-profile__match-row">
                  <span className="text-secondary">{m.match_type}</span>
                  {m.opponent_username && <span style={{ flex: 1 }}>vs {m.opponent_username}</span>}
                  {!m.opponent_username && <span style={{ flex: 1 }} />}
                  <span
                    className={`ck-dashboard__result-badge ck-dashboard__result-badge--${
                      m.result === 'win' ? 'win' : m.result === 'draw' ? 'draw' : 'loss'
                    }`}
                  >
                    {m.result === 'win' ? 'W' : m.result === 'draw' ? 'D' : 'L'}
                  </span>
                </Card>
              ))}
            </div>
          ))}

        {tab === 'gifts' &&
          (gifts.length === 0 ? (
            <EmptyState icon="🎁" text="No gifts received yet" />
          ) : (
            <div className="ck-profile__gift-grid">
              {gifts.map((g) => (
                <div key={g.shop_item_id} className="ck-profile__gift-item">
                  <span>🎁</span>
                  <span className="tabular-nums">x{g.count}</span>
                </div>
              ))}
            </div>
          ))}
      </main>

      {showGiftPicker && (
        <GiftPicker
          recipientLabel={displayName}
          onSend={handleSendGift}
          onClose={() => setShowGiftPicker(false)}
        />
      )}

      {playingGift && <GiftAnimation item={playingGift} onComplete={() => setPlayingGift(null)} />}

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
