import { useState } from 'react';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import { authApi } from '../../services/api';

export default function ForgotForm() {
  const [email, setEmail] = useState('');
  const [loading, setLoading] = useState(false);
  const [sent, setSent] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setLoading(true);
    try {
      await authApi.forgotPassword(email);
    } finally {
      setLoading(false);
      // §6 step 3: always show the same generic success message, even if
      // the request technically failed — never leak whether the email
      // exists via a different UI outcome.
      setSent(true);
    }
  }

  if (sent) {
    return (
      <div className="ck-auth-pending-state">
        <p>If this email is registered, a reset link has been sent.</p>
      </div>
    );
  }

  return (
    <form className="ck-auth-form" onSubmit={handleSubmit}>
      <Input
        id="forgot-email"
        icon="✉️"
        type="email"
        placeholder="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        autoComplete="email"
      />
      <div style={{ marginTop: 'var(--space-4)' }}>
        <Button type="submit" loading={loading} loadingLabel="Sending..." pill>
          Send Reset Link
        </Button>
      </div>
    </form>
  );
}
