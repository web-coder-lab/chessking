import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import { socialApi } from '../../services/api';
import './Profile.css';

export default function ProfileSettings({ user }) {
  const navigate = useNavigate();
  const [bio, setBio] = useState(user?.bio || '');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [newEmail, setNewEmail] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [emailChangeSent, setEmailChangeSent] = useState(false);

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
      }
      if (!(currentPassword && newEmail)) {
        navigate('/profile');
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="ck-profile" style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-6)' }}>Edit Profile</h1>

      <button className="ck-profile__avatar" style={{ margin: '0 0 var(--space-4)' }} onClick={() => navigate('/inventory?category=avatar')}>
        <img src="/assets/default-avatar.png" alt="" style={{ width: '100%', height: '100%', borderRadius: '50%' }} />
      </button>

      <div className="ck-input-group">
        <label className="ck-input-label" htmlFor="bio">Bio</label>
        <textarea
          id="bio"
          className="ck-input"
          style={{ width: '100%', minHeight: 80, background: 'var(--bg-surface)', border: '1.5px solid var(--border-subtle)', borderRadius: 'var(--radius-button)', padding: 'var(--space-3)' }}
          value={bio}
          maxLength={300}
          onChange={(e) => setBio(e.target.value)}
        />
        <span className="text-secondary" style={{ fontSize: 'var(--text-caption-size)' }}>{bio.length}/300</span>
      </div>

      <h2 className="section-heading" style={{ margin: 'var(--space-4) 0' }}>Change Email</h2>
      {emailChangeSent ? (
        <p className="text-secondary">
          Check <strong style={{ color: 'var(--text-primary)' }}>{newEmail}</strong> for a confirmation link. Your account still uses {user?.email} until you click it.
        </p>
      ) : (
        <Input id="new-email" type="email" placeholder="New Email" value={newEmail} onChange={(e) => setNewEmail(e.target.value)} autoComplete="email" />
      )}

      <h2 className="section-heading" style={{ margin: 'var(--space-4) 0' }}>Change Password</h2>
      <Input id="current-password" type="password" placeholder="Current Password (needed for email or password changes)" value={currentPassword} onChange={(e) => setCurrentPassword(e.target.value)} autoComplete="current-password" />
      <Input id="new-password" type="password" placeholder="New Password" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} error={error} autoComplete="new-password" />

      <Button onClick={handleSave} loading={saving} loadingLabel="Saving...">Save</Button>
    </div>
  );
}
