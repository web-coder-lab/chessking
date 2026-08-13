import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { authApi } from '../../services/api';

export default function TwoFactorSettings({ user, refreshUser }) {
  const navigate = useNavigate();
  const enabled = !!user?.two_fa_enabled;

  const [password, setPassword] = useState('');
  const [code, setCode] = useState('');
  const [confirmCode, setConfirmCode] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [toast, setToast] = useState(null);

  async function handleEnable(e) {
    e.preventDefault();
    setError('');
    if (code !== confirmCode) {
      setError('Codes do not match');
      return;
    }
    if (code.length !== 6 || !/^\d{6}$/.test(code)) {
      setError('Enter a 6-digit code');
      return;
    }
    setSubmitting(true);
    try {
      await authApi.enable2FA(password, code, confirmCode);
      await refreshUser?.();
      setToast({ message: 'Two-step verification is on' });
      setTimeout(() => navigate('/settings'), 1000);
    } catch (e) {
      setError(e.message || 'Could not enable 2FA');
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDisable(e) {
    e.preventDefault();
    setError('');
    setSubmitting(true);
    try {
      await authApi.disable2FA(password, code);
      await refreshUser?.();
      setToast({ message: 'Two-step verification is off' });
      setTimeout(() => navigate('/settings'), 1000);
    } catch (e) {
      setError(e.message || 'Could not disable 2FA');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-2)' }}>Two-Step Verification</h1>
      <p className="text-secondary" style={{ marginBottom: 'var(--space-6)' }}>
        {enabled
          ? 'Enter your password and current code to turn this off.'
          : 'Choose a 6-digit code. You\'ll enter it (alongside your password) whenever you log in from a new device.'}
      </p>

      <form onSubmit={enabled ? handleDisable : handleEnable}>
        <Input
          id="current-password"
          type="password"
          label="Current Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <Input
          id="code"
          type="text"
          inputMode="numeric"
          label={enabled ? 'Current 6-Digit Code' : 'New 6-Digit Code'}
          value={code}
          onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
        />
        {!enabled && (
          <Input
            id="confirm-code"
            type="text"
            inputMode="numeric"
            label="Confirm Code"
            value={confirmCode}
            onChange={(e) => setConfirmCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
            error={error}
          />
        )}
        {enabled && error && <p style={{ color: 'var(--danger-red)', fontSize: 'var(--text-caption-size)' }}>{error}</p>}

        <Button type="submit" variant={enabled ? 'destructive' : 'primary'} loading={submitting} loadingLabel="Saving...">
          {enabled ? 'Turn Off' : 'Turn On'}
        </Button>
      </form>

      <Toast visible={!!toast} message={toast?.message} onDismiss={() => setToast(null)} />
    </div>
  );
}
