-- Add context window tracking to sessions table
-- This allows us to track the context window size and calculate utilization

ALTER TABLE sessions ADD COLUMN context_window INTEGER DEFAULT 8192;

-- Create an index on context_window for filtering/analytics
CREATE INDEX IF NOT EXISTS idx_sessions_context_window ON sessions(context_window);
