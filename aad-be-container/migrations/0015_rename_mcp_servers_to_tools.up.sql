ALTER TABLE mcp_servers RENAME TO tools;
ALTER TABLE agents RENAME COLUMN attached_mcp_servers TO attached_tools;
ALTER TABLE skills RENAME COLUMN attached_mcp_servers TO attached_tools;
