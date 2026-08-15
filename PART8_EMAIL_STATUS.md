# Part 8 — Email send status not silent

## Changes
- `AuthError::EmailSendFailed` when SMTP missing or send fails
- Register response: `{ status, email_sent, message }`
- Frontend shows message if `email_sent: false`
- Resend verification propagates send errors

## Behaviour
- Account still created if email fails (register)
- User sees honest message, not fake "check your email" only
