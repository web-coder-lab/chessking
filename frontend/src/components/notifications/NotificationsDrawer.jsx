import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { socialApi } from '../../services/api';
import './NotificationsDrawer.css';

const TYPE_ICON = {
  login_from_new_device: '🔐', custom_match_invite: '⚔️', gift_received: '🎁',
  daily_reward: '📅', report_status_update: '📋', referral_reward: '🔗',
};

export default function NotificationsDrawer({ open, onClose }) {
  const navigate = useNavigate();
  const [notifications, setNotifications] = useState([]);

  useEffect(() => {
    if (open) {
      socialApi.getNotifications().then((d) => setNotifications(d.notifications));
    }
  }, [open]);

  async function handleTap(n) {
    if (!n.is_read) {
      await socialApi.markNotificationRead(n.id);
      setNotifications((prev) => prev.map((x) => (x.id === n.id ? { ...x, is_read: 1 } : x)));
    }

    switch (n.type) {
      case 'custom_match_invite':
        navigate('/custom-match');
        break;
      case 'gift_received':
        navigate('/profile?tab=gifts');
        break;
      case 'login_from_new_device':
        navigate('/settings/sessions');
        break;
      case 'referral_reward':
        navigate('/invite');
        break;
      case 'daily_reward':
        navigate('/dashboard');
        break;
      default:
        // report_status_update and any unrecognized future type: no
        // single obvious destination exists yet, so just mark read
        // rather than guess at a navigation target.
        break;
    }
    onClose();
  }

  if (!open) return null;

  return (
    <div className="ck-notif-drawer__overlay" onClick={onClose}>
      <div className="ck-notif-drawer" onClick={(e) => e.stopPropagation()}>
        <h2 className="section-heading" style={{ padding: 'var(--space-4)' }}>Notifications</h2>
        <div className="ck-notif-drawer__list">
          {notifications.map((n) => (
            <button key={n.id} className={`ck-notif-drawer__row ${!n.is_read ? 'ck-notif-drawer__row--unread' : ''}`} onClick={() => handleTap(n)}>
              <span className="ck-notif-drawer__icon" aria-hidden="true">{TYPE_ICON[n.type] ?? '🔔'}</span>
              <div className="ck-notif-drawer__text">
                <span>{n.title}</span>
                <span className="text-secondary" style={{ fontSize: 'var(--text-caption-size)' }}>
                  {new Date(n.created_at).toLocaleString()}
                </span>
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
