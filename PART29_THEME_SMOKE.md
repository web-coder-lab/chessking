# Part 29 — Theme / pages visual smoke

## Design system
- Tokens: dark `#0F1115`, surface `#1A1D23`, gold `#D4AF37`
- Utilities: `.ck-brand-bar`, `.ck-page-shell`, `.ck-card-gold`, `.ck-btn-primary`

## Page coverage (Parts 16–21)
| Page | Theme applied |
|------|----------------|
| Splash | Crown, brand tag, lang rows |
| Auth / Complete signup / Verify email | Brand + gold tabs |
| 404 | Branded Genius card |
| Dashboard | Claim gold, Play pulse, shortcuts |
| Play / Board | Option cards, info bars |
| Shop / Inventory / Wallet | Cards, equip glow, balance |
| Profile / Settings / Support | Avatar ring, rows, support CTA |
| Leaderboard / Invite / Custom match | Shells + “me” highlight |

## Frontend host
`https://genius-clan.onrender.com` → HTTP 200 (static SPA)

## Manual smoke (browser)
- [ ] Splash → language → Auth (gold underline tabs)
- [ ] Dashboard Play button gold ring
- [ ] Shop segment active = gold pill
- [ ] Inventory equipped = gold border
- [ ] Support = workn8312@gmail.com + Send/Copy
- [ ] Unknown path → Genius 404 (SPA) or API Genius HTML

## Note
New FE CSS ships when static site rebuilds from `main`. API theme-independent.
