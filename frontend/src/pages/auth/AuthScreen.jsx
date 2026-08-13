import { useState } from 'react';
import LoginForm from './LoginForm';
import RegisterForm from './RegisterForm';
import ForgotForm from './ForgotForm';
import './Auth.css';

/**
 * Doc 4 §2.2: chess piece illustration top, three-tab switcher
 * (Login/Register/Forgot), active tab underlined in gold.
 * Reset Password is a SEPARATE screen, reached only via emailed link —
 * not part of this tab switcher (see ResetPasswordScreen.jsx).
 */
const TABS = [
  { key: 'login', label: 'Login' },
  { key: 'register', label: 'Register' },
  { key: 'forgot', label: 'Forgot' },
];

export default function AuthScreen() {
  const [tab, setTab] = useState('login');

  return (
    <div className="ck-auth-screen">
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
