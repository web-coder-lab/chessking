import { useEffect } from 'react';
import './Toast.css';

/**
 * Doc 4 §1.4: "Toast/snackbar: slide up from bottom, auto-dismiss after 3s."
 * Used e.g. Shop §2.5 "Not enough coins" + shortcut to Wallet,
 * Bug Report §2.13 "Your report has been submitted."
 */
export default function Toast({ message, actionLabel, onAction, onDismiss, visible }) {
  useEffect(() => {
    if (!visible) return;
    const t = setTimeout(onDismiss, 3000); // §1.4: auto-dismiss after 3s
    return () => clearTimeout(t);
  }, [visible, onDismiss]);

  if (!visible) return null;

  return (
    <div className="ck-toast" role="status">
      <span className="ck-toast-message">{message}</span>
      {actionLabel && (
        <button className="ck-toast-action" onClick={onAction}>
          {actionLabel}
        </button>
      )}
    </div>
  );
}
