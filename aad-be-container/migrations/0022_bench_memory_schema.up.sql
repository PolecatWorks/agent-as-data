CREATE TABLE IF NOT EXISTS bench_memory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bench_id UUID NOT NULL REFERENCES benches(id) ON DELETE CASCADE,
    memory_type TEXT NOT NULL DEFAULT 'working',
    title TEXT NOT NULL DEFAULT 'Active Working Memory',
    content TEXT NOT NULL DEFAULT '',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_bench_memory_working_unique 
ON bench_memory (bench_id) WHERE memory_type = 'working';

CREATE INDEX IF NOT EXISTS idx_bench_memory_bench_type ON bench_memory (bench_id, memory_type);
