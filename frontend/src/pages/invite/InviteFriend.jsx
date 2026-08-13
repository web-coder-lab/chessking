import { useEffect, useState } from 'react';
import Button from '../../components/common/Button';
import { socialApi } from '../../services/api';
import './Invite.css';

export default function InviteFriend() {
  const [link, setLink] = useState(null);
  const [progress, setProgress] = useState([]);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    socialApi.getReferralLink().then(setLink);
    socialApi.getReferralProgress().then((d) => setProgress(d.referrals));
  }, []);

  function handleCopy() {
    if (link) {
      navigator.clipboard.writeText(link.share_url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  function handleShare() {
    if (navigator.share && link) {
      navigator.share({ title: 'Join Chess King', url: link.share_url });
    }
  }

  async function handleClaim(referralId) {
    await socialApi.claimReferral(referralId);
    socialApi.getReferralProgress().then((d) => setProgress(d.referrals));
  }

  return (
    <div style={{ padding: 'var(--space-6) var(--screen-padding-x)' }}>
      <h1 className="page-title" style={{ marginBottom: 'var(--space-4)' }}>Invite a Friend</h1>

      <div className="ck-invite__link-box">
        <span className="ck-invite__link-text">{link?.share_url || 'Loading...'}</span>
        <button className="ck-invite__copy-btn" onClick={handleCopy}>{copied ? 'Copied!' : 'Copy'}</button>
      </div>

      <Button variant="outline" onClick={handleShare} style={{ marginTop: 'var(--space-3)' }}>Share</Button>

      <h2 className="section-heading" style={{ margin: 'var(--space-6) 0 var(--space-3)' }}>Your Invites</h2>
      {progress.map((r) => (
        <div key={r.referral_id} className="ck-invite__progress-row">
          <span style={{ flex: 1 }}>{r.username}</span>
          <span className="text-secondary tabular-nums">{r.spent}/{r.target} coins</span>
          {r.claimable ? (
            <button className="ck-invite__claim-btn" onClick={() => handleClaim(r.referral_id)}>Claim</button>
          ) : (
            <span className="ck-invite__pending-badge">In progress</span>
          )}
        </div>
      ))}
    </div>
  );
}
