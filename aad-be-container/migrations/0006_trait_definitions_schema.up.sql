-- Migration 0006: Trait Definition Specification Registry DDL Schema
CREATE TABLE IF NOT EXISTS trait_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    capability_requirements TEXT[] NOT NULL DEFAULT '{}',
    behavioral_invariants TEXT[] NOT NULL DEFAULT '{}',
    evaluation_criteria TEXT[] NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    guardrails JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trait_contracts_name ON trait_contracts(name);
CREATE INDEX IF NOT EXISTS idx_trait_contracts_tags ON trait_contracts USING GIN(tags);
