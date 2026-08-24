import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { socialApi } from '../../services/api';
import { avatarEmoji } from '../../utils/avatar';
import './ProfileSettings.css';

export default function ProfileSettings({ user }) {
  const navigate = useNavigate();
  const [bio, setBio] = useState(user?.bio || '');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [newEmail, setNewEmail] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [emailChangeSent, setEmailChangeSent] = useState(false);
  const [toast, setToast] = useState(null);

  async function handleSave() {
    setSaving(true);
    setError('');
    try {
      await socialApi.updateProfile({ bio });
      if (currentPassword && newPassword) {
        await socialApi.changePassword(currentPassword, newPassword);
      }
      if (currentPassword && newEmail) {
        await socialApi.changeEmail(currentPassword, newEmail);
        setEmailChangeSent(true);
        setToast({ message: 'Check your new email for a confirmation link' });
      } else {
        setToast({ message: 'Profile saved' });
        setTimeout(() => navigate('/profile'), 700);
      }
    } catch (err) {
      setError(err.message || 'Could not save');
    } finally {
      setSaving(false);
    }
  }

  const avatar = avatarEmoji(user?.avatar_id);

  return (
    <div className="ck-edit-profile">
      <header className="ck-edit-profile__header">
        <button type="button" className="ck-edit-profile__back" onClick={() => navigate('/profile')} aria-label="Back">
          ←
        </button>
        <h1 className="page-title">Edit Profile</h1>
      </header>

      <button
        type="button"
        className="ck-edit-profile__avatar"
        onClick={() => navigate('/inventory?category=avatar')}
      >
        <span style={{ fontSize: 40 }}>{typeof avatar === 'string' && avatar.length < 4 ? avatar : '👤'}</span>
        <span className="ck-edit-profile__avatar-hint">Change avatar</span>
      </button>

      <label className="ck-edit-profile__label" htmlFor="bio">Bio</label>
      <textarea
        id="bio"
        className="ck-edit-profile__bio"
        value={bio}
        maxLength={300}
        onChange={(e) => setBio(e.target.value)}
        placeholder="Tell others about you…"
      />
      <span className="ck-edit-profile__count">{bio.length}/300</span>

      <h2 className="section-heading">Change email</h2>
      {emailChangeSent ? (
        <p className="text-secondary">
          Check <strong style={{ color: 'var(--text-primary)' }}>{newEmail}</strong> for a link.
          Account still uses {user?.email} until confirmed.
        </p>
      ) : (
        <Input
          id="new-email"
          type="email"
          placeholder="New email"
          value={newEmail}
          onChange={(e) => setNewEmail(e.target.value)}
          autoComplete="email"
        />
      )}

      <h2 className="section-heading">Change password</h2>
      <Input
        id="current-password"
        type="password"
        placeholder="Current password (for email/password changes)"
        value={currentPassword}
        onChange={(e) => setCurrentPassword(e.target.value)}
        autoComplete="current-password"
      />
      <Input
        id="new-password"
        type="password"
        placeholder="New password"
        value={newPassword}
        onChange={(e) => setNewPassword(e.target.value)}
        autoComplete="new-password"
      />

      {error && <p className="ck-edit-profile__error">{error}</p>}

      <Button onClick={handleSave} loading={saving} loadingLabel="Saving…">
        Save
      </Button>

      <Toast visible={!!toast} message={toast?.message} onDismiss={() => setToast(null)} />
    </div>
  );
}
