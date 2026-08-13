-- Chess King — Migration 0006: notifications
-- Doc 9 §12 documents GET /notifications, POST /notifications/{id}/read,
-- PATCH /notifications/settings — but Doc 1's schema has no notifications
-- table at all. Additive-only, per Doc 1's migration rule.

CREATE TABLE notifications (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id),
    type        TEXT NOT NULL,        -- e.g. 'gift_received', 'custom_match_invite', 'device_approval_request'
    title       TEXT NOT NULL,
    body        TEXT,
    reference_id TEXT,                -- id of the related gift/invite/match/etc.
    is_read     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_notifications_user_id ON notifications(user_id, created_at);

-- Per-user notification preferences (Doc 9 Sec12: PATCH /notifications/settings)
CREATE TABLE notification_settings (
    user_id     TEXT PRIMARY KEY REFERENCES users(id),
    enabled     INTEGER NOT NULL DEFAULT 1,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
