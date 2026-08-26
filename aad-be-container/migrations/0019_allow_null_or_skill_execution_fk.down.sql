-- Migration: 0019_allow_null_or_skill_execution_fk.down.sql
-- Restore strict executions agent_id foreign key constraint
ALTER TABLE executions ADD CONSTRAINT executions_agent_id_fkey FOREIGN KEY (agent_id) REFERENCES agents(id);
