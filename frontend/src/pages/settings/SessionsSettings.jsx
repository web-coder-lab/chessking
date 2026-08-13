import { useEffect, useState } from 'react';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import { authApi } from '../../services/api';

export default function SessionsSettings() {
  const [sessions, setSessions] = useState(null);
  const [revoking, setRevoking] = useState(null);

  function load() {
    authApi.getSessions().then((d) => setSessions(d.sessions));
  }

  useEffect(() => { load(); }, []);

  async function handleRevoke(id) {
    setRevoking(id);
    try {
      await authApi.revokeSession(id);
      load();
    } finally {
      setRevoking(null);
    }
  }

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-2)' }}>Active Sessions</h1>
      <p className="text-secondary" style={{ marginBottom: 'var(--space-6)' }}>
        Devices currently signed in to your account.
      </p>

      {sessions == null && <p className="text-secondary">Loading...</p>}
      {sessions?.length === 0 && <p className="text-secondary">No active sessions.</p>}

      {sessions?.map((s) => (
        <Card key={s.id} style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-3)', marginBottom: 'var(--space-3)' }}>
          <span aria-hidden="true" style={{ fontSize: 20 }}>💻</span>
          <div style={{ flex: 1 }}>
            <p>{s.browser || 'Unknown browser'} · {s.os || 'Unknown OS'}</p>
            <p className="text-secondary" style={{ fontSize: 'var(--text-caption-size)' }}>
              {s.is_active ? 'Active' : 'Signed out'} · last seen {s.last_seen_at ? new Date(s.last_seen_at).toLocaleString() : '—'}
            </p>
          </div>
          {!!s.is_active && (
            <Button
              variant="outline"
              fullWidth={false}
              loading={revoking === s.id}
              onClick={() => handleRevoke(s.id)}
            >
              Revoke
            </Button>
          )}
        </Card>
      ))}
    </div>
  );
}
