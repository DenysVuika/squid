-- Migration 015: Add timeout support for background jobs
-- Adds timeout_seconds field to control maximum execution time

ALTER TABLE background_jobs ADD COLUMN timeout_seconds INTEGER DEFAULT 3600;