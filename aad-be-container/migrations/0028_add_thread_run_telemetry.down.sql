-- Down Migration
DROP INDEX IF EXISTS idx_thread_runs_originating_message_id;
ALTER TABLE thread_runs
    DROP COLUMN IF EXISTS originating_message_id,
    DROP COLUMN IF EXISTS model,
    DROP COLUMN IF EXISTS prompt_tokens,
    DROP COLUMN IF EXISTS completion_tokens,
    DROP COLUMN IF EXISTS total_tokens,
    DROP COLUMN IF EXISTS tools_invoked,
    DROP COLUMN IF EXISTS duration_ms;
