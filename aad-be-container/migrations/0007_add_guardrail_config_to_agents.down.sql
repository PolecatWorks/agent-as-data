-- Migration 0007: Drop guardrail_config column from agents table
ALTER TABLE agents DROP COLUMN IF EXISTS guardrail_config;
