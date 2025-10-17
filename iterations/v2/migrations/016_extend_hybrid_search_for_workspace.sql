-- Migration: Extend Hybrid Search for Workspace Files
-- Version: 016
-- Description: Extend existing hybrid_search_index view to include workspace files and external knowledge
-- Author: @darianrosebrook
-- Date: 2025-01-27
--
-- This migration extends the existing hybrid search infrastructure by:
-- - Adding workspace_file entity type to hybrid_search_index view
-- - Adding EXTERNAL_KNOWLEDGE entity type to entity_type enum
-- - Recreating materialized view with new union
-- - Maintaining existing HNSW indexes

BEGIN;

-- ============================================================================
-- EXTEND ENTITY TYPE ENUM
-- ============================================================================

-- Add new entity types for workspace files and external knowledge
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'EXTERNAL_KNOWLEDGE';

-- ============================================================================
-- EXTEND HYBRID SEARCH VIEW
-- ============================================================================

-- Drop existing view to recreate with new unions
DROP VIEW IF EXISTS hybrid_search_index CASCADE;

-- Recreate hybrid search index with workspace files and external knowledge
CREATE VIEW hybrid_search_index AS
-- Existing: Agent capabilities
SELECT 
    'agent_capability'::VARCHAR(50) as entity_type,
    id,
    agent_id as parent_id,
    capability_name as name,
    canonical_name,
    embedding,
    confidence,
    tenant_id,
    metadata,
    last_updated as updated_at
FROM agent_capabilities_graph
WHERE embedding IS NOT NULL
  AND capability_type = 'CAPABILITY'

UNION ALL

-- Existing: CAWS provenance
SELECT
    'caws_provenance'::VARCHAR(50) as entity_type,
    id,
    parent_entity_id as parent_id,
    entity_id as name,
    entity_id as canonical_name,
    embedding,
    GREATEST(
        COALESCE(evidence_completeness, 0),
        COALESCE(budget_adherence, 0),
        COALESCE(gate_integrity, 0),
        COALESCE(provenance_clarity, 0)
    ) as confidence,
    NULL as tenant_id,
    metadata,
    created_at as updated_at
FROM caws_provenance_graph
WHERE embedding IS NOT NULL

UNION ALL

-- NEW: Workspace files (stored as TECHNOLOGY capability type)
SELECT
    'workspace_file'::VARCHAR(50) as entity_type,
    id,
    NULL as parent_id,
    (metadata->>'file_path')::TEXT as name,
    (metadata->>'file_path')::TEXT as canonical_name,
    embedding,
    1.0::DECIMAL(3,2) as confidence,
    NULL as tenant_id,
    metadata,
    last_updated as updated_at
FROM agent_capabilities_graph
WHERE capability_type = 'TECHNOLOGY'
  AND metadata->>'source' = 'workspace_file'
  AND embedding IS NOT NULL

UNION ALL

-- NEW: External knowledge (Wikidata, WordNet)
SELECT
    'external_knowledge'::VARCHAR(50) as entity_type,
    id,
    NULL as parent_id,
    capability_name as name,
    canonical_name,
    embedding,
    confidence,
    NULL as tenant_id,
    metadata,
    last_updated as updated_at
FROM agent_capabilities_graph
WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
  AND embedding IS NOT NULL

UNION ALL

-- Agent profiles (for semantic agent discovery)
SELECT
    'agent'::VARCHAR(50) as entity_type,
    id::UUID,
    NULL as parent_id,
    name,
    name as canonical_name,
    NULL as embedding, -- Will add embeddings to agent_profiles in future
    NULL as confidence,
    tenant_id,
    NULL as metadata,
    updated_at
FROM agent_profiles;

-- ============================================================================
-- RECREATE MATERIALIZED VIEW
-- ============================================================================

-- Drop existing materialized view
DROP MATERIALIZED VIEW IF EXISTS hybrid_search_materialized CASCADE;

-- Create new materialized view with extended unions
CREATE MATERIALIZED VIEW hybrid_search_materialized AS
SELECT *
FROM hybrid_search_index;

-- Recreate indexes for performance
CREATE INDEX idx_hybrid_search_type ON hybrid_search_materialized (entity_type);
CREATE INDEX idx_hybrid_search_tenant ON hybrid_search_materialized (tenant_id);

-- HNSW index for fast semantic search (reuse existing configuration)
CREATE INDEX idx_hybrid_search_embedding ON hybrid_search_materialized 
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64)
WHERE embedding IS NOT NULL;

-- Additional indexes for new entity types
CREATE INDEX idx_hybrid_search_workspace_source ON hybrid_search_materialized 
USING GIN ((metadata->>'source')) 
WHERE entity_type = 'workspace_file';

CREATE INDEX idx_hybrid_search_external_source ON hybrid_search_materialized 
USING GIN ((metadata->>'source')) 
WHERE entity_type = 'external_knowledge';

-- ============================================================================
-- EXTEND HYBRID SEARCH FUNCTION
-- ============================================================================

-- Update hybrid_search function to include new entity types
CREATE OR REPLACE FUNCTION hybrid_search(
    query_embedding vector(768),
    query_text TEXT DEFAULT NULL,
    max_results INTEGER DEFAULT 10,
    include_graph_hops INTEGER DEFAULT 2,
    entity_types VARCHAR(50)[] DEFAULT NULL,
    p_tenant_id VARCHAR(255) DEFAULT NULL,
    min_confidence DECIMAL DEFAULT 0.5
) RETURNS TABLE(
    entity_id UUID,
    entity_type VARCHAR(50),
    name VARCHAR(500),
    relevance_score DECIMAL,
    source VARCHAR(20),
    hop_distance INTEGER,
    parent_id VARCHAR(255)
) AS $$
BEGIN
    RETURN QUERY
    WITH vector_results AS (
        -- Step 1: Vector similarity search across all entity types
        SELECT 
            id as entity_id,
            entity_type,
            name,
            (1 - (embedding <=> query_embedding))::DECIMAL(5,4) as relevance_score,
            'vector'::VARCHAR(20) as source,
            0 as hop_distance,
            parent_id
        FROM hybrid_search_materialized
        WHERE embedding IS NOT NULL
            AND (entity_types IS NULL OR entity_type = ANY(entity_types))
            AND (p_tenant_id IS NULL OR tenant_id = p_tenant_id OR tenant_id IS NULL)
            AND (confidence IS NULL OR confidence >= min_confidence)
        ORDER BY embedding <=> query_embedding
        LIMIT max_results
    ),
    graph_results AS (
        -- Step 2: Graph traversal from top vector results (only for agent capabilities)
        SELECT DISTINCT
            acg.id as entity_id,
            'agent_capability'::VARCHAR(50) as entity_type,
            acg.capability_name as name,
            (ar.confidence * ar.strength * (1.0 / (trav.hop_distance + 1)))::DECIMAL(5,4) as relevance_score,
            'graph'::VARCHAR(20) as source,
            trav.hop_distance,
            acg.agent_id as parent_id
        FROM vector_results vr
        CROSS JOIN LATERAL (
            SELECT * FROM traverse_agent_relationships(
                vr.parent_id,
                include_graph_hops,
                min_confidence
            )
        ) trav
        JOIN agent_capabilities_graph acg ON acg.agent_id = trav.agent_id
        JOIN agent_relationships ar ON (
            ar.source_agent_id = vr.parent_id 
            AND ar.target_agent_id = trav.agent_id
        )
        WHERE include_graph_hops > 0
            AND vr.entity_type = 'agent_capability'
            AND (p_tenant_id IS NULL OR acg.tenant_id = p_tenant_id)
            AND ar.confidence >= min_confidence
        LIMIT max_results
    )
    -- Combine and rank results
    SELECT * FROM vector_results
    UNION ALL
    SELECT * FROM graph_results
    ORDER BY relevance_score DESC, hop_distance ASC
    LIMIT max_results * 2; -- Return more results to account for both sources
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- ADDITIONAL SEARCH FUNCTIONS
-- ============================================================================

-- Function to find similar workspace files
CREATE OR REPLACE FUNCTION find_similar_workspace_files(
    target_embedding vector(768),
    file_types TEXT[] DEFAULT NULL,
    max_results INTEGER DEFAULT 10,
    min_confidence DECIMAL DEFAULT 0.7
) RETURNS TABLE(
    file_id UUID,
    file_path TEXT,
    file_type TEXT,
    similarity_score DECIMAL,
    last_modified TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        hsm.id as file_id,
        (hsm.metadata->>'file_path')::TEXT as file_path,
        (hsm.metadata->>'file_type')::TEXT as file_type,
        (1 - (hsm.embedding <=> target_embedding))::DECIMAL(5,4) as similarity_score,
        hsm.updated_at as last_modified
    FROM hybrid_search_materialized hsm
    WHERE hsm.entity_type = 'workspace_file'
        AND hsm.embedding IS NOT NULL
        AND (file_types IS NULL OR (hsm.metadata->>'file_type') = ANY(file_types))
        AND hsm.confidence >= min_confidence
    ORDER BY hsm.embedding <=> target_embedding
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to find similar external knowledge
CREATE OR REPLACE FUNCTION find_similar_external_knowledge(
    target_embedding vector(768),
    knowledge_sources TEXT[] DEFAULT NULL,
    max_results INTEGER DEFAULT 10,
    min_confidence DECIMAL DEFAULT 0.7
) RETURNS TABLE(
    knowledge_id UUID,
    entity_name TEXT,
    source TEXT,
    similarity_score DECIMAL,
    metadata JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        hsm.id as knowledge_id,
        hsm.name as entity_name,
        (hsm.metadata->>'source')::TEXT as source,
        (1 - (hsm.embedding <=> target_embedding))::DECIMAL(5,4) as similarity_score,
        hsm.metadata
    FROM hybrid_search_materialized hsm
    WHERE hsm.entity_type = 'external_knowledge'
        AND hsm.embedding IS NOT NULL
        AND (knowledge_sources IS NULL OR (hsm.metadata->>'source') = ANY(knowledge_sources))
        AND hsm.confidence >= min_confidence
    ORDER BY hsm.embedding <=> target_embedding
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- REFRESH FUNCTION
-- ============================================================================

-- Function to refresh hybrid search materialized view
CREATE OR REPLACE FUNCTION refresh_hybrid_search_index()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY hybrid_search_materialized;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON VIEW hybrid_search_index IS 'Extended unified view of all searchable entities including workspace files and external knowledge';

COMMENT ON MATERIALIZED VIEW hybrid_search_materialized IS 'Materialized version of extended hybrid search index with HNSW index for performance';

COMMENT ON FUNCTION find_similar_workspace_files IS 'Semantic search for workspace files using vector embeddings';

COMMENT ON FUNCTION find_similar_external_knowledge IS 'Semantic search for external knowledge (Wikidata, WordNet) using vector embeddings';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Search across all entity types
-- SELECT * FROM hybrid_search(
--     '[your_query_embedding]'::vector,
--     'TypeScript code generation',
--     max_results => 20,
--     include_graph_hops => 2,
--     entity_types => ARRAY['workspace_file', 'agent_capability', 'external_knowledge'],
--     p_tenant_id => NULL,
--     min_confidence => 0.7
-- );

-- Example 2: Find similar workspace files
-- SELECT * FROM find_similar_workspace_files(
--     '[file_embedding]'::vector,
--     ARRAY['.ts', '.js', '.md'],
--     max_results => 10,
--     min_confidence => 0.7
-- );

-- Example 3: Find similar external knowledge
-- SELECT * FROM find_similar_external_knowledge(
--     '[query_embedding]'::vector,
--     ARRAY['wikidata', 'wordnet'],
--     max_results => 10,
--     min_confidence => 0.7
-- );

COMMIT;

-- ============================================================================
-- POST-MIGRATION SETUP
-- ============================================================================

-- Populate materialized view (run after first embeddings are generated)
-- REFRESH MATERIALIZED VIEW hybrid_search_materialized;

-- Schedule periodic refresh (use pg_cron or application scheduler)
-- SELECT cron.schedule('refresh-hybrid-search', '*/30 * * * *', 'SELECT refresh_hybrid_search_index();');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
