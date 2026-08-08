-- Migration 0004: Remote MCP Server Cache Schema

CREATE TABLE IF NOT EXISTS mcp_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_name VARCHAR(255) UNIQUE NOT NULL,
    transport_type VARCHAR(50) NOT NULL,
    endpoint_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    cached_capabilities JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mcp_servers_name ON mcp_servers(server_name);
