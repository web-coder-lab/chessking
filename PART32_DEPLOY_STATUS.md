# Part 32 — Deploy status snapshot

## API (`genius-clan-api`)
| Item | Status |
|------|--------|
| Latest | Part 31 low-RAM Dockerfile **queued / building** |
| Prior | Multiple **build_failed** |
| Live `/health` | 200 ok |
| Live `/api/v1/auth/login` | **404** HTML (old/broken binary) |

## Frontend (`genius-clan`)
| Item | Status |
|------|--------|
| Live site | **200** — title Genius Clan |
| Last live commit | ~Part 24 (newer theme/auth FE may need redeploy) |
| Manual redeploy | Triggered in Part 32 |

## Action
Wait for API **live** after Part 31 image. If still fail → plan upgrade or external build.
