-- Migration 0001 Down: Drop Core Storage Tables
DROP INDEX IF EXISTS idx_agents_traits;
DROP INDEX IF EXISTS idx_agents_tags;
DROP INDEX IF EXISTS idx_agents_name;

DROP TABLE IF EXISTS executions CASCADE;
DROP TABLE IF EXISTS agent_revisions CASCADE;
DROP TABLE IF EXISTS agents CASCADE;
