import { NavLink } from 'react-router-dom';
import './BottomNav.css';

/**
 * Doc 4 §2.3 Bottom navigation: fixed, 5 tabs — Home, Wallet, Play,
 * Leaderboard, Profile. Play tab: visually elevated, larger icon,
 * FAB-style, sits slightly above the bar line, gold circle background.
 * Active tab: gold icon+label. Inactive: text-secondary.
 */
const TABS = [
  { to: '/dashboard', label: 'Home', icon: '🏠' },
  { to: '/wallet', label: 'Wallet', icon: '👛' },
  { to: '/play', label: 'Play', icon: '♞', isPlay: true },
  { to: '/leaderboard', label: 'Leaderboard', icon: '🏆' },
  { to: '/profile', label: 'Profile', icon: '👤' },
];

export default function BottomNav() {
  return (
    <nav className="ck-bottom-nav" aria-label="Main navigation">
      {TABS.map((tab) =>
        tab.isPlay ? (
          <NavLink
            key={tab.to}
            to={tab.to}
            className="ck-bottom-nav__play"
            aria-label="Play"
          >
            <span className="ck-bottom-nav__play-icon">{tab.icon}</span>
          </NavLink>
        ) : (
          <NavLink
            key={tab.to}
            to={tab.to}
            className={({ isActive }) =>
              `ck-bottom-nav__tab ${isActive ? 'ck-bottom-nav__tab--active' : ''}`
            }
          >
            <span className="ck-bottom-nav__icon" aria-hidden="true">{tab.icon}</span>
            <span className="ck-bottom-nav__label">{tab.label}</span>
          </NavLink>
        )
      )}
    </nav>
  );
}
