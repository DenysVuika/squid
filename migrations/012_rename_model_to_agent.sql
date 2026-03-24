-- Migration 012: Rename model_id to agent_id
-- Version: 012
-- Description: Renames model_id column to agent_id to reflect agent-based architecture

-- Rename model_id to agent_id in sessions table
ALTER TABLE sessions RENAME COLUMN model_id TO agent_id;
