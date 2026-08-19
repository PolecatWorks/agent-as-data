ALTER TABLE agents ALTER COLUMN current_version TYPE VARCHAR(64) USING (current_version || '.0.0')::VARCHAR(64);
ALTER TABLE agents ALTER COLUMN current_version SET DEFAULT '1.0.0';

ALTER TABLE agent_revisions ALTER COLUMN version TYPE VARCHAR(64) USING (version || '.0.0')::VARCHAR(64);
ALTER TABLE agent_revisions ALTER COLUMN version SET DEFAULT '1.0.0';

ALTER TABLE executions ALTER COLUMN agent_version TYPE VARCHAR(64) USING (agent_version || '.0.0')::VARCHAR(64);

ALTER TABLE agent_test_runs ALTER COLUMN agent_version TYPE VARCHAR(64) USING (agent_version || '.0.0')::VARCHAR(64);

ALTER TABLE skills ALTER COLUMN current_version TYPE VARCHAR(64) USING (current_version || '.0.0')::VARCHAR(64);
ALTER TABLE skills ALTER COLUMN current_version SET DEFAULT '1.0.0';

ALTER TABLE trait_contracts ALTER COLUMN version TYPE VARCHAR(64) USING (version || '.0.0')::VARCHAR(64);
ALTER TABLE trait_contracts ALTER COLUMN version SET DEFAULT '1.0.0';
