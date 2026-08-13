import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import './Splash.css';

const LANGUAGES = [
  { code: 'en', flag: '🇬🇧', name: 'English' },
  { code: 'ur', flag: '🇵🇰', name: 'اردو' },
];

export default function Splash() {
  const navigate = useNavigate();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const saved = localStorage.getItem('ck_language');
    if (saved) {
      navigate('/auth', { replace: true });
      return;
    }
    setVisible(true);
  }, [navigate]);

  function selectLanguage(code) {
    localStorage.setItem('ck_language', code);
    navigate('/auth', { replace: true });
  }

  if (!visible) return null;

  return (
    <div className="ck-splash">
      <div className="ck-splash__logo">♔</div>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-6)' }}>Chess King</h1>

      <div className="ck-splash__lang-list">
        {LANGUAGES.map((lang) => (
          <button key={lang.code} className="ck-splash__lang-row" onClick={() => selectLanguage(lang.code)}>
            <span aria-hidden="true">{lang.flag}</span>
            <span style={{ flex: 1 }}>{lang.name}</span>
            <span className="text-secondary" aria-hidden="true">›</span>
          </button>
        ))}
      </div>
    </div>
  );
}
