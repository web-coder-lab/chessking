import { useMemo, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import Input from '../../components/common/Input';
import Button from '../../components/common/Button';
import PasswordStrengthBar from './PasswordStrengthBar';
import { authApi } from '../../services/api';
import { useAuth } from '../../context/AuthContext';
import './Auth.css';

export default function CompleteSignup() {
  const [params] = useSearchParams();
  const token = useMemo(() => params.get('token') || '', [params]);
  const navigate = useNavigate();
  const { setSession } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [errors, setErrors] = useState({});
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setErrors({});
    if (!token) {
      setErrors({ form: 'Invalid or missing signup link.' });
      return;
    }
    if (password !== confirm) {
      setErrors({ confirm: 'Passwords do not match.' });
      return;
    }
    setLoading(true);
    try {
      const tokens = await authApi.completeSignup(token, username, password);
      setSession(tokens);
      navigate('/dashboard', { replace: true });
    } catch (err) {
      if (err.code === 'username_taken' || err.code === 'username_invalid' || err.code === 'username_reserved') {
        setErrors({ username: err.message });
      } else if (err.code === 'password_weak') {
        setErrors({ password: err.message });
      } else if (err.status === 401 || err.code === 'unauthorized') {
        setErrors({ form: 'This link expired or was already used. Request a new signup email.' });
      } else {
        setErrors({ form: err.message || 'Something went wrong.' });
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="ck-auth-screen">
      <div style={{ maxWidth: 400, margin: '0 auto', width: '100%' }}>
        <p className="ck-auth-brand">♚ Genius Clan</p>
        <h1 className="page-title" style={{ textAlign: 'center', marginBottom: 8 }}>Complete signup</h1>
        <p className="text-secondary" style={{ textAlign: 'center', marginBottom: 24 }}>
          Choose a username and password to finish.
        </p>
        <form onSubmit={handleSubmit}>
          <Input label="Username" value={username} onChange={(e) => setUsername(e.target.value)} error={errors.username} autoComplete="username" />
          <Input label="Password" type="password" value={password} onChange={(e) => setPassword(e.target.value)} error={errors.password} autoComplete="new-password" />
          <PasswordStrengthBar password={password} />
          <Input label="Confirm password" type="password" value={confirm} onChange={(e) => setConfirm(e.target.value)} error={errors.confirm} autoComplete="new-password" />
          {errors.form && <p style={{ color: '#F87171', fontSize: 14 }}>{errors.form}</p>}
          <Button type="submit" fullWidth disabled={loading} style={{ marginTop: 16 }}>
            {loading ? 'Creating…' : 'Create account'}
          </Button>
        </form>
      </div>
    </div>
  );
}
