ALTER TABLE agents 
ADD COLUMN IF NOT EXISTS attached_skills UUID[] NOT NULL DEFAULT '{}',
ADD COLUMN IF NOT EXISTS attached_mcp_servers UUID[] NOT NULL DEFAULT '{}',
ADD COLUMN IF NOT EXISTS attached_agents UUID[] NOT NULL DEFAULT '{}';

-- Migrate existing available_skills JSONB array of strings/UUIDs to attached_skills UUID[]
UPDATE agents
SET attached_skills = ARRAY(
    SELECT val::uuid
    FROM jsonb_array_elements_text(COALESCE(available_skills, '[]'::jsonb)) AS val
    WHERE val ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
)
WHERE available_skills IS NOT NULL;

-- Migrate existing available_agents JSONB array of UUIDs to attached_agents UUID[]
UPDATE agents
SET attached_agents = ARRAY(
    SELECT val::uuid
    FROM jsonb_array_elements_text(COALESCE(available_agents, '[]'::jsonb)) AS val
    WHERE val ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
)
WHERE available_agents IS NOT NULL;

-- Migrate existing tools JSONB array (if containing UUIDs) to attached_mcp_servers UUID[]
UPDATE agents
SET attached_mcp_servers = ARRAY(
    SELECT val::uuid
    FROM jsonb_array_elements_text(COALESCE(tools, '[]'::jsonb)) AS val
    WHERE val ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$'
)
WHERE tools IS NOT NULL;
