-- Migration 0007: Add guardrail_config column to agents table
ALTER TABLE agents ADD COLUMN IF NOT EXISTS guardrail_config JSONB;
