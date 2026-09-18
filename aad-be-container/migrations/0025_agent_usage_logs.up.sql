-- Migration 0025: Agent Usage & Telemetry Analytics

CREATE TABLE IF NOT EXISTS agent_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    agent_version VARCHAR(50) NOT NULL,
    caller_identity VARCHAR(255),
    tool_calls JSONB NOT NULL DEFAULT '[]'::jsonb,
    token_metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    guardrail_events JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_usage_logs_agent_id ON agent_usage_logs(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_usage_logs_created_at ON agent_usage_logs(created_at DESC);
