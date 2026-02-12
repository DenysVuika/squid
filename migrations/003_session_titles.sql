-- Migration 003: Add title column to sessions table
-- Version: 003
-- Description: Adds title column for auto-generated and user-editable session names

-- Add title column to sessions table
-- This will be auto-generated from the first user message
ALTER TABLE sessions ADD COLUMN title TEXT;

-- Create index for title searches (for future search functionality)
CREATE INDEX IF NOT EXISTS idx_sessions_title ON sessions(title);
