import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { gameApi } from '../../services/api';
import './CustomMatch.css';

const POLL_INTERVAL_MS = 3000;

export default function CustomMatch({ user }) {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState([]);
  const [history, setHistory] = useState([]);
  const [waitingFor, setWaitingFor] = useState(null);
  const [toast, setToast] = useState(null);
  const pollTimer = useRef(null);

  function loadHistory() {
    gameApi.getInviteHistory().then((d) => setHistory(d.invites));
  }

  useEffect(() => { loadHistory(); }, []);

  useEffect(() => {
    if (query.length < 2) { setResults([]); return; }
    const t = setTimeout(() => {
      gameApi.searchCustomMatch(query).then((d) => setResults(d.results));
    }, 300);
    return () => clearTimeout(t);
  }, [query]);

  // Doc 7 §6 step 6a/6b's real-time notify-the-receiver path isn't wired
  // up on the backend yet, so there's no push to react to here either -
  // poll invite history instead while on the Waiting screen, so accept/
  // decline is at least observable instead of hanging forever.
  useEffect(() => {
    if (!waitingFor || waitingFor.status !== 'waiting') return;
    let cancelled = false;

    async function tick() {
      const d = await gameApi.getInviteHistory();
      if (cancelled) return;
      setHistory(d.invites);
      const mine = d.invites.find((h) => h.id === waitingFor.inviteId);
      if (mine?.status === 'accepted' && mine.match_id) {
        navigate(`/board/${mine.match_id}`);
      } else if (mine?.status === 'declined') {
        setWaitingFor((w) => (w ? { ...w, status: 'declined' } : w));
      } else if (!cancelled) {
        pollTimer.current = setTimeout(tick, POLL_INTERVAL_MS);
      }
    }
    pollTimer.current = setTimeout(tick, POLL_INTERVAL_MS);

    return () => { cancelled = true; clearTimeout(pollTimer.current); };
  }, [waitingFor?.inviteId, waitingFor?.status, navigate]);

  async function handleInvite(user) {
    const resp = await gameApi.sendCustomMatchInvite(user.username);
    setWaitingFor({ inviteId: resp.invite_id, username: user.username, status: 'waiting' });
  }

  function cancelWaiting() {
    setWaitingFor(null);
  }

  async function handleRespond(inviteId, decision) {
    try {
      const resp = await gameApi.respondToInvite(inviteId, decision);
      if (decision === 'accept' && resp.match_id) {
        navigate(`/board/${resp.match_id}`);
      } else {
        loadHistory();
      }
    } catch (e) {
      setToast({ message: e.message || 'Could not respond to invite' });
    }
  }

  if (waitingFor) {
    return (
      <div className="ck-play-fullscreen">
        {waitingFor.status === 'declined' ? (
          <>
            <p className="page-title">This player did not accept your request.</p>
            <Button variant="outline" onClick={cancelWaiting} style={{ marginTop: 'var(--space-6)' }}>Back</Button>
          </>
        ) : (
          <>
            <span className="ck-custom-match__pulse-dot" aria-hidden="true" />
            <p className="page-title" style={{ marginTop: 'var(--space-6)' }}>Waiting for {waitingFor.username}...</p>
            <Button variant="outline" onClick={cancelWaiting} style={{ marginTop: 'var(--space-8)' }}>Cancel</Button>
          </>
        )}
      </div>
    );
  }

  const incoming = user ? history.filter((h) => h.receiver_id === user.id && h.status === 'pending') : [];

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>Custom Match</h1>

      {incoming.length > 0 && (
        <div style={{ marginBottom: 'var(--space-6)' }}>
          <h2 className="section-heading" style={{ marginBottom: 'var(--space-2)' }}>Incoming Invites</h2>
          {incoming.map((inv) => (
            <Card key={inv.id} className="ck-custom-match__result-row">
              <span style={{ flex: 1 }}>Match request</span>
              <Button fullWidth={false} onClick={() => handleRespond(inv.id, 'accept')}>Accept</Button>
              <Button fullWidth={false} variant="outline" onClick={() => handleRespond(inv.id, 'decline')}>Decline</Button>
            </Card>
          ))}
        </div>
      )}

      <input
        className="ck-input"
        style={{ width: '100%', height: 48, background: 'var(--bg-surface)', border: '1.5px solid var(--border-subtle)', borderRadius: 'var(--radius-button)', padding: '0 var(--space-4)' }}
        placeholder="🔍 Search username..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />

      <div style={{ marginTop: 'var(--space-3)' }}>
        {results.map((r) => (
          <Card key={r.id} className="ck-custom-match__result-row">
            <img src="/assets/default-avatar.svg" alt="" className="ck-custom-match__avatar" />
            <span style={{ flex: 1 }}>{r.username}</span>
            <Button fullWidth={false} onClick={() => handleInvite(r)}>Invite</Button>
          </Card>
        ))}
      </div>

      {history.length > 0 && (
        <>
          <h2 className="section-heading" style={{ margin: 'var(--space-6) 0 var(--space-2)' }}>Recent</h2>
          {history.slice(0, 5).map((h) => (
            <div key={h.id} className="ck-custom-match__history-row text-secondary">
              {h.status} — {new Date(h.created_at).toLocaleDateString()}
            </div>
          ))}
        </>
      )}

      <Toast visible={!!toast} message={toast?.message} onDismiss={() => setToast(null)} />
    </div>
  );
}
