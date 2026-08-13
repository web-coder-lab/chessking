import { useState } from 'react';
import './Button.css';

/**
 * Doc 4 §3 Component States (applies globally):
 * Default / Pressed (scale 0.96-0.98) / Disabled (opacity ~0.4) /
 * Loading (spinner replaces label, disabled) / Error (red border/text)
 *
 * Doc 4 §2.2: primary auth button = full-width, gold bg, dark text,
 * 48px height, rounded-pill. This component covers that plus the
 * secondary/outline variants used elsewhere (Wallet "Add Coins", etc.)
 */
export default function Button({
  children,
  onClick,
  variant = 'primary', // 'primary' | 'outline' | 'destructive'
  loading = false,
  disabled = false,
  loadingLabel,
  fullWidth = true,
  pill = false,
  type = 'button',
  style,
}) {
  const [pressed, setPressed] = useState(false);
  const isDisabled = disabled || loading;

  return (
    <button
      type={type}
      className={[
        'ck-button',
        `ck-button--${variant}`,
        pill ? 'ck-button--pill' : '',
        fullWidth ? 'ck-button--full' : '',
        pressed && !isDisabled ? 'ck-button--pressed' : '',
      ].join(' ').trim()}
      style={style}
      disabled={isDisabled}
      onClick={onClick}
      onPointerDown={() => setPressed(true)}
      onPointerUp={() => setPressed(false)}
      onPointerLeave={() => setPressed(false)}
      aria-busy={loading}
    >
      {loading ? (
        <>
          <span className="ck-spinner" aria-hidden="true" />
          <span>{loadingLabel ?? 'Loading...'}</span>
        </>
      ) : (
        children
      )}
    </button>
  );
}
