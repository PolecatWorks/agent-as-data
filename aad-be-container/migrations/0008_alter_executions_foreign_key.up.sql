-- Migration 0008: Alter executions foreign key to delete cascade
ALTER TABLE executions
DROP CONSTRAINT IF EXISTS executions_agent_id_fkey,
ADD CONSTRAINT executions_agent_id_fkey
FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE;
