import { useEffect, useMemo, useState } from 'react';
import { giftTierFor } from './giftTiers';
import './GiftAnimation.css';

// Duration must match each tier's CSS animation timing (see
// GiftAnimation.css custom properties) - used to auto-dismiss.
const TIER_DURATION_MS = { simple: 1400, nice: 1800, premium: 2200, vip: 2800 };
const TIER_PARTICLE_COUNT = { simple: 0, nice: 6, premium: 12, vip: 24 };
const PARTICLE_COLORS = ['var(--accent-gold)', 'var(--accent-gold-dim)', 'var(--success-green)', 'var(--text-primary)'];

function buildParticles(count) {
  return Array.from({ length: count }, (_, i) => {
    // Evenly spread around the circle with a little jitter so a 24-particle
    // VIP burst doesn't look like a rigid wheel, plus randomized distance
    // and timing so the burst reads as organic rather than mechanical.
    const angle = (360 / count) * i + (Math.random() * 20 - 10);
    const distance = 90 + Math.random() * 70;
    const rad = (angle * Math.PI) / 180;
    return {
      id: i,
      tx: Math.cos(rad) * distance,
      ty: Math.sin(rad) * distance,
      rotate: Math.random() * 360,
      delay: Math.random() * 0.15,
      color: PARTICLE_COLORS[i % PARTICLE_COLORS.length],
      isSpark: i % 3 === 0,
    };
  });
}

/**
 * item: the gift that was just sent (needs icon_emoji, price_coins, name).
 * onComplete: called once the animation has fully played out, so the
 * caller can unmount this component.
 */
export default function GiftAnimation({ item, onComplete }) {
  const tier = giftTierFor(item.price_coins).key;
  const duration = TIER_DURATION_MS[tier];
  const particles = useMemo(() => buildParticles(TIER_PARTICLE_COUNT[tier]), [tier]);
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const t = setTimeout(() => {
      setVisible(false);
      onComplete?.();
    }, duration);
    return () => clearTimeout(t);
  }, [duration, onComplete]);

  if (!visible) return null;

  return (
    <div className={`ck-gift-anim ck-gift-anim--${tier}`} style={{ '--duration': `${duration}ms` }} aria-hidden="true">
      {tier === 'vip' && <div className="ck-gift-anim__shimmer" />}
      {(tier === 'premium' || tier === 'vip') && <div className="ck-gift-anim__ring ck-gift-anim__ring--1" />}
      {tier === 'vip' && <div className="ck-gift-anim__ring ck-gift-anim__ring--2" />}

      <div className="ck-gift-anim__stage">
        {particles.map((p) => (
          <span
            key={p.id}
            className={`ck-gift-anim__particle ${p.isSpark ? 'ck-gift-anim__particle--spark' : ''}`}
            style={{
              '--tx': `${p.tx}px`,
              '--ty': `${p.ty}px`,
              '--rotate': `${p.rotate}deg`,
              '--delay': `${p.delay}s`,
              '--color': p.color,
            }}
          >
            {p.isSpark ? '✨' : ''}
          </span>
        ))}

        <span className="ck-gift-anim__emoji">{item.icon_emoji || '🎁'}</span>
      </div>

      <span className="ck-gift-anim__label">{item.name}</span>
    </div>
  );
}
