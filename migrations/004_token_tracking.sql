-- Add token tracking and model metadata to sessions table
-- This allows us to track usage and costs per session

ALTER TABLE sessions ADD COLUMN model_id TEXT;
ALTER TABLE sessions ADD COLUMN total_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN input_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN output_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN reasoning_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN cache_tokens INTEGER DEFAULT 0;
ALTER TABLE sessions ADD COLUMN cost_usd REAL DEFAULT 0.0;

-- Create an index on model_id for faster filtering by model
CREATE INDEX IF NOT EXISTS idx_sessions_model_id ON sessions(model_id);

-- Create an index on total_tokens for sorting/filtering by usage
CREATE INDEX IF NOT EXISTS idx_sessions_total_tokens ON sessions(total_tokens);
