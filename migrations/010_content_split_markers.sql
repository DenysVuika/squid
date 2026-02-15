-- Content split markers for tool approvals
-- Version: 010
-- Description: Adds content_before_tool column to track where tools appear in message content flow

ALTER TABLE thinking_steps ADD COLUMN content_before_tool TEXT;
