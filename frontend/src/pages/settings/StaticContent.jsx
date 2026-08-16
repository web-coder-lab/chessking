import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

export default function StaticContent({ title, fetchFn }) {
  const navigate = useNavigate();
  const [content, setContent] = useState(null);
  const [error, setError] = useState('');

  useEffect(() => {
    let cancelled = false;
    fetchFn()
      .then((d) => {
        if (!cancelled) setContent(d.content ?? d.body ?? '');
      })
      .catch((e) => {
        if (!cancelled) setError(e.message || 'Could not load.');
      });
    return () => {
      cancelled = true;
    };
  }, [fetchFn]);

  return (
    <div className="ck-page-shell" style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <button
        type="button"
        onClick={() => navigate(-1)}
        style={{
          background: 'none',
          border: 'none',
          color: 'var(--accent-gold)',
          fontSize: 14,
          marginBottom: 12,
          cursor: 'pointer',
          padding: 0,
        }}
      >
        ← Back
      </button>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>{title}</h1>
      {error && <p style={{ color: 'var(--danger-red)' }}>{error}</p>}
      {content == null && !error && <p className="text-secondary">Loading...</p>}
      {content === '' && <p className="text-secondary">Nothing here yet.</p>}
      {content && <p style={{ whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>{content}</p>}
    </div>
  );
}
