import { useEffect, useState } from 'react';
import { supportApi } from '../../services/api';

export default function SupportPage() {
  const [email, setEmail] = useState(null);

  useEffect(() => {
    supportApi.getSupportInfo().then((d) => setEmail(d.email));
  }, []);

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>Support Team</h1>
      <p className="text-secondary" style={{ marginBottom: 'var(--space-4)' }}>
        Need help? Reach out and we'll get back to you.
      </p>
      {email && (
        <a href={`mailto:${email}`} className="ck-button ck-button--primary" style={{ display: 'inline-block', textDecoration: 'none' }}>
          {email}
        </a>
      )}
      {email === '' && <p className="text-secondary">Support contact isn't set up yet.</p>}
    </div>
  );
}
