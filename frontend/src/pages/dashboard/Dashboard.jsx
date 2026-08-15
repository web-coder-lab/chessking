import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Toast from '../../components/common/Toast';
import { socialApi } from '../../services/api';
import './Dashboard.css';

const SHORTCUTS = [
  { key: 'shop', label: 'Shop', icon: '🛍️', to: '/shop' },
  { key: 'inventory', label: 'Inventory', icon: '🎒', to: '/inventory' },
  { key: 'custom', label: 'Custom Match', icon: '⚔️', to: '/custom-match' },
  { key: 'invite', label: 'Invite Friend', icon: '🔗', to: '/invite' },
];

export default function Dashboard({ user, refreshUser }) {
  const navigate = useNavigate();
  const [dailyReward, setDailyReward] = useState(null);
  const [recentMatches, setRecentMatches] = useState([]);
  const [claiming, setClaiming] = useState(false);
  const [toast, setToast] = useState(null);

  useEffect(() => {
    if (!user?.username) return;
    function loadDaily() {
      socialApi.getDailyRewardStatus().then((s) => setDailyReward({
        claimedToday: s.claimed_today,
        streakDay: s.current_streak_day,
        coins: s.next_reward_coins,
      })).catch(() => {});
    }
    loadDaily();
    socialApi.getMatchHistory(user.username, 3).then((d) => setRecentMatches(d.matches || [])).catch(() => {});
    // Part 13: when tab focuses again, re-sync coins + claim state from server
    function onVis() {
      if (document.visibilityState === 'visible') {
        loadDaily();
        refreshUser?.();
      }
    }
    document.addEventListener('visibilitychange', onVis);
    return () => document.removeEventListener('visibilitychange', onVis);
  }, [user?.username, refreshUser]);

  async function handleClaim() {
    setClaiming(true);
    try {
      const resp = await socialApi.claimDailyReward();
      setDailyReward((d) =>
        d
          ? { ...d, claimedToday: true, streakDay: resp.new_streak_day ?? d.streakDay }
          : { claimedToday: true, streakDay: resp.new_streak_day ?? 1, coins: 0 }
      );
      setToast({
        message: resp.coins_awarded
          ? `+${resp.coins_awarded} coins claimed`
          : 'Reward claimed',
      });
      await refreshUser?.();
      // Re-fetch status so claimed_today stays true after soft refresh
      const s = await socialApi.getDailyRewardStatus();
      setDailyReward({
        claimedToday: s.claimed_today,
        streakDay: s.current_streak_day,
        coins: s.next_reward_coins,
      });
    } catch (e) {
      if (e.code === 'already_claimed') {
        setDailyReward((d) => (d ? { ...d, claimedToday: true } : d));
        setToast({ message: 'Already claimed today' });
      } else {
        setToast({ message: e.message || 'Could not claim reward' });
      }
    } finally {
      setClaiming(false);
    }
  }

  return (
    <div className="ck-dashboard">
      <TopBar
        avatarUser={user}
        coinBalance={user?.coin_balance ?? 0}
        hasUnread={user?.hasUnreadNotifications}
        onBellClick={() => navigate('/notifications')}
      />

      <main className="ck-dashboard__body">
        {/* §2.3 item 1: Daily reward banner — only shown if today unclaimed */}
        {dailyReward && !dailyReward.claimedToday && (
          <Card glass className="ck-dashboard__daily-reward">
            <div>
              <p className="ck-dashboard__daily-reward-title">
                Day {dailyReward.streakDay} — Claim {dailyReward.coins} coins
              </p>
            </div>
            <button className="ck-dashboard__claim-btn" onClick={handleClaim} disabled={claiming}>
              {claiming ? '...' : 'Claim'}
            </button>
          </Card>
        )}

        {/* §2.3 item 2: Big central Play button, pulsing glow */}
        <div className="ck-dashboard__play-wrap">
          <button className="ck-dashboard__play-button" onClick={() => navigate('/play')}>
            <span className="ck-dashboard__play-icon" aria-hidden="true">♞</span>
            <span className="ck-dashboard__play-label">PLAY</span>
          </button>
        </div>

        {/* §2.3 item 3: Quick-shortcut grid, tap-scale animation */}
        <div className="ck-dashboard__shortcut-grid">
          {SHORTCUTS.map((s) => (
            <button
              key={s.key}
              className="ck-dashboard__shortcut-card"
              onClick={() => navigate(s.to)}
            >
              <span className="ck-dashboard__shortcut-icon" aria-hidden="true">{s.icon}</span>
              <span className="ck-dashboard__shortcut-label">{s.label}</span>
            </button>
          ))}
        </div>

        {/* §2.3 item 4: Recent Matches mini-list (last 3) */}
        {recentMatches.length > 0 && (
          <section>
            <h2 className="section-heading" style={{ marginBottom: 'var(--space-3)' }}>
              Recent Matches
            </h2>
            <div className="ck-dashboard__recent-list">
              {recentMatches.slice(0, 3).map((m) => (
                <Card key={m.id} className="ck-dashboard__recent-row">
                  <span className="text-secondary" style={{ flex: 1 }}>
                    {m.opponent_username ? `vs ${m.opponent_username}` : m.match_type}
                  </span>
                  <span
                    className={`ck-dashboard__result-badge ck-dashboard__result-badge--${m.result === 'win' ? 'win' : m.result === 'draw' ? 'draw' : 'loss'}`}
                  >
                    {m.result === 'win' ? 'W' : m.result === 'draw' ? 'D' : 'L'}
                  </span>
                </Card>
              ))}
            </div>
          </section>
        )}
      </main>

      <BottomNav />

      <Toast
        visible={!!toast}
        message={toast?.message}
        onDismiss={() => setToast(null)}
      />
    </div>
  );
}
