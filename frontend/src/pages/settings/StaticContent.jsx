import { useEffect, useState } from 'react';

export default function StaticContent({ title, fetchFn }) {
  const [content, setContent] = useState(null);

  useEffect(() => {
    fetchFn().then((d) => setContent(d.content));
  }, [fetchFn]);

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>{title}</h1>
      {content == null && <p className="text-secondary">Loading...</p>}
      {content === '' && <p className="text-secondary">Nothing here yet.</p>}
      {content && <p style={{ whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>{content}</p>}
    </div>
  );
}
