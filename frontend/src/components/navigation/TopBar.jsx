import { Link } from 'react-router-dom';
import { avatarEmoji, avatarImageUrl } from '../../utils/avatar';
import './TopBar.css';

export default function TopBar({ avatarUser, avatarUrl, coinBalance, hasUnread, onBellClick }) {
  const img = avatarImageUrl(avatarUser) || (avatarUrl && String(avatarUrl).startsWith('http') ? avatarUrl : null);
  const emoji = avatarEmoji(avatarUser?.avatar_id);

  return (
    <header className="ck-topbar">
      <Link to="/profile" className="ck-topbar__avatar-link" aria-label="Profile">
        {img ? (
          <img src={img} alt="" className="ck-topbar__avatar" />
        ) : (
          <span className="ck-topbar__avatar ck-topbar__avatar--emoji" aria-hidden>
            {emoji}
          </span>
        )}
      </Link>

      <Link to="/wallet" className="ck-topbar__coins">
        <span aria-hidden="true">🪙</span>
        <span className="tabular-nums">{(coinBalance ?? 0).toLocaleString()}</span>
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
