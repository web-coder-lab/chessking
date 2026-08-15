import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { authApi } from '../../services/api';
import { useAuth } from '../../context/AuthContext';
import './Auth.css';

export default function VerifyEmail() {
  const [params] = useSearchParams();
  const token = useMemo(() => params.get('token') || '', [params]);
  const navigate = useNavigate();
  const { setSession } = useAuth();
  const [status, setStatus] = useState('working'); // working | ok | fail
  const [message, setMessage] = useState('Verifying your email…');

  useEffect(() => {
    if (!token) {
      setStatus('fail');
      setMessage('Missing verification token.');
      return;
    }
    let cancelled = false;
    authApi
      .verifyEmail(token)
      .then((data) => {
        if (cancelled) return;
        if (data.access_token && data.refresh_token) {
          setSession(data);
          setStatus('ok');
          setMessage('Email verified. Taking you in…');
          setTimeout(() => navigate('/dashboard', { replace: true }), 800);
        } else {
          setStatus('ok');
          setMessage('Email verified. You can sign in now.');
        }
      })
      .catch((e) => {
        if (cancelled) return;
        setStatus('fail');
        setMessage(e.message || 'Verification failed or link expired.');
      });
    return () => {
      cancelled = true;
    };
  }, [token, setSession, navigate]);

  return (
    <div className="ck-auth-screen">
      <p className="ck-auth-brand">♚ Genius Clan</p>
      <h1 className="page-title" style={{ textAlign: 'center' }}>
        {status === 'working' ? 'Verifying…' : status === 'ok' ? 'Verified' : 'Could not verify'}
      </h1>
      <p className="text-secondary" style={{ textAlign: 'center', marginTop: 12 }}>
        {message}
      </p>
      {status === 'fail' && (
        <button
          type="button"
          className="ck-btn-primary"
          style={{ marginTop: 24, width: '100%' }}
          onClick={() => navigate('/auth')}
        >
          Back to sign in
        </button>
      )}
    </div>
  );
}
