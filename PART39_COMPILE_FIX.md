# Part 39 — Compile fix (E0382)

## Root cause
`src/social/rewards.rs`: match moved `date: String` out of `last`, then `last.is_none()` borrowed moved value.

## Fix
Match on `&last` and compare `date == &today`.

## Rebuild
Push triggers GH Actions `Build API image` again.
