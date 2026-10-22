ALTER TABLE place ADD COLUMN cancelled_at TEXT;

UPDATE place
SET cancelled_at = updated_at
WHERE status = 'cancelled';
