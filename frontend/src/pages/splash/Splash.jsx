import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';
import './Splash.css';

const LANGUAGES = [
  { code: 'en', flag: '🇬🇧', name: 'English' },
  { code: 'ur', flag: '🇵🇰', name: 'اردو' },
];

export default function Splash() {
  const navigate = useNavigate();
  const { isAuthenticated, bootstrapping } = useAuth();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (bootstrapping) return;
    const saved = localStorage.getItem('ck_language');
    if (isAuthenticated) {
      navigate('/dashboard', { replace: true });
      return;
    }
    if (saved) {
      navigate('/auth', { replace: true });
      return;
    }
    setVisible(true);
  }, [navigate, isAuthenticated, bootstrapping]);

  function selectLanguage(code) {
    localStorage.setItem('ck_language', code);
    navigate('/auth', { replace: true });
  }

  if (bootstrapping || !visible) {
    return (
      <div className="ck-splash">
        <div className="ck-splash__logo">♚</div>
        <p className="ck-splash__tag">Genius Clan</p>
        <p className="text-secondary">Loading…</p>
      </div>
    );
  }

  return (
    <div className="ck-splash">
      <div className="ck-splash__logo">♚</div>
      <p className="ck-splash__tag">Multiplayer Chess</p>
      <h1 className="ck-splash__title">Genius Clan</h1>
      <div className="ck-splash__lang-list">
        {LANGUAGES.map((lang) => (
          <button key={lang.code} type="button" className="ck-splash__lang-row" onClick={() => selectLanguage(lang.code)}>
            <span aria-hidden="true">{lang.flag}</span>
            <span style={{ flex: 1 }}>{lang.name}</span>
            <span className="text-secondary" aria-hidden="true">›</span>
          </button>
        ))}
      </div>
    </div>
  );
}
