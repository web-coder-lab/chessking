import { Link } from 'react-router-dom';
import './TopBar.css';

/**
 * Doc 4 §2.3 Top bar: fixed 56px. Left = circular avatar (32px) -> Profile.
 * Center-left = coin balance pill ("🪙 1,250") -> Wallet.
 * Right = notification bell (badge dot if unread) -> drawer.
 */
export default function TopBar({ avatarUrl, coinBalance, hasUnread, onBellClick }) {
  return (
    <header className="ck-topbar">
      <Link to="/profile" className="ck-topbar__avatar-link" aria-label="Profile">
        <img
          src={avatarUrl || '/assets/default-avatar.svg'}
          alt=""
          className="ck-topbar__avatar"
        />
      </Link>

      <Link to="/wallet" className="ck-topbar__coins">
        <span aria-hidden="true">🪙</span>
        <span className="tabular-nums">{coinBalance.toLocaleString()}</span>
      </Link>

      <button
        className="ck-topbar__bell icon-tap-target"
        onClick={onBellClick}
        aria-label="Notifications"
      >
        🔔
        {hasUnread && <span className="ck-topbar__badge" aria-hidden="true" />}
      </button>
    </header>
  );
}
