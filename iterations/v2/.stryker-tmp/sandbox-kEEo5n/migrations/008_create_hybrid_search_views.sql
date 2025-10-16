-- Migration: Create Hybrid Vector-Graph Search Architecture
-- Version: 008
-- Description: Unified search interface combining vector similarity with graph traversal
-- Author: @darianrosebrook
-- Date: 2025-10-12
--
-- This migration creates:
-- - Hybrid search views combining multiple entity types
-- - Graph traversal functions for multi-hop queries
-- - Search session tracking
-- - Performance optimization helpers

BEGIN;

-- ============================================================================
-- UNIFIED HYBRID SEARCH VIEW
-- ============================================================================

-- Create unified view combining all searchable entities

CREATE VIEW hybrid_search_index AS
-- Agent capabilities
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

UNION ALL

-- CAWS provenance

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

-- Create index on hybrid search view (materialized for performance)
CREATE MATERIALIZED VIEW hybrid_search_materialized AS
SELECT *
FROM hybrid_search_index;

CREATE INDEX idx_hybrid_search_type ON hybrid_search_materialized (entity_type);

CREATE INDEX idx_hybrid_search_tenant ON hybrid_search_materialized (tenant_id);

CREATE INDEX idx_hybrid_search_embedding ON hybrid_search_materialized USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64)
WHERE
    embedding IS NOT NULL;

-- ============================================================================
-- GRAPH TRAVERSAL FUNCTIONS
-- ============================================================================

-- Function to traverse agent relationships (multi-hop)
CREATE OR REPLACE FUNCTION traverse_agent_relationships(
    start_agent_id VARCHAR(255),
    max_hops INTEGER DEFAULT 2,
    min_confidence DECIMAL DEFAULT 0.5,
    relationship_types relationship_type[] DEFAULT NULL
) RETURNS TABLE(
    agent_id VARCHAR(255),
    agent_name VARCHAR(255),
    hop_distance INTEGER,
    relationship_path relationship_type[],
    cumulative_confidence DECIMAL,
    path UUID[]
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE agent_graph AS (
        -- Base case: Start agent
        SELECT 
            ap.id as agent_id,
            ap.name as agent_name,
            0 as hop_distance,
            ARRAY[]::relationship_type[] as relationship_path,
            1.0::DECIMAL as cumulative_confidence,
            ARRAY[ap.id::UUID] as path
        FROM agent_profiles ap
        WHERE ap.id = start_agent_id
        
        UNION ALL
        
        -- Recursive case: Follow relationships
        SELECT 
            ap.id as agent_id,
            ap.name as agent_name,
            ag.hop_distance + 1 as hop_distance,
            ag.relationship_path || ar.type as relationship_path,
            (ag.cumulative_confidence * ar.confidence)::DECIMAL as cumulative_confidence,
            ag.path || ap.id::UUID as path
        FROM agent_graph ag
        JOIN agent_relationships ar ON ar.source_agent_id = ag.agent_id
        JOIN agent_profiles ap ON ap.id = ar.target_agent_id
        WHERE ag.hop_distance < max_hops
            AND ar.confidence >= min_confidence
            AND (relationship_types IS NULL OR ar.type = ANY(relationship_types))
            AND NOT ap.id::UUID = ANY(ag.path) -- Prevent cycles
    )
    SELECT 
        agent_id,
        agent_name,
        hop_distance,
        relationship_path,
        cumulative_confidence,
        path
    FROM agent_graph
    WHERE hop_distance > 0 -- Exclude start agent
    ORDER BY hop_distance, cumulative_confidence DESC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to find shortest path between two agents
CREATE OR REPLACE FUNCTION find_agent_path(
    source_agent_id VARCHAR(255),
    target_agent_id VARCHAR(255),
    max_hops INTEGER DEFAULT 5
) RETURNS TABLE(
    path_length INTEGER,
    relationship_path relationship_type[],
    agent_path VARCHAR(255)[],
    total_confidence DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE agent_paths AS (
        -- Base case: Start agent
        SELECT 
            0 as path_length,
            ARRAY[]::relationship_type[] as relationship_path,
            ARRAY[source_agent_id] as agent_path,
            1.0::DECIMAL as total_confidence
        
        UNION ALL
        
        -- Recursive case: Extend path
        SELECT 
            ap.path_length + 1,
            ap.relationship_path || ar.type,
            ap.agent_path || ar.target_agent_id,
            (ap.total_confidence * ar.confidence)::DECIMAL
        FROM agent_paths ap
        JOIN agent_relationships ar ON ar.source_agent_id = ap.agent_path[array_upper(ap.agent_path, 1)]
        WHERE ap.path_length < max_hops
            AND NOT ar.target_agent_id = ANY(ap.agent_path) -- Prevent cycles
    )
    SELECT 
        path_length,
        relationship_path,
        agent_path,
        total_confidence
    FROM agent_paths
    WHERE agent_path[array_upper(agent_path, 1)] = target_agent_id
    ORDER BY path_length, total_confidence DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- HYBRID SEARCH FUNCTION
-- ============================================================================

-- Function combining vector similarity with graph traversal
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
        -- Step 1: Vector similarity search
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
        -- Step 2: Graph traversal from top vector results
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
-- SEARCH SESSION TRACKING
-- ============================================================================

CREATE TABLE graph_search_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

-- Query information
query_text TEXT,
query_hash VARCHAR(64) NOT NULL,
search_type VARCHAR(20) NOT NULL CHECK (
    search_type IN (
        'vector',
        'graph',
        'hybrid',
        'text'
    )
),

-- Search parameters
max_results INTEGER DEFAULT 10,
    max_hops INTEGER DEFAULT 2,
    min_confidence DECIMAL(3,2) DEFAULT 0.5,
    entity_type_filters VARCHAR(50)[],

-- Results and performance
result_count INTEGER NOT NULL DEFAULT 0,
execution_time_ms INTEGER NOT NULL,
vector_search_time_ms INTEGER,
graph_traversal_time_ms INTEGER,

-- Graph metrics
nodes_visited INTEGER DEFAULT 0,
edges_traversed INTEGER DEFAULT 0,
max_hops_reached INTEGER DEFAULT 0,

-- User context
tenant_id VARCHAR(255),
user_id VARCHAR(100),
session_id VARCHAR(100),

-- Temporal
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Metadata
metadata JSONB DEFAULT '{}' );

-- Indexes for search sessions
CREATE INDEX idx_search_sessions_query_hash ON graph_search_sessions (query_hash);

CREATE INDEX idx_search_sessions_tenant ON graph_search_sessions (tenant_id);

CREATE INDEX idx_search_sessions_created ON graph_search_sessions (created_at DESC);

CREATE INDEX idx_search_sessions_type ON graph_search_sessions (search_type);

CREATE INDEX idx_search_sessions_execution_time ON graph_search_sessions (execution_time_ms);

-- ============================================================================
-- SEARCH ANALYTICS FUNCTIONS
-- ============================================================================

-- Function to log search session
CREATE OR REPLACE FUNCTION log_search_session(
    p_query_text TEXT,
    p_search_type VARCHAR(20),
    p_result_count INTEGER,
    p_execution_time_ms INTEGER,
    p_tenant_id VARCHAR(255) DEFAULT NULL,
    p_metadata JSONB DEFAULT '{}'
) RETURNS UUID AS $$
DECLARE
    session_id UUID;
    query_hash_value VARCHAR(64);
BEGIN
    -- Compute query hash
    query_hash_value := encode(digest(p_query_text, 'sha256'), 'hex');
    
    -- Insert session
    INSERT INTO graph_search_sessions (
        query_text,
        query_hash,
        search_type,
        result_count,
        execution_time_ms,
        tenant_id,
        metadata
    ) VALUES (
        p_query_text,
        query_hash_value,
        p_search_type,
        p_result_count,
        p_execution_time_ms,
        p_tenant_id,
        p_metadata
    ) RETURNING id INTO session_id;
    
    RETURN session_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- SEARCH OPTIMIZATION VIEWS
-- ============================================================================

-- View for slow queries analysis
CREATE VIEW slow_search_queries AS
SELECT
    query_text,
    search_type,
    AVG(execution_time_ms) as avg_time_ms,
    COUNT(*) as frequency,
    AVG(result_count) as avg_results,
    MAX(execution_time_ms) as max_time_ms,
    MIN(created_at) as first_seen,
    MAX(created_at) as last_seen
FROM graph_search_sessions
WHERE
    created_at > NOW() - INTERVAL '7 days'
GROUP BY
    query_text,
    search_type
HAVING
    AVG(execution_time_ms) > 200
ORDER BY avg_time_ms DESC;

-- View for popular queries
CREATE VIEW popular_search_queries AS
SELECT
    query_hash,
    query_text,
    search_type,
    COUNT(*) as frequency,
    AVG(execution_time_ms) as avg_time_ms,
    AVG(result_count) as avg_results
FROM graph_search_sessions
WHERE
    created_at > NOW() - INTERVAL '30 days'
GROUP BY
    query_hash,
    query_text,
    search_type
HAVING
    COUNT(*) > 5
ORDER BY frequency DESC
LIMIT 100;

-- View for search performance by type
CREATE VIEW search_performance_by_type AS
SELECT
    search_type,
    COUNT(*) as total_searches,
    AVG(execution_time_ms) as avg_time_ms,
    PERCENTILE_CONT (0.50) WITHIN GROUP (
        ORDER BY execution_time_ms
    ) as p50_time_ms,
    PERCENTILE_CONT (0.95) WITHIN GROUP (
        ORDER BY execution_time_ms
    ) as p95_time_ms,
    PERCENTILE_CONT (0.99) WITHIN GROUP (
        ORDER BY execution_time_ms
    ) as p99_time_ms,
    AVG(result_count) as avg_results
FROM graph_search_sessions
WHERE
    created_at > NOW() - INTERVAL '7 days'
GROUP BY
    search_type;

-- ============================================================================
-- HELPER FUNCTIONS FOR SEARCH
-- ============================================================================

-- Function to refresh hybrid search materialized view
CREATE OR REPLACE FUNCTION refresh_hybrid_search_index()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY hybrid_search_materialized;
END;
$$ LANGUAGE plpgsql;

-- Function to get agent capabilities by semantic similarity
CREATE OR REPLACE FUNCTION find_similar_capabilities(
    target_embedding vector(768),
    p_tenant_id VARCHAR(255) DEFAULT NULL,
    max_results INTEGER DEFAULT 10,
    min_confidence DECIMAL DEFAULT 0.7
) RETURNS TABLE(
    capability_id UUID,
    capability_name VARCHAR(500),
    agent_id VARCHAR(255),
    similarity_score DECIMAL,
    confidence DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        id as capability_id,
        capability_name,
        agent_id,
        (1 - (embedding <=> target_embedding))::DECIMAL(5,4) as similarity_score,
        confidence
    FROM agent_capabilities_graph
    WHERE embedding IS NOT NULL
        AND (p_tenant_id IS NULL OR tenant_id = p_tenant_id)
        AND confidence >= min_confidence
    ORDER BY embedding <=> target_embedding
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to find CAWS precedents by semantic similarity
CREATE OR REPLACE FUNCTION find_similar_caws_verdicts(
    target_embedding vector(768),
    verdict_type VARCHAR(50) DEFAULT NULL,
    max_results INTEGER DEFAULT 10
) RETURNS TABLE(
    verdict_id UUID,
    entity_id VARCHAR(255),
    entity_type VARCHAR(50),
    similarity_score DECIMAL,
    constitutional_refs TEXT[],
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        id as verdict_id,
        entity_id,
        entity_type,
        (1 - (embedding <=> target_embedding))::DECIMAL(5,4) as similarity_score,
        constitutional_refs,
        created_at
    FROM caws_provenance_graph
    WHERE embedding IS NOT NULL
        AND (verdict_type IS NULL OR entity_type = verdict_type)
    ORDER BY embedding <=> target_embedding
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- GRAPH ANALYTICS FUNCTIONS
-- ============================================================================

-- Function to compute agent centrality (importance in graph)
CREATE OR REPLACE FUNCTION compute_agent_centrality()
RETURNS TABLE(
    agent_id VARCHAR(255),
    degree_centrality DECIMAL,
    betweenness_estimate DECIMAL,
    connection_count INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ap.id as agent_id,
        (
            COALESCE(outbound.out_count, 0) + COALESCE(inbound.in_count, 0)
        )::DECIMAL / NULLIF(
            (SELECT COUNT(*) FROM agent_profiles) - 1, 
            0
        ) as degree_centrality,
        -- Betweenness is expensive to compute exactly, use degree as estimate
        (
            COALESCE(outbound.out_count, 0) * COALESCE(inbound.in_count, 0)
        )::DECIMAL as betweenness_estimate,
        COALESCE(outbound.out_count, 0) + COALESCE(inbound.in_count, 0) as connection_count
    FROM agent_profiles ap
    LEFT JOIN (
        SELECT source_agent_id, COUNT(*) as out_count
        FROM agent_relationships
        GROUP BY source_agent_id
    ) outbound ON ap.id = outbound.source_agent_id
    LEFT JOIN (
        SELECT target_agent_id, COUNT(*) as in_count
        FROM agent_relationships
        GROUP BY target_agent_id
    ) inbound ON ap.id = inbound.target_agent_id
    ORDER BY degree_centrality DESC NULLS LAST;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON VIEW hybrid_search_index IS 'Unified view of all searchable entities (capabilities, agents, CAWS verdicts)';

COMMENT ON MATERIALIZED VIEW hybrid_search_materialized IS 'Materialized version of hybrid search index with HNSW index for performance';

COMMENT ON
TABLE graph_search_sessions IS 'Performance tracking for search queries with execution metrics';

COMMENT ON FUNCTION traverse_agent_relationships IS 'Multi-hop graph traversal following agent relationships with confidence thresholding';

COMMENT ON FUNCTION hybrid_search IS 'Combined vector similarity and graph traversal search for discoverable governance';

COMMENT ON FUNCTION find_similar_capabilities IS 'Semantic search for agent capabilities using vector embeddings';

COMMENT ON FUNCTION find_similar_caws_verdicts IS 'Semantic search for CAWS precedents and governance patterns';

COMMIT;

-- ============================================================================
-- POST-MIGRATION SETUP
-- ============================================================================

-- Populate materialized view (run after first embeddings are generated)
-- REFRESH MATERIALIZED VIEW hybrid_search_materialized;

-- Schedule periodic refresh (use pg_cron or application scheduler)
-- SELECT cron.schedule('refresh-hybrid-search', '*/30 * * * *', 'SELECT refresh_hybrid_search_index();');

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Hybrid search for capabilities
-- SELECT * FROM hybrid_search(
--     '[your_query_embedding]'::vector,
--     'TypeScript code generation',
--     max_results => 20,
--     include_graph_hops => 2,
--     entity_types => ARRAY['agent_capability'],
--     p_tenant_id => 'tenant-123'
-- );

-- Example 2: Find related agents within 2 hops
-- SELECT * FROM traverse_agent_relationships(
--     'agent-123',
--     max_hops => 2,
--     min_confidence => 0.7
-- );

-- Example 3: Find similar CAWS verdicts
-- SELECT * FROM find_similar_caws_verdicts(
--     '[verdict_embedding]'::vector,
--     verdict_type => 'verdict',
--     max_results => 10
-- );

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
