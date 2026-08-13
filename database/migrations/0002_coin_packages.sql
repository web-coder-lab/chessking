-- Chess King — Migration 0002: coin_packages
-- Doc 5 §1: "Coin packages (predefined, admin-editable, shown in the
-- Wallet screen package grid) ... must not be hardcoded in frontend code."
-- Per Doc 1's rule: never modify a table directly in production, always
-- add a new migration — this is additive only, does not touch 0001.

CREATE TABLE coin_packages (
    id           TEXT PRIMARY KEY,
    amount_pkr   INTEGER NOT NULL,
    coins        INTEGER NOT NULL,
    bonus_label  TEXT,                          -- e.g. "+10% Bonus", NULL if none
    is_active    INTEGER NOT NULL DEFAULT 1,
    sort_order   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed the exact default tiers from Doc 5 §1
INSERT INTO coin_packages (id, amount_pkr, coins, bonus_label, sort_order) VALUES
    ('pkg_100',   100,   50,   NULL,        1),
    ('pkg_500',   500,   250,  NULL,        2),
    ('pkg_1000',  1000,  550,  '+10% Bonus', 3),
    ('pkg_2000',  2000,  1150, '+15% Bonus', 4),
    ('pkg_5000',  5000,  3000, '+20% Bonus', 5);
