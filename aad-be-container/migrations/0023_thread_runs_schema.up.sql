CREATE TABLE IF NOT EXISTS thread_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id UUID NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    bench_id UUID NOT NULL REFERENCES benches(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'running',
    current_phase TEXT NOT NULL DEFAULT 'thinking',
    active_tool_name TEXT,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_thread_runs_thread_id ON thread_runs (thread_id);
CREATE INDEX IF NOT EXISTS idx_thread_runs_bench_id ON thread_runs (bench_id);
CREATE INDEX IF NOT EXISTS idx_thread_runs_status ON thread_runs (status);
