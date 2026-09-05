CREATE TABLE IF NOT EXISTS benches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    filesystem_path TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_benches_owner_id ON benches (owner_id);

-- Backfill default benches for any existing owners in threads
INSERT INTO benches (id, owner_id, name, description, filesystem_path)
SELECT gen_random_uuid(), owner_id, 'Default Bench', 'Auto-created default bench for historical threads', '/tmp/workspace/benches/' || owner_id
FROM (SELECT DISTINCT owner_id FROM threads) t
ON CONFLICT DO NOTHING;

-- Add bench_id column to threads
ALTER TABLE threads ADD COLUMN IF NOT EXISTS bench_id UUID;

-- Backfill threads.bench_id matching owner's bench
UPDATE threads SET bench_id = benches.id
FROM benches
WHERE threads.owner_id = benches.owner_id AND threads.bench_id IS NULL;

-- Enforce NOT NULL and foreign key constraint
ALTER TABLE threads ALTER COLUMN bench_id SET NOT NULL;
ALTER TABLE threads ADD CONSTRAINT fk_threads_bench_id FOREIGN KEY (bench_id) REFERENCES benches(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_threads_bench_id ON threads (bench_id);
