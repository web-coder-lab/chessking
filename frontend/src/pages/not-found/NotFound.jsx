import { useNavigate } from 'react-router-dom';
import './NotFound.css';

/**
 * Branded 404 — unknown frontend paths only.
 * Legitimate app routes stay registered in App.jsx.
 */
export default function NotFound() {
  const navigate = useNavigate();
  return (
    <div className="ck-404">
      <div className="ck-404__card">
        <div className="ck-404__crown" aria-hidden="true">♚</div>
        <p className="ck-404__brand">Genius Clan</p>
        <h1 className="ck-404__code">404</h1>
        <p className="ck-404__title">Page not found</p>
        <p className="ck-404__body">
          This path doesn&apos;t exist on Genius Clan. Check the link or return home.
        </p>
        <div className="ck-404__actions">
          <button type="button" className="ck-404__btn ck-404__btn--primary" onClick={() => navigate('/dashboard')}>
            Go to Home
          </button>
          <button type="button" className="ck-404__btn" onClick={() => navigate('/auth')}>
            Sign in
          </button>
        </div>
      </div>
    </div>
  );
}
