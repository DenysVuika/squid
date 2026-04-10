-- Migration 014: Background Jobs System
-- Adds support for scheduled cron jobs, one-off background tasks, and execution history

-- Main jobs table
CREATE TABLE IF NOT EXISTS background_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    schedule_type TEXT CHECK(schedule_type IN ('cron', 'once')),
    cron_expression TEXT,  -- Standard cron format (e.g., "0 9 * * 1-5")
    priority INTEGER DEFAULT 5 CHECK(priority BETWEEN 0 AND 10),
    max_cpu_percent INTEGER DEFAULT 50 CHECK(max_cpu_percent BETWEEN 1 AND 100),
    timeout_seconds INTEGER DEFAULT 3600,  -- Job timeout (0 = no timeout)
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

-- Job execution history table
-- Tracks individual executions of background jobs (1-to-many relationship)
-- This allows cron jobs to maintain a complete execution history
CREATE TABLE IF NOT EXISTS job_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    session_id TEXT,  -- Links to chat sessions for full conversation history
    status TEXT NOT NULL CHECK(status IN ('completed', 'failed', 'cancelled')),
    result TEXT,  -- JSON: execution result/output
    error_message TEXT,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    duration_ms INTEGER,  -- Execution duration in milliseconds
    tokens_used INTEGER,  -- Total tokens consumed
    cost_usd REAL,  -- Cost of this execution
    FOREIGN KEY (job_id) REFERENCES background_jobs(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_job_executions_job_id ON job_executions(job_id);
CREATE INDEX IF NOT EXISTS idx_job_executions_status ON job_executions(status);
CREATE INDEX IF NOT EXISTS idx_job_executions_started_at ON job_executions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_job_executions_session_id ON job_executions(session_id);
