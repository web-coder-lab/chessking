-- Chess King — Migration 0005: ad_views unique index
-- Doc 8 §15 step 3a explicitly requires: "duplicate-callback protection,
-- unique index on ad_views.ad_network_transaction_id." Doc 1's original
-- schema didn't include it. Additive-only, per Doc 1's migration rule.
-- NULLs (rows with no transaction id, if any ever existed) don't
-- conflict with each other under SQLite's UNIQUE semantics, so this is
-- safe to add without backfilling.

CREATE UNIQUE INDEX idx_ad_views_txn_id ON ad_views(ad_network_transaction_id);
