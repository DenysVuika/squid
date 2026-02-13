-- Add reasoning column to messages table
-- Version: 007
-- Description: Adds optional reasoning field to store LLM thinking process

ALTER TABLE messages ADD COLUMN reasoning TEXT;
