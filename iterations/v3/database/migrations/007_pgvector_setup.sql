-- @darianrosebrook
-- Migration: Enable pgvector and create HNSW indices for multimodal RAG
-- Version: 007
-- Date: October 18, 2025
-- Description: Enable vector similarity search via pgvector extension with HNSW indices

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create HNSW index for e5-small-v2 embeddings (semantic text search)
-- Cosine similarity for normalized vectors
CREATE INDEX IF NOT EXISTS idx_block_vectors_e5_small_v2_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id = 'e5-small-v2';

-- Create HNSW index for CLIP-ViT-B/32 embeddings (image/diagram search)
-- Inner product similarity for normalized CLIP embeddings
CREATE INDEX IF NOT EXISTS idx_block_vectors_clip_vit_b32_hnsw
  ON block_vectors USING hnsw (vec vector_ip_ops)
  WHERE model_id = 'clip-vit-b32';

-- Create HNSW index for multilingual embeddings (e5-multilingual-large)
-- Cosine similarity
CREATE INDEX IF NOT EXISTS idx_block_vectors_e5_multilingual_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id = 'e5-multilingual-large';

-- Create HNSW index for all other models (default cosine similarity)
-- This catches any new models added to the registry
CREATE INDEX IF NOT EXISTS idx_block_vectors_generic_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id NOT IN ('e5-small-v2', 'clip-vit-b32', 'e5-multilingual-large');

-- Create composite index for queries by model_id + modality (for filtering)
CREATE INDEX IF NOT EXISTS idx_block_vectors_model_modality
  ON block_vectors (model_id, modality);

-- Create index for project_scope filtering (row-level visibility)
CREATE INDEX IF NOT EXISTS idx_segments_project_scope
  ON segments (project_scope);

-- Create index for document lookup by sha256 (deduplication)
CREATE INDEX IF NOT EXISTS idx_documents_sha256
  ON documents (sha256);

-- Create index for search logs audit trail
CREATE INDEX IF NOT EXISTS idx_search_logs_created_at
  ON search_logs (created_at DESC);

-- Add HNSW index parameters via comment (documentation)
COMMENT ON INDEX idx_block_vectors_e5_small_v2_hnsw IS 
  'HNSW index for e5-small-v2 embeddings (768 dimensions, cosine similarity)';

COMMENT ON INDEX idx_block_vectors_clip_vit_b32_hnsw IS 
  'HNSW index for CLIP-ViT-B/32 embeddings (512 dimensions, inner product)';

-- Verify pgvector is enabled
SELECT 
  extname,
  extversion
FROM pg_extension
WHERE extname = 'vector';
