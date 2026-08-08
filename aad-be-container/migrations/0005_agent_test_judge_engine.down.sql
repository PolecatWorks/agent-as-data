-- Drop indexes
DROP INDEX IF EXISTS idx_agent_test_runs_agent;

-- Drop tables
DROP TABLE IF EXISTS agent_test_runs;
DROP TABLE IF EXISTS agent_test_suites;

-- Remove judge_threshold from agents table
ALTER TABLE agents DROP COLUMN IF EXISTS judge_threshold;
