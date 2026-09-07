-- Migration 0024: Add MCP sync policy and status to tools

ALTER TABLE tools 
ADD COLUMN IF NOT EXISTS sync_policy VARCHAR(50) NOT NULL DEFAULT 'manual',
ADD COLUMN IF NOT EXISTS sync_status VARCHAR(50) NOT NULL DEFAULT 'synced',
ADD COLUMN IF NOT EXISTS last_sync_error TEXT;

CREATE INDEX IF NOT EXISTS idx_tools_sync_status ON tools(sync_status);
