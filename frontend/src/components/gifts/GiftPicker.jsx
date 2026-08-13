import { useEffect, useState } from 'react';
import { giftsApi } from '../../services/api';
import { GIFT_TIERS, giftTierFor } from './giftTiers';
import './GiftPicker.css';

/**
 * recipientLabel: who the gifts are going to, shown in the header.
 * onSend(item): async callback, called when a gift is confirmed.
 * onClose(): close the picker.
 */
export default function GiftPicker({ recipientLabel, onSend, onClose }) {
  const [catalog, setCatalog] = useState(null);
  const [sendingId, setSendingId] = useState(null);

  useEffect(() => {
    giftsApi.getCatalog().then((d) => setCatalog(d.items));
  }, []);

  async function handlePick(item) {
    setSendingId(item.id);
    try {
      await onSend(item);
    } finally {
      setSendingId(null);
    }
  }

  const grouped = GIFT_TIERS.map((tier) => ({
    ...tier,
    items: (catalog || []).filter((i) => giftTierFor(i.price_coins).key === tier.key),
  })).filter((t) => t.items.length > 0);

  return (
    <div className="ck-gift-picker__overlay" onClick={onClose}>
      <div className="ck-gift-picker__sheet" onClick={(e) => e.stopPropagation()}>
        <h2 className="section-heading" style={{ padding: '0 var(--space-1)' }}>
          Send a Gift{recipientLabel ? ` to ${recipientLabel}` : ''}
        </h2>

        {catalog === null ? (
          <p className="text-secondary">Loading...</p>
        ) : (
          grouped.map((tier) => (
            <section key={tier.key} className="ck-gift-picker__tier">
              <h3 className="ck-gift-picker__tier-label">{tier.label}</h3>
              <div className="ck-gift-picker__grid">
                {tier.items.map((item) => (
                  <button
                    key={item.id}
                    className="ck-gift-picker__option"
                    onClick={() => handlePick(item)}
                    disabled={sendingId !== null}
                    aria-label={`Send ${item.name} for ${item.price_coins} coins`}
                  >
                    <span className="ck-gift-picker__icon" aria-hidden="true">
                      {sendingId === item.id ? '…' : (item.icon_emoji || '🎁')}
                    </span>
                    <span className="ck-gift-picker__name">{item.name}</span>
                    <span className="ck-gift-picker__price tabular-nums">{item.price_coins} 🪙</span>
                  </button>
                ))}
              </div>
            </section>
          ))
        )}
      </div>
    </div>
  );
}
