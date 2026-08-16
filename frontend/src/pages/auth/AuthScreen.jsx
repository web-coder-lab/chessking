import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import LoginForm from './LoginForm';
import RegisterForm from './RegisterForm';
import ForgotForm from './ForgotForm';
import { useAuth } from '../../context/AuthContext';
import './Auth.css';

const TABS = [
  { key: 'login', label: 'Login' },
  { key: 'register', label: 'Register' },
  { key: 'forgot', label: 'Forgot' },
];

export default function AuthScreen() {
  const [tab, setTab] = useState('login');
  const { isAuthenticated, bootstrapping } = useAuth();
  const navigate = useNavigate();

  // Part 23: already logged-in users shouldn't sit on auth after refresh
  useEffect(() => {
    if (!bootstrapping && isAuthenticated) {
      navigate('/dashboard', { replace: true });
    }
  }, [bootstrapping, isAuthenticated, navigate]);

  if (bootstrapping) {
    return (
      <div className="ck-auth-screen" style={{ justifyContent: 'center', alignItems: 'center' }}>
        <p className="ck-auth-brand">♚ Genius Clan</p>
        <p className="text-secondary">Restoring session…</p>
      </div>
    );
  }

  return (
    <div className="ck-auth-screen">
      <p className="ck-auth-brand">Genius Clan</p>
      <div className="ck-auth-illustration" aria-hidden="true">♔</div>

      <div className="ck-auth-tabs" role="tablist">
        {TABS.map((t) => (
          <button
            key={t.key}
            role="tab"
            aria-selected={tab === t.key}
            className={`ck-auth-tab ${tab === t.key ? 'ck-auth-tab--active' : ''}`}
            onClick={() => setTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === 'login' && <LoginForm onSwitchTab={setTab} />}
      {tab === 'register' && <RegisterForm />}
      {tab === 'forgot' && <ForgotForm />}
    </div>
  );
}
