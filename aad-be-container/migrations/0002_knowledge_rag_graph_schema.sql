-- Migration 0002: Knowledge & RAG/Graph System DDL Schema

-- Raw knowledge text nodes
CREATE TABLE IF NOT EXISTS knowledge_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic VARCHAR(255) NOT NULL,
    title VARCHAR(255),
    content TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- RAG Vector Embeddings for chunked content
CREATE TABLE IF NOT EXISTS knowledge_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    chunk_index INT NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding vector(1536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Knowledge Graph Tuples with Canonical Resolution
CREATE TABLE IF NOT EXISTS knowledge_tuples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject VARCHAR(255) NOT NULL,
    subject_canonical VARCHAR(255),
    predicate VARCHAR(255) NOT NULL,
    object VARCHAR(255) NOT NULL,
    object_canonical VARCHAR(255),
    confidence FLOAT NOT NULL DEFAULT 1.0,
    traversal_count INT NOT NULL DEFAULT 0,
    source_node_id UUID REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indices
CREATE INDEX IF NOT EXISTS idx_knowledge_embeddings_vector ON knowledge_embeddings USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_knowledge_tuples_spo ON knowledge_tuples(subject, predicate, object);
