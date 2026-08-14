import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { supportApi } from '../../services/api';
import './SupportPage.css';

const DEFAULT_EMAIL = 'workn8312@gmail.com';

export default function SupportPage() {
  const navigate = useNavigate();
  const [email, setEmail] = useState(DEFAULT_EMAIL);

  useEffect(() => {
    supportApi
      .getSupportInfo()
      .then((d) => {
        if (d?.email) setEmail(d.email);
      })
      .catch(() => {});
  }, []);

  return (
    <div className="ck-support">
      <header className="ck-support__header">
        <button type="button" className="ck-support__back" onClick={() => navigate(-1)} aria-label="Back">
          ←
        </button>
        <h1 className="ck-support__title">Support Team</h1>
      </header>

      <div className="ck-support__hero">
        <div className="ck-support__icon" aria-hidden>
          ♟️
        </div>
        <p className="ck-support__brand">Genius Clan</p>
        <p className="ck-support__lead">
          Questions, bugs, or account help — we&apos;re here for you.
        </p>
      </div>

      <div className="ck-support__card">
        <div className="ck-support__row">
          <span className="ck-support__label">Email</span>
          <a className="ck-support__email" href={`mailto:${email}`}>
            {email}
          </a>
        </div>
        <p className="ck-support__hint">Usually reply within 24–48 hours.</p>
        <a className="ck-support__cta" href={`mailto:${email}?subject=Genius%20Clan%20Support`}>
          Send email
        </a>
      </div>

      <div className="ck-support__tips">
        <h2>Quick tips</h2>
        <ul>
          <li>Include your username in the email</li>
          <li>Describe the issue and when it happened</li>
          <li>Screenshots help us fix faster</li>
        </ul>
      </div>
    </div>
  );
}
