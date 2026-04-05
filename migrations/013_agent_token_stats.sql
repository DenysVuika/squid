-- Add agent_token_stats table for tracking token usage per agent
-- This allows us to see lifetime statistics and average cost savings per agent

CREATE TABLE IF NOT EXISTS agent_token_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL UNIQUE,
    total_sessions INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    reasoning_tokens INTEGER DEFAULT 0,
    cache_tokens INTEGER DEFAULT 0,
    total_cost_usd REAL DEFAULT 0.0,
    first_used_at INTEGER NOT NULL,
    last_used_at INTEGER NOT NULL
);

-- Create an index on agent_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_agent_token_stats_agent_id ON agent_token_stats(agent_id);
