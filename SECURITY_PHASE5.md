# Genius Clan — Security Phase 5: Per-route limits + fingerprint velocity

## Auth public routes (stricter governor)
| Setting | Value |
|---------|--------|
| Key | Client IP (`SmartIpKeyExtractor`) |
| Sustained | **1 request / second** |
| Burst | **5** |

Applies to: register, login, forgot-password, reset-password, verify-email, refresh, 2FA pending polls under public auth.

## Global API (unchanged)
| Setting | Value |
|---------|--------|
| Sustained | 3 / second |
| Burst | 15 |

Auth routes effectively hit the **stricter** of the two layers.

## Fingerprint velocity (register)
- Max **3 registrations per device_fingerprint per rolling hour**
- Tracked via `security_events` (`register_attempt`)
- Exceed → HTTP 429 `rate_limited`

## Stack with Phase 4
1. IP rate limit (this phase)
2. CAPTCHA after 3 failed logins (Phase 4)
3. Hard lockout after 5 fails (auth lockout)
4. Probe path block (Phase 3)

## Next
- Phase 6: structured security logging / alerts
- Phase 7: payment isolation
