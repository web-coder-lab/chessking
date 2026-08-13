import './Card.css';

/**
 * Doc 4 §1.3: standard card = bg-surface, 16px radius, soft shadow.
 * Glassmorphism variant reserved for dashboard highlight cards only
 * (e.g. Daily Reward banner) — never use glass everywhere.
 */
export default function Card({ children, glass = false, className = '', onClick, style }) {
  return (
    <div
      className={`ck-card ${glass ? 'ck-card--glass' : ''} ${className}`.trim()}
      onClick={onClick}
      style={style}
    >
      {children}
    </div>
  );
}
