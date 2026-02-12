-- Migration 003: Add title column to sessions table
-- Version: 003
-- Description: Adds title column for auto-generated and user-editable session names

-- Add title column to sessions table (will fail silently if column exists)
-- This will be auto-generated from the first user message
-- Note: SQLite doesn't support IF NOT EXISTS for ALTER TABLE ADD COLUMN
-- If this migration was already applied, the database layer will skip it

ALTER TABLE sessions ADD COLUMN title TEXT;

-- Create index for title searches (for future search functionality)
CREATE INDEX IF NOT EXISTS idx_sessions_title ON sessions(title);
