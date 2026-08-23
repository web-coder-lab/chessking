# Phase 6 — Draw offers

## Protocol (WS)
- `offer_draw` → broadcast `draw_offered`
- `accept_draw` → `finalize_match` Draw + `agreement` → `match_ended`
- `decline_draw` → broadcast `draw_declined`

## UI
- **Offer draw** button
- Banner: Accept / Decline when opponent offers
