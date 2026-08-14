# Part 5 — IP allowlist OFF (public API)

## Change
Render env `IP_ALLOWLIST` set to **empty**.

## Behaviour (code already)
- Empty list → middleware allows all client IPs
- Non-empty → only listed IPs reach API (others Genius 404)

## Result
Normal users / any network can call API again (no lockout 404).

Probe paths (`.env`, `.git`, etc.) still blocked by `probe_guard`.

## Re-lock later (optional admin only)
```
IP_ALLOWLIST=x.x.x.x
```
