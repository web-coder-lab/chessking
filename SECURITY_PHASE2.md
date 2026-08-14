# Genius Clan — Security Phase 2: Edge / Cloudflare

**Status:** Guide ready — needs your custom domain + Cloudflare account (manual steps).  
**Depends on:** Phase 1 research.

Without a domain you own, traffic stays on `*.onrender.com` and Cloudflare cannot sit in front as a full WAF.  
Phase 2 = buy/use domain → Cloudflare → Render (proxied) → enable bots + first rules.

---

## Architecture after Phase 2

```
User
  → Cloudflare (WAF, Bot Fight, rate limit, SSL)
    → Render static:  genius-clan  (or app.yourdomain.com)
    → Render API:     genius-clan-api (or api.yourdomain.com)
      → App rules (JWT, CORS, tower_governor)
      → GitHub private DB
```

---

## Prerequisites

1. Domain you control (e.g. `geniusclan.app`)
2. Free Cloudflare account
3. Render services already live:
   - `https://genius-clan.onrender.com`
   - `https://genius-clan-api.onrender.com`

---

## Step A — Add domain on Render

### Frontend (static site `genius-clan`)
1. Render Dashboard → **genius-clan** → **Settings** → **Custom Domains**
2. Add: `www.yourdomain.com` and/or `yourdomain.com`
3. Note the instructions Render shows for DNS

### API (web service `genius-clan-api`)
1. Same for **genius-clan-api**
2. Add: `api.yourdomain.com`

Wait until Render shows domain **verified** (TLS certificate issued).

---

## Step B — Cloudflare DNS (two-stage)

### Stage 1 — Verify on Render (DNS only / grey cloud)

Render docs: during certificate verification, Cloudflare proxy can break checks.

| Type | Name | Target | Proxy |
|------|------|--------|-------|
| CNAME | `www` | `genius-clan.onrender.com` | **DNS only** (grey) |
| CNAME | `api` | `genius-clan-api.onrender.com` | **DNS only** (grey) |
| CNAME | `@` (root) | `genius-clan.onrender.com` | **DNS only** (if supported) |

- Delete any **AAAA** records (Render has no IPv6).
- SSL/TLS mode in Cloudflare: **Full** (not Flexible).

After Render shows **Certificate active**, go to Stage 2.

### Stage 2 — Turn on Cloudflare proxy (orange cloud) = real WAF

| Name | Target | Proxy |
|------|--------|-------|
| `www` | `genius-clan.onrender.com` | **Proxied** (orange) |
| `api` | `genius-clan-api.onrender.com` | **Proxied** (orange) |

This is **Orange-to-Orange (O2O)** style: your Cloudflare zone proxies to Render’s Cloudflare.  
You get **your** WAF / Bot rules on traffic before it hits the app.

---

## Step C — Bot Fight Mode (Free)

Cloudflare Dashboard → domain → **Security** → **Settings** (or Bots):

1. Enable **Bot Fight Mode**
2. Optional: **Block AI bots** if you don’t want GPTBot/ClaudeBot scraping

**Warning (API):** Bot Fight Mode can challenge legitimate API / non-browser clients.  
For a browser-only SPA talking to your API with normal `fetch`, it usually works.  
If mobile apps or scripts break later → upgrade to Super Bot Fight Mode (Pro) for path exceptions, or keep API on a hostname with softer bot settings.

---

## Step D — First custom WAF rules (Free = up to 5)

Dashboard → **Security** → **WAF** → **Custom rules**

### Rule 1 — Block secret / probe paths
- **Name:** Block probes  
- **Expression:**  
  `(http.request.uri.path contains "/.env") or (http.request.uri.path contains "/.git") or (http.request.uri.path contains "/wp-admin") or (http.request.uri.path contains "/phpmyadmin")`  
- **Action:** Block  

### Rule 2 — Challenge empty User-Agent (optional)
- **Expression:**  
  `(http.user_agent eq "")`  
- **Action:** Managed Challenge  

### Rule 3 — Protect auth paths (stricter) — careful with false positives
- **Expression (example):**  
  `(http.request.uri.path contains "/api/v1/auth/login") or (http.request.uri.path contains "/api/v1/auth/register")`  
- **Action:** Managed Challenge  
  *Or* use Rate limiting rule instead (see Step E).

### Rule 4 — Allow health without challenge
If something challenges `/health`, prefer **not** matching health in challenge rules.  
Render health checks should hit the service directly on `*.onrender.com` (bypass Cloudflare) — keep using onrender URL for health on Render side.

---

## Step E — Rate limiting (Free = 1 rule, 10s window)

**Security** → **WAF** → **Rate limiting rules**

Example (login abuse):
- Path contains `/api/v1/auth/login`
- 10 requests / 10 seconds / IP  
- Action: Block for 10 seconds  

App-layer limits (`tower_governor` + login lockout) remain the second line.

---

## Step F — App config after domain works

Update Render env on **genius-clan-api**:

```
FRONTEND_BASE_URL=https://www.yourdomain.com
```

Redeploy frontend with:

```
VITE_API_BASE=https://api.yourdomain.com
VITE_WS_BASE=wss://api.yourdomain.com
```

CORS already allows `FRONTEND_BASE_URL` + genius-clan.onrender.com; add new origin if needed in code.

---

## What Phase 2 does NOT replace

| Still required in app |
|----------------------|
| JWT on every protected route |
| CORS allow-list |
| Per-IP rate limit (SmartIp) |
| Login lockout / 2FA |
| GitHub token scoped PAT |

Edge stops bulk bots/DDoS noise; app stops logic abuse.

---

## Checklist

- [ ] Domain purchased / available  
- [ ] Custom domains added on both Render services  
- [ ] Cloudflare zone + nameservers  
- [ ] Stage 1 DNS only → certs green  
- [ ] Stage 2 Proxied (orange)  
- [ ] SSL Full  
- [ ] Bot Fight Mode ON  
- [ ] Probe-path Block rule  
- [ ] Auth rate limit rule  
- [ ] FRONTEND_BASE_URL + VITE_* updated  
- [ ] Test: register, login, play, WebSocket if used  

---

## Next phases

| Phase | Topic |
|-------|--------|
| 3 | More WAF managed rules + tune false positives |
| 4 | Turnstile / CAPTCHA step-up on failed logins |
| 5 | App rate limits per route + fingerprint velocity |
| 6 | Logging / alerts / weekly review |
| 7 | Payment isolation |

