-- Up Migration
ALTER TABLE thread_runs
    ADD COLUMN originating_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    ADD COLUMN model TEXT,
    ADD COLUMN prompt_tokens INT,
    ADD COLUMN completion_tokens INT,
    ADD COLUMN total_tokens INT,
    ADD COLUMN tools_invoked JSONB,
    ADD COLUMN duration_ms BIGINT;

CREATE INDEX idx_thread_runs_originating_message_id ON thread_runs(originating_message_id);
