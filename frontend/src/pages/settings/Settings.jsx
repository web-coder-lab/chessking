import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';
import { socialApi } from '../../services/api';
import './Settings.css';

function SettingsRow({ icon, label, value, onClick, danger, toggle, onToggle }) {
  return (
    <button className={`ck-settings__row ${danger ? 'ck-settings__row--danger' : ''}`} onClick={onClick} disabled={!onClick && !toggle}>
      <span className="ck-settings__row-icon" aria-hidden="true">{icon}</span>
      <span className="ck-settings__row-label">{label}</span>
      {toggle ? (
        <label className="ck-settings__switch">
          <input type="checkbox" checked={value} onChange={(e) => onToggle(e.target.checked)} />
          <span className="ck-settings__switch-track" />
        </label>
      ) : (
        <>
          {value !== undefined && <span className="text-secondary">{value}</span>}
          {onClick && <span className="text-secondary" aria-hidden="true">›</span>}
        </>
      )}
    </button>
  );
}

export default function Settings({ user }) {
  const navigate = useNavigate();
  const { logout } = useAuth();
  const [notifsEnabled, setNotifsEnabled] = useState(true);

  function handleToggleNotifs(next) {
    setNotifsEnabled(next); // optimistic
    socialApi.updateNotificationSettings(next).catch(() => setNotifsEnabled(!next)); // revert on failure
  }

  return (
    <div className="ck-settings">
      <h1 className="page-title" style={{ padding: 'var(--space-4) var(--screen-padding-x) 0' }}>Settings</h1>

      <section className="ck-settings__section">
        <h2 className="ck-settings__section-title">Security</h2>
        <SettingsRow icon="🔐" label="Two-Step Verification" value={user?.two_fa_enabled ? 'On' : 'Off'} onClick={() => navigate('/settings/2fa')} />
        <SettingsRow icon="📱" label="Active Sessions" onClick={() => navigate('/settings/sessions')} />
      </section>

      <section className="ck-settings__section">
        <h2 className="ck-settings__section-title">Preferences</h2>
        <SettingsRow icon="🔔" label="Notifications" toggle value={notifsEnabled} onToggle={handleToggleNotifs} />
        <SettingsRow icon="🌐" label="Language" value="English" />
      </section>

      <section className="ck-settings__section">
        <h2 className="ck-settings__section-title">Support</h2>
        <SettingsRow icon="🐞" label="Bug Report" onClick={() => navigate('/settings/bug-report')} />
        <SettingsRow icon="💬" label="Support Team" onClick={() => navigate('/settings/support')} />
        <SettingsRow icon="📄" label="Privacy Policy" onClick={() => navigate('/settings/privacy-policy')} />
        <SettingsRow icon="ℹ️" label="About" onClick={() => navigate('/settings/about')} />
      </section>

      <section className="ck-settings__section">
        <h2 className="ck-settings__section-title">Account</h2>
        <SettingsRow icon="🚪" label="Logout" danger onClick={() => { logout(); navigate('/auth'); }} />
      </section>
    </div>
  );
}
