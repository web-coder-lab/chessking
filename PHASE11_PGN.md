# Phase 11 — PGN share

## Export
- Standard PGN headers (Event, Site, Date, White, Black, Result, MatchId)
- Body from SAN move list

## UI
- In-match: history panel → **Copy PGN** / **Share**
- End screen: **Copy PGN** / **Share PGN**

Share uses `navigator.share` when available, else clipboard.
