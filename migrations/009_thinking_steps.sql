-- Thinking steps tracking
-- Version: 009
-- Description: Adds thinking_steps table to store the exact order of reasoning and tool steps

CREATE TABLE thinking_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,
    step_order INTEGER NOT NULL,
    step_type TEXT NOT NULL CHECK(step_type IN ('reasoning', 'tool')),
    content TEXT,
    tool_name TEXT,
    tool_arguments TEXT,
    tool_result TEXT,
    tool_error TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_thinking_steps_message_id ON thinking_steps(message_id);
CREATE INDEX idx_thinking_steps_order ON thinking_steps(message_id, step_order);
