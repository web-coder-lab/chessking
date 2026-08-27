# Wave 6 — Auth completeness

| Screen | API |
|--------|-----|
| Login 2FA | POST /auth/login/2fa |
| Forgot password | POST /auth/forgot-password |
| Reset password | POST /auth/reset-password |

Login now branches on `requires_2fa` + `pending_id`.
