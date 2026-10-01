DROP INDEX IF EXISTS idx_entity_embeddings_origin;
ALTER TABLE entity_embeddings
    DROP COLUMN IF EXISTS origin_id,
    DROP COLUMN IF EXISTS origin_type,
    DROP COLUMN IF EXISTS origin_uri,
    DROP COLUMN IF EXISTS origin_name;
