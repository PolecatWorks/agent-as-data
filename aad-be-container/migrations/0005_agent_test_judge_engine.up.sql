-- Migration 0005: Probabilistic Unit Testing & LLM-as-a-Judge Engine

-- 1. Add judge_threshold to agents table
ALTER TABLE agents ADD COLUMN IF NOT EXISTS judge_threshold FLOAT NOT NULL DEFAULT 0.8;

-- 2. Agent Test Suites Table
CREATE TABLE IF NOT EXISTS agent_test_suites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    test_cases JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Agent Test Runs Table (Audit log for deterministic & Judge evaluation scores)
CREATE TABLE IF NOT EXISTS agent_test_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    agent_version INT NOT NULL,
    suite_id UUID REFERENCES agent_test_suites(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL,
    deterministic_results JSONB NOT NULL DEFAULT '{}'::jsonb,
    judge_evaluation JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_test_runs_agent ON agent_test_runs(agent_id, agent_version);
