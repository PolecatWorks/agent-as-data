-- Migration 0002 Down: Drop Knowledge & RAG/Graph System Tables
DROP INDEX IF EXISTS idx_knowledge_tuples_spo;
DROP INDEX IF EXISTS idx_knowledge_embeddings_vector;

DROP TABLE IF EXISTS knowledge_tuples CASCADE;
DROP TABLE IF EXISTS knowledge_embeddings CASCADE;
DROP TABLE IF EXISTS knowledge_nodes CASCADE;
