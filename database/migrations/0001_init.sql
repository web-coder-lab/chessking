-- Chess King — Initial Schema (Phase 1)
-- Matches 01_DATABASE_SCHEMA.md exactly. SQLite 3.
-- UUIDs: TEXT (UUIDv4). Booleans: INTEGER (0/1). Timestamps: TEXT (ISO 8601 UTC).
-- Coins: INTEGER (whole numbers only, no fractional coins).
-- Never modify a table directly in production — always add a new migration.

PRAGMA foreign_keys = ON;

-- NOTE ON TABLE ORDER: SQLite resolves FK targets at reference time, not
-- creation time within the same script, so forward references (e.g. users
-- -> shop_items, gifts -> matches) are fine here.

-- =========================================================
-- 1. users
-- =========================================================
CREATE TABLE users (
    id                  TEXT PRIMARY KEY,
    username            TEXT NOT NULL UNIQUE,               -- 3-20 chars, immutable after register
    username_lower      TEXT NOT NULL UNIQUE,                -- case-insensitive lookup
    email               TEXT NOT NULL UNIQUE,                -- stored lowercased
    password_hash       TEXT NOT NULL,                        -- Argon2id
    email_verified      INTEGER NOT NULL DEFAULT 0,
    avatar_id           TEXT,
    banner_id           TEXT,
    bio                 TEXT,
    country_code        TEXT,                                 -- ISO 3166-1 alpha-2, from geo-IP, not self-selected
    province            TEXT,
    rating              INTEGER NOT NULL DEFAULT 1200,
    coin_balance        INTEGER NOT NULL DEFAULT 0,           -- authoritative; cross-checked against wallet_logs
    two_fa_enabled      INTEGER NOT NULL DEFAULT 0,
    two_fa_secret       TEXT,                                  -- encrypted at rest
    role                TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user','super_admin','security_admin','finance_admin','support_admin','moderator')),
    status              TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','banned')),
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 2. sessions
-- =========================================================
CREATE TABLE sessions (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL REFERENCES users(id),
    refresh_token_hash  TEXT NOT NULL,
    previous_refresh_token_hash TEXT,                     -- one generation back, for rotation-reuse detection
    device_fingerprint  TEXT,
    ip_address          TEXT,
    browser             TEXT,
    os                  TEXT,
    is_active           INTEGER NOT NULL DEFAULT 1,
    last_seen_at        TEXT,                                  -- for online/offline device-approval flow
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at          TEXT NOT NULL                          -- created_at + 3 days
);

-- =========================================================
-- 3. email_verification_tokens
-- =========================================================
CREATE TABLE email_verification_tokens (
    id             TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL REFERENCES users(id),
    token_hash     TEXT NOT NULL,
    expires_at     TEXT NOT NULL,                                 -- 15 minutes from creation
    used           INTEGER NOT NULL DEFAULT 0,
    pending_email  TEXT,                                          -- set only for email-change requests (§2 change-email flow); NULL means "verify the address already on the account" (registration)
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 4. password_reset_tokens (same shape as email_verification_tokens)
-- =========================================================
CREATE TABLE password_reset_tokens (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id),
    token_hash  TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    used        INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 5. two_fa_pending_verifications
-- =========================================================
CREATE TABLE two_fa_pending_verifications (
    id                             TEXT PRIMARY KEY,
    user_id                        TEXT NOT NULL REFERENCES users(id),
    device_fingerprint             TEXT,                        -- the NEW device attempting login
    code_hash                      TEXT NOT NULL,                -- hashed 6-digit code
    approval_status                TEXT NOT NULL DEFAULT 'pending' CHECK (approval_status IN ('pending','approved','denied','expired')),
    requires_old_device_approval   INTEGER NOT NULL DEFAULT 0,   -- true when old session is online
    expires_at                     TEXT NOT NULL,
    created_at                     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 6. wallet_logs — immutable ledger, source of truth for coin_balance
-- =========================================================
CREATE TABLE wallet_logs (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT NOT NULL REFERENCES users(id),
    type                TEXT NOT NULL CHECK (type IN ('deposit','shop_purchase','gift_sent','daily_reward','ad_reward','referral_reward','admin_adjustment','refund')),
    amount              INTEGER NOT NULL,                        -- positive = credit, negative = debit
    balance_before      INTEGER NOT NULL,
    balance_after       INTEGER NOT NULL,
    reference_id        TEXT,                                    -- FK to related row depending on `type`
    ip_address          TEXT,
    device_fingerprint  TEXT,
    status              TEXT NOT NULL DEFAULT 'success' CHECK (status IN ('success','failed','pending','reversed')),
    prev_hash           TEXT,                                    -- hash of previous ledger row (tamper-proof chain, Doc 7)
    row_hash            TEXT,                                    -- hash of this row's contents + prev_hash
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 7. payment_transactions
-- =========================================================
CREATE TABLE payment_transactions (
    id                       TEXT PRIMARY KEY,
    user_id                  TEXT NOT NULL REFERENCES users(id),
    gateway                  TEXT NOT NULL CHECK (gateway IN ('jazzcash','easypaisa','googlepay')),
    gateway_transaction_id   TEXT UNIQUE,                        -- idempotency / duplicate detection
    idempotency_key          TEXT,                                -- client-generated, §7 double-tap protection
    payer_phone              TEXT,                                -- mobile-wallet number for jazzcash/easypaisa; format-checked, not gateway-verified (see gateway.rs stub note)
    amount_pkr               INTEGER NOT NULL,
    coins_credited           INTEGER,                            -- amount_pkr / coin_rate at transaction time
    coin_rate_used           INTEGER,                             -- snapshot of rate, in case admin changes it later
    status                   TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','success','failed','refunded')),
    webhook_verified         INTEGER NOT NULL DEFAULT 0,
    raw_gateway_response     TEXT,                                -- JSON blob, audit purposes
    created_at               TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at             TEXT,
    UNIQUE(user_id, idempotency_key)
);

-- =========================================================
-- 8. shop_items
-- =========================================================
CREATE TABLE shop_items (
    id                   TEXT PRIMARY KEY,
    category             TEXT NOT NULL CHECK (category IN ('board','piece_set','avatar','banner','gift')),
    name                 TEXT NOT NULL,
    description          TEXT,
    image_url            TEXT,
    icon_emoji           TEXT,                                    -- plain Unicode glyph, used in place of image_url where no real asset file exists (e.g. every gift) - avoids referencing nonexistent images or needing any copyrighted artwork
    price_coins          INTEGER NOT NULL,
    is_active            INTEGER NOT NULL DEFAULT 1,
    is_limited_edition   INTEGER NOT NULL DEFAULT 0,
    available_from       TEXT,
    available_until      TEXT,
    created_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 9. inventory
-- =========================================================
CREATE TABLE inventory (
    id             TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL REFERENCES users(id),
    shop_item_id   TEXT NOT NULL REFERENCES shop_items(id),
    is_equipped    INTEGER NOT NULL DEFAULT 0,                  -- one-equipped-per-category enforced at API layer
    acquired_via   TEXT NOT NULL CHECK (acquired_via IN ('purchase','gift_received')),
    acquired_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 10. matches (created before gifts, since gifts.match_id references it)
-- =========================================================
CREATE TABLE matches (
    id                    TEXT PRIMARY KEY,
    player_white_id       TEXT NOT NULL REFERENCES users(id),
    player_black_id       TEXT NOT NULL REFERENCES users(id),
    match_type            TEXT NOT NULL CHECK (match_type IN ('ranked','casual','custom')),
    status                TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress','completed','aborted')),
    result                TEXT CHECK (result IN ('white_win','black_win','draw','void')),
    result_reason         TEXT CHECK (result_reason IN ('checkmate','resign','disconnect_timeout','cheat_detected','agreement')),
    pgn                   TEXT,                                  -- full move history, PGN format
    white_rating_before   INTEGER,
    black_rating_before   INTEGER,
    white_rating_after    INTEGER,
    black_rating_after    INTEGER,
    started_at            TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at              TEXT
);

-- =========================================================
-- 11. gifts — one-way, non-redeemable
-- =========================================================
CREATE TABLE gifts (
    id             TEXT PRIMARY KEY,
    sender_id      TEXT NOT NULL REFERENCES users(id),
    receiver_id    TEXT NOT NULL REFERENCES users(id),
    shop_item_id   TEXT NOT NULL REFERENCES shop_items(id),
    coins_spent    INTEGER NOT NULL,                            -- deducted from sender, permanently consumed
    context        TEXT NOT NULL CHECK (context IN ('profile','in_match')),
    match_id       TEXT REFERENCES matches(id),
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 12. match_disconnect_events
-- =========================================================
CREATE TABLE match_disconnect_events (
    id                     TEXT PRIMARY KEY,
    match_id               TEXT NOT NULL REFERENCES matches(id),
    user_id                TEXT NOT NULL REFERENCES users(id),
    disconnected_at        TEXT NOT NULL,
    reconnected_at         TEXT,
    grace_period_expired   INTEGER NOT NULL DEFAULT 0
);

-- =========================================================
-- 13. match_hint_usage — paid AI hint, casual matches only, max 2/match
-- =========================================================
CREATE TABLE match_hint_usage (
    id              TEXT PRIMARY KEY,
    match_id        TEXT NOT NULL REFERENCES matches(id),
    user_id         TEXT NOT NULL REFERENCES users(id),
    usage_number    INTEGER NOT NULL CHECK (usage_number IN (1,2)),
    coins_spent     INTEGER NOT NULL,                            -- 1 for first use, 2 for second
    move_suggested  TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 14. custom_match_invites
-- =========================================================
CREATE TABLE custom_match_invites (
    id           TEXT PRIMARY KEY,
    sender_id    TEXT NOT NULL REFERENCES users(id),
    receiver_id  TEXT NOT NULL REFERENCES users(id),
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','accepted','declined','expired')),
    match_id     TEXT REFERENCES matches(id),
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 15. referrals
-- =========================================================
CREATE TABLE referrals (
    id                          TEXT PRIMARY KEY,
    inviter_id                  TEXT NOT NULL REFERENCES users(id),
    invited_id                  TEXT NOT NULL REFERENCES users(id),
    invite_link_code            TEXT NOT NULL,
    invited_topup_pkr           INTEGER NOT NULL DEFAULT 0,       -- cumulative top-up by invited user
    invited_topup_target_pkr    INTEGER NOT NULL DEFAULT 300,
    invited_has_spent_in_shop   INTEGER NOT NULL DEFAULT 0,
    reward_claimed              INTEGER NOT NULL DEFAULT 0,
    reward_coins                INTEGER NOT NULL DEFAULT 10,
    created_at                  TEXT NOT NULL DEFAULT (datetime('now')),
    claimed_at                  TEXT
);

-- =========================================================
-- 16. daily_rewards
-- =========================================================
CREATE TABLE daily_rewards (
    id             TEXT PRIMARY KEY,
    user_id        TEXT NOT NULL REFERENCES users(id),
    streak_day     INTEGER NOT NULL CHECK (streak_day BETWEEN 1 AND 7),
    coins_awarded  INTEGER NOT NULL,
    claimed_at     TEXT NOT NULL DEFAULT (datetime('now')),
    claim_date     TEXT NOT NULL                                 -- YYYY-MM-DD, unique per user/day
);

-- =========================================================
-- 17. ad_views
-- =========================================================
CREATE TABLE ad_views (
    id                          TEXT PRIMARY KEY,
    user_id                     TEXT NOT NULL REFERENCES users(id),
    ad_network_transaction_id   TEXT,                             -- from S2S callback, duplicate detection
    coins_awarded               INTEGER NOT NULL DEFAULT 1,
    verified_server_side        INTEGER NOT NULL DEFAULT 0,
    view_date                   TEXT NOT NULL,                     -- date only, enforces daily cap of 10
    created_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 18. leaderboard_snapshot (materialized, refreshed periodically)
-- =========================================================
CREATE TABLE leaderboard_snapshot (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id),
    scope         TEXT NOT NULL CHECK (scope IN ('global','country','province')),
    scope_value   TEXT,                                           -- NULL for global
    rank          INTEGER NOT NULL,
    wins          INTEGER NOT NULL DEFAULT 0,
    losses        INTEGER NOT NULL DEFAULT 0,
    draws         INTEGER NOT NULL DEFAULT 0,
    rating        INTEGER NOT NULL,
    generated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 19. bug_reports
-- =========================================================
CREATE TABLE bug_reports (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    description     TEXT,
    screenshot_url  TEXT,
    status          TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','in_review','resolved')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 20. voice_abuse_reports
-- =========================================================
CREATE TABLE voice_abuse_reports (
    id                TEXT PRIMARY KEY,
    match_id          TEXT NOT NULL REFERENCES matches(id),
    reporter_id       TEXT NOT NULL REFERENCES users(id),
    reported_id       TEXT NOT NULL REFERENCES users(id),
    reason            TEXT,
    audio_buffer_url  TEXT,
    status            TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','in_review','resolved')),
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 21. risk_scores
-- =========================================================
CREATE TABLE risk_scores (
    id                   TEXT PRIMARY KEY,
    user_id              TEXT NOT NULL REFERENCES users(id),
    score                INTEGER NOT NULL DEFAULT 0 CHECK (score BETWEEN 0 AND 100),
    category_breakdown   TEXT,                                    -- JSON: {"wallet":20,"match":10,"login":0,...}
    last_evaluated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 22. security_events
-- =========================================================
CREATE TABLE security_events (
    id                  TEXT PRIMARY KEY,
    user_id             TEXT REFERENCES users(id),
    event_type          TEXT NOT NULL,                            -- e.g. impossible_move_timing, wallet_mismatch, multi_account_device
    severity            INTEGER NOT NULL DEFAULT 1,                -- points added to risk score
    metadata            TEXT,                                      -- JSON blob
    ip_address          TEXT,
    device_fingerprint  TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 23. bans
-- =========================================================
CREATE TABLE bans (
    id                   TEXT PRIMARY KEY,
    user_id              TEXT NOT NULL REFERENCES users(id),
    ban_type             TEXT NOT NULL CHECK (ban_type IN ('temporary','permanent')),
    reason               TEXT NOT NULL,
    evidence_ref         TEXT,                                     -- links to security_events / risk_scores snapshot
    ip_blacklisted       INTEGER NOT NULL DEFAULT 0,
    device_blacklisted   INTEGER NOT NULL DEFAULT 0,
    issued_by            TEXT NOT NULL,                            -- 'system' or admin user_id
    expires_at           TEXT,                                      -- nullable, for temporary bans
    created_at           TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 24. admin_audit_log
-- =========================================================
CREATE TABLE admin_audit_log (
    id          TEXT PRIMARY KEY,
    admin_id    TEXT NOT NULL REFERENCES users(id),
    action      TEXT NOT NULL,                                     -- e.g. update_payment_config, edit_privacy_policy, ban_user
    old_value   TEXT,                                               -- JSON, nullable
    new_value   TEXT,                                               -- JSON, nullable
    ip_address  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 25. app_config — admin-editable runtime config, never hardcoded
-- =========================================================
CREATE TABLE app_config (
    key         TEXT PRIMARY KEY,                                  -- e.g. coin_rate_pkr, jazzcash_api_key, smtp_email, ad_daily_cap
    value       TEXT NOT NULL,                                      -- encrypted at rest for sensitive keys
    updated_by  TEXT REFERENCES users(id),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 26. static_pages
-- =========================================================
CREATE TABLE static_pages (
    key         TEXT PRIMARY KEY,                                  -- privacy_policy / about / support_email
    content     TEXT,
    updated_by  TEXT REFERENCES users(id),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- =========================================================
-- 27. Indexes (per spec §27)
-- =========================================================
CREATE UNIQUE INDEX idx_users_username_lower ON users(username_lower);
CREATE UNIQUE INDEX idx_users_email ON users(email);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_wallet_logs_user_created ON wallet_logs(user_id, created_at);
CREATE UNIQUE INDEX idx_payment_tx_gateway_tx_id ON payment_transactions(gateway_transaction_id);
CREATE INDEX idx_ad_views_user_date ON ad_views(user_id, view_date);
CREATE UNIQUE INDEX idx_daily_rewards_user_date ON daily_rewards(user_id, claim_date);
CREATE INDEX idx_matches_white ON matches(player_white_id);
CREATE INDEX idx_matches_black ON matches(player_black_id);

-- Seed default runtime config
INSERT INTO app_config (key, value) VALUES
    ('coin_rate_pkr', '2'),
    ('ad_daily_cap', '10'),
    ('referral_topup_target_pkr', '300'),
    ('referral_reward_coins', '10'),
    ('hint_cost_first_use', '1'),
    ('hint_cost_second_use', '2');
