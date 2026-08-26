-- Migration: 0019_allow_null_or_skill_execution_fk.up.sql
-- Allow executions to track skill executions and unlinked executions without strict agent_id FK constraint
ALTER TABLE executions DROP CONSTRAINT IF EXISTS executions_agent_id_fkey;
ALTER TABLE executions ALTER COLUMN agent_id DROP NOT NULL;
