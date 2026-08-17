-- Migration 0008: Remove archived_at column from agents table
DROP INDEX IF EXISTS idx_agents_archived_at;
ALTER TABLE agents DROP COLUMN IF EXISTS archived_at;
