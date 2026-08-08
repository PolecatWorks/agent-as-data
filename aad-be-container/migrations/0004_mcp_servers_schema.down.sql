-- Migration 0004 Down: Drop Remote MCP Server Cache Table
DROP INDEX IF EXISTS idx_mcp_servers_name;

DROP TABLE IF EXISTS mcp_servers CASCADE;
