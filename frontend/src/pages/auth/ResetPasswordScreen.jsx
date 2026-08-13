import { useState } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import PasswordStrengthBar from './PasswordStrengthBar';
import { authApi } from '../../services/api';
import './Auth.css';

/**
 * Doc 4 §2.2: "Reset Password screen (reached only via emailed link, not
 * directly navigable from the tab bar): New Password, Confirm Password,
 * live strength indicator bar."
 */
export default function ResetPasswordScreen() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const token = searchParams.get('token');

  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');
    if (newPassword !== confirmPassword) {
      setError('Passwords do not match.');
      return;
    }
    setLoading(true);
    try {
      await authApi.resetPassword(token, newPassword);
      setDone(true);
      // §6 step 9: redirect to Login, force fresh login with new password
      setTimeout(() => navigate('/auth'), 2000);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  if (done) {
    return (
      <div className="ck-auth-screen">
        <div className="ck-auth-pending-state">
          <p>Password updated.</p>
          <p className="text-secondary">You've been logged out everywhere for security. Redirecting to login...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="ck-auth-screen">
      <div className="ck-auth-illustration" aria-hidden="true">♔</div>
      <h1 className="page-title" style={{ textAlign: 'center', marginBottom: 'var(--space-6)' }}>
        Reset Password
      </h1>

      <form className="ck-auth-form" onSubmit={handleSubmit}>
        <Input
          id="reset-new-password"
          icon="🔒"
          type="password"
          placeholder="New Password"
          value={newPassword}
          onChange={(e) => setNewPassword(e.target.value)}
          autoComplete="new-password"
        />
        <PasswordStrengthBar password={newPassword} />

        <Input
          id="reset-confirm-password"
          icon="🔒"
          type="password"
          placeholder="Confirm Password"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          error={error}
          autoComplete="new-password"
        />

        <div style={{ marginTop: 'var(--space-4)' }}>
          <Button type="submit" loading={loading} loadingLabel="Updating..." pill>
            Update Password
          </Button>
        </div>
      </form>
    </div>
  );
}
