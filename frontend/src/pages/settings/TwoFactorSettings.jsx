import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { authApi } from '../../services/api';
import './TwoFactorSettings.css';

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
    if (!/^\d{6}$/.test(code)) {
      setError('Enter a 6-digit code');
      return;
    }
    setSubmitting(true);
    try {
      await authApi.enable2FA(password, code, confirmCode);
      await refreshUser?.();
      setToast({ message: 'Two-step verification is on' });
      setTimeout(() => navigate('/settings'), 900);
    } catch (err) {
      setError(err.message || 'Could not enable 2FA');
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
      setTimeout(() => navigate('/settings'), 900);
    } catch (err) {
      setError(err.message || 'Could not disable 2FA');
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="ck-2fa">
      <header className="ck-2fa__header">
        <button type="button" className="ck-2fa__back" onClick={() => navigate('/settings')} aria-label="Back">
          ←
        </button>
        <h1 className="page-title">Two-Step Verification</h1>
      </header>

      <div className={`ck-2fa__badge ${enabled ? 'ck-2fa__badge--on' : ''}`}>
        {enabled ? 'Enabled' : 'Disabled'}
      </div>

      <p className="ck-2fa__desc">
        {enabled
          ? 'Turn off by entering your password and your 6-digit code.'
          : 'Create a 6-digit code. You will need it when signing in on a new device.'}
      </p>

      <form className="ck-2fa__form" onSubmit={enabled ? handleDisable : handleEnable}>
        <Input
          id="2fa-password"
          type="password"
          label="Current password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete="current-password"
          required
        />
        <Input
          id="2fa-code"
          type="text"
          inputMode="numeric"
          label={enabled ? 'Your 6-digit code' : 'Choose a 6-digit code'}
          value={code}
          onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
          maxLength={6}
          required
        />
        {!enabled && (
          <Input
            id="2fa-confirm"
            type="text"
            inputMode="numeric"
            label="Confirm code"
            value={confirmCode}
            onChange={(e) => setConfirmCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
            maxLength={6}
            required
          />
        )}
        {error && <p className="ck-2fa__error">{error}</p>}
        <Button type="submit" loading={submitting} loadingLabel="Please wait…">
          {enabled ? 'Turn off' : 'Turn on'}
        </Button>
      </form>

      <Toast visible={!!toast} message={toast?.message} onDismiss={() => setToast(null)} />
    </div>
  );
}
