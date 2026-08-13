import './EmptyState.css';

/**
 * Doc 4 §3 Empty state: "centered icon + muted text" — e.g. Wallet §2.4
 * "No transactions yet", Profile §2.10 "No gifts received yet".
 */
export default function EmptyState({ icon, text }) {
  return (
    <div className="ck-empty-state">
      <div className="ck-empty-icon" aria-hidden="true">{icon}</div>
      <p className="ck-empty-text">{text}</p>
    </div>
  );
}
