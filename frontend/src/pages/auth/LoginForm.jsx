import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import { authApi } from '../../services/api';
import { useAuth } from '../../context/AuthContext';

const APPROVAL_POLL_INTERVAL_MS = 2500;

export default function LoginForm({ onSwitchTab }) {
  const navigate = useNavigate();
  const { setSession } = useAuth();

  const [identifier, setIdentifier] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState('');
  const [needsVerification, setNeedsVerification] = useState(false);
  const [resendState, setResendState] = useState('idle'); // idle | sending | sent

  // §5 device-conflict branching state
  const [pendingLoginId, setPendingLoginId] = useState(null);
  const [awaitingApproval, setAwaitingApproval] = useState(false);
  const [approvalDeniedMsg, setApprovalDeniedMsg] = useState('');
  const [code, setCode] = useState('');
  const [codeError, setCodeError] = useState('');

  // §5 Case C 3a/3b/3c: poll while waiting on the old device. Stops
  // itself on approve/deny/expire or if the user cancels/unmounts -
  // this is the piece the old "will update automatically" comment was
  // waiting on; nothing was actually polling before.
  useEffect(() => {
    if (!awaitingApproval || !pendingLoginId) return;

    let cancelled = false;
    const interval = setInterval(async () => {
      try {
        const { status } = await authApi.checkDeviceApprovalStatus(pendingLoginId);
        if (cancelled) return;
        if (status === 'approved') {
          setAwaitingApproval(false); // falls through to the code-entry screen below
        } else if (status === 'denied') {
          setApprovalDeniedMsg('Login request was denied from your other device.');
          setPendingLoginId(null);
          setAwaitingApproval(false);
        } else if (status === 'expired') {
          setApprovalDeniedMsg('That login request expired. Please try again.');
          setPendingLoginId(null);
          setAwaitingApproval(false);
        }
        // 'pending' → keep polling
      } catch {
        // transient network hiccup - next tick tries again rather than
        // dropping the user out of the flow over one failed poll
      }
    }, APPROVAL_POLL_INTERVAL_MS);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [awaitingApproval, pendingLoginId]);

  async function handleSubmit(e) {
    e.preventDefault();
    setFormError('');
    setNeedsVerification(false);
    setApprovalDeniedMsg('');
    setLoading(true);
    try {
      const result = await authApi.login(identifier, password);
      if (!result.requires_2fa) {
        // two_fa_enabled = 0 → straight to dashboard (§4.3)
        setSession(result);
        navigate('/dashboard');
      } else if (result.requires_device_approval) {
        // §5 Case C: old device must Approve/Deny first
        setPendingLoginId(result.pending_id);
        setAwaitingApproval(true);
      } else {
        // §5 Case A / Case B: prompt new device directly for the code
        setPendingLoginId(result.pending_id);
        setAwaitingApproval(false);
      }
    } catch (err) {
      if (err.code === 'email_not_verified') {
        setNeedsVerification(true);
      }
      setFormError(err.message);
    } finally {
      setLoading(false);
    }
  }

  async function handleResendVerification() {
    setResendState('sending');
    try {
      await authApi.resendVerification(identifier);
    } catch {
      // Same enumeration-safe pattern as forgot-password - don't reveal
      // whether it actually found an account either way.
    } finally {
      setResendState('sent');
    }
  }

  async function handleSubmitCode(e) {
    e.preventDefault();
    setCodeError('');
    setLoading(true);
    try {
      const tokens = await authApi.submit2faCode(pendingLoginId, code);
      setSession(tokens);
      navigate('/dashboard');
    } catch (err) {
      setCodeError(err.message);
    } finally {
      setLoading(false);
    }
  }

  // --- §5 Case C: waiting for the OLD device to Approve/Deny ---
  if (pendingLoginId && awaitingApproval) {
    return (
      <div className="ck-auth-pending-state">
        <p>A login request was sent to your other device.</p>
        <p className="text-secondary">Approve it there to continue — this checks automatically every few seconds.</p>
        <Button variant="outline" onClick={() => { setPendingLoginId(null); setAwaitingApproval(false); }}>
          Cancel
        </Button>
      </div>
    );
  }

  // --- §5 Case A/B, or Case C after approval: new device enters the code ---
  if (pendingLoginId) {
    return (
      <form className="ck-auth-form" onSubmit={handleSubmitCode}>
        <p className="text-secondary" style={{ textAlign: 'center', marginBottom: 'var(--space-4)' }}>
          Enter the 6-digit code for this account.
        </p>
        <Input
          id="twofa-code"
          type="text"
          inputMode="numeric"
          maxLength={6}
          value={code}
          onChange={(e) => setCode(e.target.value.replace(/\D/g, ''))}
          placeholder="000000"
          error={codeError}
          className="ck-auth-pending-code-input"
        />
        <Button type="submit" loading={loading} loadingLabel="Verifying..." pill>
          Verify
        </Button>
      </form>
    );
  }

  // --- Default: Login form ---
  return (
    <form className="ck-auth-form" onSubmit={handleSubmit}>
      {approvalDeniedMsg && (
        <p className="ck-input-error" role="alert" style={{ marginBottom: 'var(--space-3)' }}>
          {approvalDeniedMsg}
        </p>
      )}
      <Input
        id="login-identifier"
        icon="👤"
        placeholder="Username or Email"
        value={identifier}
        onChange={(e) => setIdentifier(e.target.value)}
        autoComplete="username"
      />
      <Input
        id="login-password"
        icon="🔒"
        type="password"
        placeholder="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        error={formError && !needsVerification ? formError : undefined}
        autoComplete="current-password"
      />

      {needsVerification && (
        <p className="ck-input-error" role="alert">
          {formError}{' '}
          {resendState === 'sent' ? (
            <span>Check your email for a new link (if that account exists and isn't verified yet).</span>
          ) : (
            <button
              type="button"
              className="ck-auth-secondary-link"
              style={{ display: 'inline' }}
              onClick={handleResendVerification}
              disabled={resendState === 'sending'}
            >
              {resendState === 'sending' ? 'Sending...' : 'Resend verification email'}
            </button>
          )}
        </p>
      )}

      <button type="button" className="ck-auth-secondary-link" onClick={() => onSwitchTab('forgot')}>
        Forgot password?
      </button>

      <div style={{ marginTop: 'var(--space-6)' }}>
        <Button type="submit" loading={loading} loadingLabel="Signing in..." pill>
          Log In
        </Button>
      </div>

      <p
        className="text-secondary"
        style={{ marginTop: 'var(--space-5)', textAlign: 'center', fontSize: '12px' }}
      >
        By logging in, you agree to our{' '}
        <button
          type="button"
          onClick={() => navigate('/settings/terms-of-service')}
          style={{ background: 'none', border: 'none', padding: 0, color: 'inherit', textDecoration: 'underline', cursor: 'pointer', font: 'inherit' }}
        >
          Terms of Service
        </button>{' '}
        and{' '}
        <button
          type="button"
          onClick={() => navigate('/settings/privacy-policy')}
          style={{ background: 'none', border: 'none', padding: 0, color: 'inherit', textDecoration: 'underline', cursor: 'pointer', font: 'inherit' }}
        >
          Privacy Policy
        </button>
        .
      </p>
    </form>
  );
}
