ALTER TABLE tools RENAME TO mcp_servers;
ALTER TABLE agents RENAME COLUMN attached_tools TO attached_mcp_servers;
ALTER TABLE skills RENAME COLUMN attached_tools TO attached_mcp_servers;
