import { useState } from 'react';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import PasswordStrengthBar from './PasswordStrengthBar';
import { authApi } from '../../services/api';

export default function RegisterForm() {
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [errors, setErrors] = useState({});
  const [loading, setLoading] = useState(false);
  const [verifyState, setVerifyState] = useState(false); // §2.3 step 9: "verify your email" state

  async function handleSubmit(e) {
    e.preventDefault();
    setErrors({});

    // Doc 4 §2.2: "implicit confirm on frontend, not sent to backend twice"
    if (password !== confirmPassword) {
      setErrors({ confirmPassword: 'Passwords do not match.' });
      return;
    }

    setLoading(true);
    try {
      const resp = await authApi.register(username, email, password);
      if (resp.status === 'verify_email' || resp.status === 'verify_email_sent' || resp.status === 'verify_email_pending') {
        setVerifyState(true);
        if (resp.email_sent === false) {
          setErrors({ form: resp.message || 'Account created but email could not be sent. Try logging in.' });
        }
      }
    } catch (err) {
      // Doc 3 §2.4 exact error messages, mapped to the field they concern
      if (err.code === 'username_taken' || err.code === 'username_invalid' || err.code === 'username_reserved') {
        setErrors({ username: err.message });
      } else if (err.code === 'email_taken' || err.code === 'email_invalid') {
        setErrors({ email: err.message });
      } else if (err.code === 'password_weak') {
        setErrors({ password: err.message });
      } else {
        setErrors({ form: err.message });
      }
    } finally {
      setLoading(false);
    }
  }

  if (verifyState) {
    return (
      <div className="ck-auth-pending-state">
        <p>Account created.</p>
        <p className="text-secondary">
          We emailed a verification link to {email} (check spam).
          You can also switch to Login and sign in now.
        </p>
      </div>
    );
  }

  return (
    <form className="ck-auth-form" onSubmit={handleSubmit}>
      <Input
        id="reg-username"
        icon="👤"
        placeholder="Username"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        error={errors.username}
        autoComplete="username"
      />
      <Input
        id="reg-email"
        icon="✉️"
        type="email"
        placeholder="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        error={errors.email}
        autoComplete="email"
      />
      <Input
        id="reg-password"
        icon="🔒"
        type="password"
        placeholder="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        error={errors.password}
        autoComplete="new-password"
      />
      <PasswordStrengthBar password={password} />
      <Input
        id="reg-confirm-password"
        icon="🔒"
        type="password"
        placeholder="Confirm Password"
        value={confirmPassword}
        onChange={(e) => setConfirmPassword(e.target.value)}
        error={errors.confirmPassword}
        autoComplete="new-password"
      />

      {errors.form && <p className="ck-input-error" role="alert">{errors.form}</p>}

      <div style={{ marginTop: 'var(--space-4)' }}>
        <Button type="submit" loading={loading} loadingLabel="Creating account..." pill>
          Create Account
        </Button>
      </div>
    </form>
  );
}
