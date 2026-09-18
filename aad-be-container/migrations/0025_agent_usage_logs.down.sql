-- Migration 0025 Down: Agent Usage & Telemetry Analytics
DROP INDEX IF EXISTS idx_agent_usage_logs_created_at;
DROP INDEX IF EXISTS idx_agent_usage_logs_agent_id;
DROP TABLE IF EXISTS agent_usage_logs CASCADE;
