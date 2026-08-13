-- Chess King — Migration 0004: risk_scores unique index
-- Doc 8 §1.1: risk_scores is conceptually one row per user (a running
-- score), but Doc 1's original schema didn't declare user_id UNIQUE.
-- Additive-only fix, per Doc 1's "never modify a table directly, always
-- a new migration" rule.

CREATE UNIQUE INDEX idx_risk_scores_user_id ON risk_scores(user_id);
