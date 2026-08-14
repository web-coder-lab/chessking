# IP allowlist + Genius 404

## Behaviour

| Who | Frontend (static) | API |
|-----|-------------------|-----|
| IP **on** `IP_ALLOWLIST` | Full app pages | **All** API paths |
| IP **not** on list | Full app pages (SPA) | **404 Genius page** (except `/health`) |
| `IP_ALLOWLIST` **empty** | Full | Full (normal public mode) |

Frontend host is always public — players can open the site.  
When allowlist is set, **API is locked** so only your IPs can login/play via API.

## Enable (Render → genius-clan-api → Environment)

```
IP_ALLOWLIST=YOUR.PUBLIC.IP.HERE
```

Multiple:

```
IP_ALLOWLIST=1.2.3.4,5.6.7.8,10.0.0.0/8
```

Leave empty for open public API.

## Genius 404

- Frontend unknown routes → branded **Genius Clan 404** page  
- API denied / unknown probes → HTML **Genius 404** (or probe 404)

## Note

Find your public IP: search “what is my IP”.  
Mobile IPs change often — update env when needed.
