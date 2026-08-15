# Part 9 — Genius email templates

All HTML emails share `shell()`: dark `#0F1115`, gold `#D4AF37`, crown branding.

| Method | Subject | CTA |
|--------|---------|-----|
| `send_verification_email` | Verify your Genius Clan account | Verify Email → `/verify-email?token=` |
| **`send_complete_signup_email`** (new) | Complete your Genius Clan signup | Complete signup → `/complete-signup?token=` |
| `send_welcome_email` | Welcome to Genius Clan | Open app |
| `send_password_reset_email` | Reset password | Reset → `/reset-password?token=` |
| `send_new_device_login_email` | New sign-in | Security notice |
| `send_2fa_status_email` | 2FA on/off | — |
| `send_payment_confirmation_email` | Payment confirmed | — |
| `send_test_email` | SMTP test | — |

## Next
Part 10 — API: `POST /auth/register-intent` (email only → send complete-signup link, no user row yet)
Part 11 — Complete-signup page + `POST /auth/complete-signup`
