-- Part 21: default support contact
INSERT INTO static_pages (key, content, updated_by, updated_at)
VALUES ('support_email', 'workn8312@gmail.com', 'system', datetime('now'))
ON CONFLICT(key) DO UPDATE SET
  content = excluded.content,
  updated_at = excluded.updated_at
WHERE static_pages.content IS NULL OR static_pages.content = '' OR static_pages.content LIKE '%genius-clan.app%';
