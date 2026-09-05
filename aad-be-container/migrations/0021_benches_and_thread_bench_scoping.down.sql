ALTER TABLE threads DROP CONSTRAINT IF EXISTS fk_threads_bench_id;
DROP INDEX IF EXISTS idx_threads_bench_id;
ALTER TABLE threads DROP COLUMN IF EXISTS bench_id;
DROP TABLE IF EXISTS benches;
