-- Migration 0003 Down: Drop Agent Embeddings & Managed Skills Tables
DROP INDEX IF EXISTS idx_skills_name;

DROP TABLE IF EXISTS skills CASCADE;
DROP TABLE IF EXISTS agent_embeddings CASCADE;
