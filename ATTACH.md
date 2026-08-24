# Final attach — Frontend ↔ API

```
https://genius-clan.onrender.com     (static SPA)
            │
            │  REST + WS
            ▼
https://genius-clan-api.onrender.com (Rust API only)
```

| Client | API | WS |
|--------|-----|-----|
| Web (Render) | `VITE_API_BASE=https://genius-clan-api.onrender.com` | `wss://genius-clan-api.onrender.com` |
| APK (Capacitor) | same (`.env.production`) | same |
| Local web | `http://localhost:8080` | `ws://localhost:8080` |

Config code: `frontend/src/config/endpoints.js`

CORS on API includes `https://genius-clan.onrender.com` + Capacitor origins.
