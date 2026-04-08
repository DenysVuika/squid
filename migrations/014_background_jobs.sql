-- Migration 014: Background Jobs Table
-- Adds support for scheduled cron jobs and one-off background tasks

CREATE TABLE IF NOT EXISTS background_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    schedule_type TEXT CHECK(schedule_type IN ('cron', 'once')),
    cron_expression TEXT,  -- Standard cron format (e.g., "0 9 * * 1-5")
    priority INTEGER DEFAULT 5 CHECK(priority BETWEEN 0 AND 10),
    max_cpu_percent INTEGER DEFAULT 50 CHECK(max_cpu_percent BETWEEN 1 AND 100),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    last_run TIMESTAMP,
    next_run TIMESTAMP,
    retries INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    payload TEXT NOT NULL,  -- JSON: {agent_id, message, system_prompt, file_path, session_id}
    result TEXT,  -- JSON: execution result
    error_message TEXT,
    is_active BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_jobs_status ON background_jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_is_active ON background_jobs(is_active);
CREATE INDEX IF NOT EXISTS idx_jobs_priority ON background_jobs(priority DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_schedule_type ON background_jobs(schedule_type);
