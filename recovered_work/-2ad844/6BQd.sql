-- Migration: External Knowledge Base Schema (Wikidata + WordNet)
-- Version: 009
-- Description: Add external knowledge entities with model-agnostic vectors, cross-references, and provenance
-- Author: @darianrosebrook
-- Date: 2025-10-19
--
-- This migration creates:
-- - External knowledge entities table (Wikidata + WordNet)
-- - Model-agnostic vector storage per entity
-- - Cross-reference relationships between knowledge sources
-- - Usage tracking and decay mechanisms
-- - Helper functions for knowledge queries

BEGIN;

-- ============================================================================
-- EMBEDDING MODEL REGISTRY (shared with multimodal if present)
-- ============================================================================

CREATE TABLE IF NOT EXISTS embedding_models (
  id TEXT PRIMARY KEY,           -- e.g., 'e5-small-v2', 'kb-text-default'
  modality TEXT NOT NULL,        -- 'text'|'image'|'audio'
  dim INTEGER NOT NULL,
  metric TEXT NOT NULL DEFAULT 'cosine',
  active BOOLEAN DEFAULT TRUE,
  CONSTRAINT valid_modality CHECK (modality IN ('text','image','audio')),
  CONSTRAINT valid_metric CHECK (metric IN ('cosine','ip','l2'))
);

-- Insert default KB text model
INSERT INTO embedding_models (id, modality, dim, metric, active)
VALUES ('kb-text-default', 'text', 768, 'cosine', true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- EXTERNAL KNOWLEDGE ENTITIES
-- ============================================================================

CREATE TABLE external_knowledge_entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source VARCHAR(20) NOT NULL CHECK (source IN ('wikidata','wordnet')),
  entity_key VARCHAR(255) NOT NULL,      -- QID/LID/synset key
  canonical_name TEXT NOT NULL,          -- surface form (by lang policy)
  lang VARCHAR(16),                      -- BCP47 tag, e.g., 'en'
  entity_type VARCHAR(50),               -- 'lexeme','item','synset',...
  properties JSONB NOT NULL,             -- source-specific structure
  confidence NUMERIC(3,2) DEFAULT 1.00,
  usage_count INTEGER DEFAULT 0,
  usage_decay FLOAT DEFAULT 1.0,         -- multiplicative decay factor
  last_accessed TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  dump_version TEXT,                     -- e.g., 'wikidata-2025-09-24'
  toolchain TEXT,                        -- versions of parsers/embedders
  license TEXT,                          -- 'CC0', 'WordNet-3.1'
  CONSTRAINT valid_confidence CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT valid_usage_decay CHECK (usage_decay >= 0.0)
);

-- Unique constraint on source + entity_key
CREATE UNIQUE INDEX uq_eke_source_key ON external_knowledge_entities(source, entity_key);

-- Indexes for performance
CREATE INDEX idx_eke_source ON external_knowledge_entities(source);
CREATE INDEX idx_eke_key ON external_knowledge_entities(entity_key);
CREATE INDEX idx_eke_canonical ON external_knowledge_entities(canonical_name);
CREATE INDEX idx_eke_lang ON external_knowledge_entities(lang) WHERE lang IS NOT NULL;
CREATE INDEX idx_eke_usage ON external_knowledge_entities(usage_count DESC);
CREATE INDEX idx_eke_decay ON external_knowledge_entities(usage_decay DESC);

-- Trigram index for fuzzy matching (requires pg_trgm extension)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_eke_name_trgm ON external_knowledge_entities 
USING gin (canonical_name gin_trgm_ops);

-- ============================================================================
-- MODEL-AGNOSTIC VECTOR STORAGE
-- ============================================================================

CREATE TABLE knowledge_vectors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  model_id TEXT NOT NULL REFERENCES embedding_models(id),
  vec VECTOR,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(entity_id, model_id)
);

-- Index for entity lookup
CREATE INDEX idx_kv_entity ON knowledge_vectors(entity_id);
CREATE INDEX idx_kv_model ON knowledge_vectors(model_id);

-- HNSW index for default KB text model (cosine similarity)
-- Note: Additional HNSW indexes should be created per active model
CREATE INDEX idx_kv_kb_text_default_cos ON knowledge_vectors 
USING hnsw (vec vector_cosine_ops)
WITH (m = 16, ef_construction = 64)
WHERE model_id = 'kb-text-default' AND vec IS NOT NULL;

-- ============================================================================
-- CROSS-REFERENCE RELATIONSHIPS
-- ============================================================================

CREATE TABLE knowledge_relationships (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  target_entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  relationship_type VARCHAR(50) NOT NULL,    -- 'synonym','hypernym','translation','equivalent'
  confidence NUMERIC(3,2) DEFAULT 0.80,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT valid_rel_confidence CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT no_self_reference CHECK (source_entity_id != target_entity_id)
);

-- Indexes for relationship queries
CREATE INDEX idx_kr_src ON knowledge_relationships(source_entity_id);
CREATE INDEX idx_kr_tgt ON knowledge_relationships(target_entity_id);
CREATE INDEX idx_kr_type ON knowledge_relationships(relationship_type);
CREATE INDEX idx_kr_confidence ON knowledge_relationships(confidence DESC);

-- Index for bidirectional lookups
CREATE INDEX idx_kr_both ON knowledge_relationships(source_entity_id, target_entity_id);

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Record usage with decay boost
CREATE OR REPLACE FUNCTION record_knowledge_usage(p_entity_id UUID) 
RETURNS VOID AS $$
BEGIN
  UPDATE external_knowledge_entities
     SET usage_count = usage_count + 1,
         usage_decay = LEAST(usage_decay * 1.02, 10.0),
         last_accessed = NOW()
   WHERE id = p_entity_id;
END;
$$ LANGUAGE plpgsql;

-- Apply decay to stale entities (run periodically)
CREATE OR REPLACE FUNCTION apply_knowledge_decay(p_decay_factor FLOAT DEFAULT 0.95)
RETURNS INTEGER AS $$
DECLARE
  affected_count INTEGER;
BEGIN
  UPDATE external_knowledge_entities
     SET usage_decay = usage_decay * p_decay_factor
   WHERE last_accessed < NOW() - INTERVAL '30 days'
     AND usage_decay > 0.1;
  
  GET DIAGNOSTICS affected_count = ROW_COUNT;
  RETURN affected_count;
END;
$$ LANGUAGE plpgsql;

-- Semantic search for external knowledge
CREATE OR REPLACE FUNCTION kb_semantic_search(
  p_query_vec VECTOR,
  p_model_id TEXT,
  p_source VARCHAR(20) DEFAULT NULL,
  p_limit INTEGER DEFAULT 10,
  p_min_confidence NUMERIC DEFAULT 0.5
)
RETURNS TABLE(
  entity_id UUID,
  source VARCHAR(20),
  entity_key VARCHAR(255),
  canonical_name TEXT,
  entity_type VARCHAR(50),
  properties JSONB,
  similarity FLOAT,
  confidence NUMERIC,
  usage_count INTEGER
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    e.id as entity_id,
    e.source,
    e.entity_key,
    e.canonical_name,
    e.entity_type,
    e.properties,
    (1 - (v.vec <=> p_query_vec))::FLOAT as similarity,
    e.confidence,
    e.usage_count
  FROM knowledge_vectors v
  JOIN external_knowledge_entities e ON v.entity_id = e.id
  WHERE v.model_id = p_model_id
    AND v.vec IS NOT NULL
    AND (p_source IS NULL OR e.source = p_source)
    AND e.confidence >= p_min_confidence
  ORDER BY v.vec <=> p_query_vec
  LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Get related entities via relationships (with depth limit)
CREATE OR REPLACE FUNCTION kb_get_related(
  p_entity_id UUID,
  p_relationship_types VARCHAR(50)[] DEFAULT NULL,
  p_max_depth INTEGER DEFAULT 2
)
RETURNS TABLE(
  entity_id UUID,
  canonical_name TEXT,
  relationship_type VARCHAR(50),
  depth INTEGER,
  confidence NUMERIC
) AS $$
BEGIN
  RETURN QUERY
  WITH RECURSIVE related_entities AS (
    -- Base case: direct relationships
    SELECT 
      e.id as entity_id,
      e.canonical_name,
      r.relationship_type,
      1 as depth,
      r.confidence
    FROM knowledge_relationships r
    JOIN external_knowledge_entities e ON r.target_entity_id = e.id
    WHERE r.source_entity_id = p_entity_id
      AND (p_relationship_types IS NULL OR r.relationship_type = ANY(p_relationship_types))
    
    UNION
    
    -- Recursive case: follow relationships
    SELECT 
      e.id as entity_id,
      e.canonical_name,
      r.relationship_type,
      re.depth + 1 as depth,
      (re.confidence * r.confidence)::NUMERIC(3,2) as confidence
    FROM related_entities re
    JOIN knowledge_relationships r ON r.source_entity_id = re.entity_id
    JOIN external_knowledge_entities e ON r.target_entity_id = e.id
    WHERE re.depth < p_max_depth
      AND (p_relationship_types IS NULL OR r.relationship_type = ANY(p_relationship_types))
  )
  SELECT DISTINCT ON (entity_id)
    entity_id,
    canonical_name,
    relationship_type,
    depth,
    confidence
  FROM related_entities
  ORDER BY entity_id, depth ASC, confidence DESC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Fuzzy search by canonical name
CREATE OR REPLACE FUNCTION kb_fuzzy_search(
  p_query TEXT,
  p_source VARCHAR(20) DEFAULT NULL,
  p_limit INTEGER DEFAULT 10,
  p_similarity_threshold FLOAT DEFAULT 0.3
)
RETURNS TABLE(
  entity_id UUID,
  canonical_name TEXT,
  similarity FLOAT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    e.id as entity_id,
    e.canonical_name,
    similarity(e.canonical_name, p_query) as similarity
  FROM external_knowledge_entities e
  WHERE (p_source IS NULL OR e.source = p_source)
    AND similarity(e.canonical_name, p_query) > p_similarity_threshold
  ORDER BY similarity DESC
  LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Get knowledge statistics
CREATE OR REPLACE FUNCTION kb_get_stats()
RETURNS TABLE(
  source VARCHAR(20),
  total_entities BIGINT,
  total_vectors BIGINT,
  total_relationships BIGINT,
  avg_confidence NUMERIC,
  avg_usage_count NUMERIC,
  last_updated TIMESTAMPTZ
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    e.source,
    COUNT(DISTINCT e.id) as total_entities,
    COUNT(DISTINCT v.id) as total_vectors,
    COUNT(DISTINCT r.id) as total_relationships,
    AVG(e.confidence) as avg_confidence,
    AVG(e.usage_count) as avg_usage_count,
    MAX(e.created_at) as last_updated
  FROM external_knowledge_entities e
  LEFT JOIN knowledge_vectors v ON v.entity_id = e.id
  LEFT JOIN knowledge_relationships r ON r.source_entity_id = e.id OR r.target_entity_id = e.id
  GROUP BY e.source;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE external_knowledge_entities IS 
'External knowledge sources (Wikidata lexemes/items and WordNet synsets) with provenance and usage tracking';

COMMENT ON TABLE knowledge_vectors IS 
'Model-agnostic vector embeddings for external knowledge entities, supporting multiple embedding models';

COMMENT ON TABLE knowledge_relationships IS 
'Cross-reference relationships between knowledge entities (synonyms, hypernyms, translations, equivalents)';

COMMENT ON FUNCTION record_knowledge_usage IS 
'Update usage statistics and boost decay factor for frequently accessed entities';

COMMENT ON FUNCTION apply_knowledge_decay IS 
'Apply decay to stale entities not accessed in 30+ days to manage storage';

COMMENT ON FUNCTION kb_semantic_search IS 
'Semantic similarity search over external knowledge using vector embeddings';

COMMENT ON FUNCTION kb_get_related IS 
'Retrieve related entities via relationship graph with configurable depth and relationship types';

COMMENT ON FUNCTION kb_fuzzy_search IS 
'Fuzzy text search over canonical names using trigram similarity';

COMMENT ON FUNCTION kb_get_stats IS 
'Get aggregate statistics for external knowledge sources';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Semantic search for "database" concept
-- SELECT * FROM kb_semantic_search('[0.1, 0.2, ...]'::vector, 'kb-text-default', NULL, 10, 0.6);

-- Example 2: Get WordNet synonyms and hypernyms
-- SELECT * FROM kb_get_related('uuid-here', ARRAY['synonym', 'hypernym'], 2);

-- Example 3: Fuzzy search for "databse" (typo)
-- SELECT * FROM kb_fuzzy_search('databse', 'wordnet', 5, 0.3);

-- Example 4: Record usage after successful disambiguation
-- SELECT record_knowledge_usage('uuid-here');

-- Example 5: Get knowledge base statistics
-- SELECT * FROM kb_get_stats();

-- Example 6: Apply decay to stale entities
-- SELECT apply_knowledge_decay(0.95);

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Additional HNSW indexes should be created for each active embedding model:
--    CREATE INDEX idx_kv_{model_id}_cos ON knowledge_vectors 
--    USING hnsw (vec vector_cosine_ops)
--    WHERE model_id = '{model_id}' AND vec IS NOT NULL;

-- 2. Schedule periodic decay application (e.g., via pg_cron):
--    SELECT cron.schedule('apply-kb-decay', '0 0 * * 0', 'SELECT apply_knowledge_decay(0.95);');

-- 3. Monitor index bloat and rebuild HNSW indexes periodically if needed:
--    REINDEX INDEX CONCURRENTLY idx_kv_kb_text_default_cos;

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

