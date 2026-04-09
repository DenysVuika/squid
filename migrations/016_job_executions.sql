-- Job Executions History Table
-- Tracks individual executions of background jobs (1-to-many relationship)
-- This allows cron jobs to maintain a complete execution history

CREATE TABLE IF NOT EXISTS job_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    session_id TEXT,  -- Links to sessions for full conversation history
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
