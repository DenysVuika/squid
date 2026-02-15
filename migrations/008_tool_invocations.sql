-- Tool invocations tracking
-- Version: 008
-- Description: Adds tools column to messages table to store completed tool invocations

-- Add tools column to messages table
-- Stores JSON array of tool invocations (name, arguments, result, error)
ALTER TABLE messages ADD COLUMN tools TEXT; -- JSON array of tool invocations
