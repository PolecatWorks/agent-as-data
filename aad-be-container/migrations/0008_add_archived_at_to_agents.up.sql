-- Migration 0008: Add archived_at column to agents table for Soft Delete
ALTER TABLE agents ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ DEFAULT NULL;
CREATE INDEX IF NOT EXISTS idx_agents_archived_at ON agents(archived_at);
