-- Migration 0027: Entity Embeddings Reverse References
ALTER TABLE entity_embeddings
    ADD COLUMN IF NOT EXISTS origin_id UUID,
    ADD COLUMN IF NOT EXISTS origin_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS origin_uri VARCHAR(255),
    ADD COLUMN IF NOT EXISTS origin_name VARCHAR(255);

-- Backfill existing rows
UPDATE entity_embeddings
SET origin_id = entity_id,
    origin_type = entity_type,
    origin_uri = '/' || entity_type || '/' || entity_id::text
WHERE origin_id IS NULL;

-- Index reverse references
CREATE INDEX IF NOT EXISTS idx_entity_embeddings_origin ON entity_embeddings(origin_id, origin_type);
