import './ConfirmDialog.css';

/**
 * Genius Clan themed confirm modal.
 * cancelLabel defaults to Cancel, confirmLabel to OK.
 */
export default function ConfirmDialog({
  open,
  title = 'Leave this match?',
  message = 'If you leave now, the game may count as a disconnect or loss. Are you sure you want to go back?',
  confirmLabel = 'OK',
  cancelLabel = 'Cancel',
  danger = true,
  onConfirm,
  onCancel,
}) {
  if (!open) return null;

  return (
    <div className="ck-confirm" role="dialog" aria-modal="true" aria-labelledby="ck-confirm-title">
      <button type="button" className="ck-confirm__backdrop" aria-label="Dismiss" onClick={onCancel} />
      <div className="ck-confirm__card">
        <div className="ck-confirm__icon" aria-hidden>
          ♚
        </div>
        <p className="ck-confirm__brand">Genius Clan</p>
        <h2 id="ck-confirm-title" className="ck-confirm__title">
          {title}
        </h2>
        <p className="ck-confirm__body">{message}</p>
        <div className="ck-confirm__actions">
          <button type="button" className="ck-confirm__btn ck-confirm__btn--ghost" onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            type="button"
            className={`ck-confirm__btn ${danger ? 'ck-confirm__btn--danger' : 'ck-confirm__btn--primary'}`}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
