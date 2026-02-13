-- Logs table migration
-- Version: 002
-- Description: Creates table for storing application logs

-- Logs table
-- Stores application logs for debugging and troubleshooting
CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    level TEXT NOT NULL CHECK(level IN ('trace', 'debug', 'info', 'warn', 'error')),
    target TEXT NOT NULL,  -- Module path (e.g., 'squid::api', 'squid::session')
    message TEXT NOT NULL,
    session_id TEXT,       -- Optional: correlate logs with specific sessions
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- Indexes for log querying
CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_session_id ON logs(session_id);
CREATE INDEX IF NOT EXISTS idx_logs_target ON logs(target);
