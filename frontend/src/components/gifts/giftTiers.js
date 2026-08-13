// Price bands used to group gifts into Simple/Nice/Premium/VIP - shared
// by GiftPicker (catalog display) and GiftAnimation (send animation) so
// both always agree on which tier a gift belongs to.
export const GIFT_TIERS = [
  { key: 'simple', label: 'Simple', max: 29 },
  { key: 'nice', label: 'Nice', max: 99 },
  { key: 'premium', label: 'Premium', max: 399 },
  { key: 'vip', label: 'VIP', max: Infinity },
];

export function giftTierFor(priceCoins) {
  return GIFT_TIERS.find((t) => priceCoins <= t.max) ?? GIFT_TIERS[GIFT_TIERS.length - 1];
}
