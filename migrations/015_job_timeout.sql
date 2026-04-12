-- Migration 015: Add timeout_seconds to background_jobs
-- Fixes existing databases created before the column was added to migration 014.
-- For new databases, the column already exists via CREATE TABLE in migration 014,
-- so ALTER TABLE will fail with "duplicate column" — we handle this gracefully.

-- SQLite doesn't support IF NOT EXISTS on ALTER TABLE ADD COLUMN,
-- so we use a workaround: attempt the ALTER TABLE and ignore the specific error.
-- The application code checks for the column and skips if it already exists.

ALTER TABLE background_jobs ADD COLUMN timeout_seconds INTEGER DEFAULT 3600;
