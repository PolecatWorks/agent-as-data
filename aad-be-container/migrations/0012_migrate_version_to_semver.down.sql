ALTER TABLE agents ALTER COLUMN current_version TYPE INTEGER USING split_part(current_version, '.', 1)::INTEGER;
ALTER TABLE agents ALTER COLUMN current_version SET DEFAULT 1;

ALTER TABLE agent_revisions ALTER COLUMN version TYPE INTEGER USING split_part(version, '.', 1)::INTEGER;
ALTER TABLE agent_revisions ALTER COLUMN version SET DEFAULT 1;

ALTER TABLE executions ALTER COLUMN agent_version TYPE INTEGER USING split_part(agent_version, '.', 1)::INTEGER;

ALTER TABLE agent_test_runs ALTER COLUMN agent_version TYPE INTEGER USING split_part(agent_version, '.', 1)::INTEGER;

ALTER TABLE skills ALTER COLUMN current_version TYPE INTEGER USING split_part(current_version, '.', 1)::INTEGER;
ALTER TABLE skills ALTER COLUMN current_version SET DEFAULT 1;

ALTER TABLE trait_contracts ALTER COLUMN version TYPE INTEGER USING split_part(version, '.', 1)::INTEGER;
ALTER TABLE trait_contracts ALTER COLUMN version SET DEFAULT 1;
