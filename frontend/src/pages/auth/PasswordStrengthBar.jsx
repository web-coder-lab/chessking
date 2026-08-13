/**
 * Doc 4 §2.2 Reset Password screen: "Live strength indicator bar below
 * password field (weak/fair/strong, color-coded red/amber/green)."
 * Reuses the same real rules as backend validation.rs (8+ chars, upper,
 * lower, digit) to decide the tier, so the bar never lies about what the
 * server will actually accept.
 */
function scorePassword(pw) {
  if (!pw) return { tier: 'weak', pct: 0 };
  let score = 0;
  if (pw.length >= 8) score++;
  if (/[A-Z]/.test(pw)) score++;
  if (/[a-z]/.test(pw)) score++;
  if (/[0-9]/.test(pw)) score++;
  if (/[^A-Za-z0-9]/.test(pw)) score++; // special char, recommended not required

  if (score <= 2) return { tier: 'weak', pct: 33 };
  if (score <= 4) return { tier: 'fair', pct: 66 };
  return { tier: 'strong', pct: 100 };
}

const TIER_COLOR = {
  weak: 'var(--danger-red)',
  fair: '#F5A623', // amber
  strong: 'var(--success-green)',
};

const TIER_LABEL = {
  weak: 'Weak password',
  fair: 'Fair password',
  strong: 'Strong password',
};

export default function PasswordStrengthBar({ password }) {
  const { tier, pct } = scorePassword(password);
  return (
    <>
      <div className="ck-auth-strength-bar">
        <div
          className="ck-auth-strength-fill"
          style={{ width: `${pct}%`, background: TIER_COLOR[tier] }}
        />
      </div>
      {password && (
        <p className="ck-auth-strength-label" style={{ color: TIER_COLOR[tier] }}>
          {TIER_LABEL[tier]}
        </p>
      )}
    </>
  );
}
