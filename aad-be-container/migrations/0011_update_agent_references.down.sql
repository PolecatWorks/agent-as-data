ALTER TABLE agents 
DROP COLUMN IF EXISTS attached_skills,
DROP COLUMN IF EXISTS attached_mcp_servers,
DROP COLUMN IF EXISTS attached_agents;
