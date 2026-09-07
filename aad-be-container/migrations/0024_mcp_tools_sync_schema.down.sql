-- Migration 0024 Rollback: Drop MCP sync policy and status from tools

DROP INDEX IF EXISTS idx_tools_sync_status;

ALTER TABLE tools 
DROP COLUMN IF EXISTS last_sync_error,
DROP COLUMN IF EXISTS sync_status,
DROP COLUMN IF EXISTS sync_policy;
