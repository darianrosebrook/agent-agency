-- Agent Memory System Database Schema
-- Extends the base agent_agency database with memory-specific tables

-- ===========================================
-- MEMORY EMBEDDINGS
-- ===========================================

-- Store vector embeddings for semantic memory search
CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id UUID PRIMARY KEY REFERENCES agent_experiences(id) ON DELETE CASCADE,
    embedding VECTOR(768),  -- pgvector extension for embeddings
    importance_score FLOAT DEFAULT 1.0 CHECK (importance_score >= 0.0 AND importance_score <= 3.0),
    decay_factor FLOAT DEFAULT 1.0 CHECK (decay_factor >= 0.0 AND decay_factor <= 1.0),
    last_accessed TIMESTAMPTZ DEFAULT NOW(),
    access_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create vector similarity indexes
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_embedding ON memory_embeddings
USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_importance ON memory_embeddings(importance_score);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_decay ON memory_embeddings(decay_factor);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_access ON memory_embeddings(last_accessed);

-- ===========================================
-- KNOWLEDGE GRAPH ENTITIES
-- ===========================================

-- Core entities in the knowledge graph
CREATE TABLE IF NOT EXISTS knowledge_graph_entities (
    id VARCHAR(255) PRIMARY KEY,
    entity_type INTEGER NOT NULL,  -- 0=Agent, 1=Task, 2=Capability, etc.
    name VARCHAR(500) NOT NULL,
    description TEXT,
    properties JSONB DEFAULT '{}',
    embedding VECTOR(768),
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    source_memories UUID[] DEFAULT '{}'
);

-- Indexes for knowledge graph entities
CREATE INDEX IF NOT EXISTS idx_entities_type ON knowledge_graph_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON knowledge_graph_entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_embedding ON knowledge_graph_entities
USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_entities_confidence ON knowledge_graph_entities(confidence);
CREATE INDEX IF NOT EXISTS idx_entities_updated ON knowledge_graph_entities(updated_at);

-- ===========================================
-- KNOWLEDGE GRAPH RELATIONSHIPS
-- ===========================================

-- Relationships between entities
CREATE TABLE IF NOT EXISTS knowledge_graph_relationships (
    id VARCHAR(255) PRIMARY KEY,
    source_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    target_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    relationship_type INTEGER NOT NULL,  -- 0=Performs, 1=Requires, etc.
    properties JSONB DEFAULT '{}',
    strength FLOAT DEFAULT 1.0 CHECK (strength >= 0.0 AND strength <= 2.0),
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    bidirectional BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    source_memories UUID[] DEFAULT '{}',
    CONSTRAINT different_entities CHECK (source_entity != target_entity)
);

-- Indexes for knowledge graph relationships
CREATE INDEX IF NOT EXISTS idx_relationships_source ON knowledge_graph_relationships(source_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON knowledge_graph_relationships(target_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_type ON knowledge_graph_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_relationships_strength ON knowledge_graph_relationships(strength);
CREATE INDEX IF NOT EXISTS idx_relationships_confidence ON knowledge_graph_relationships(confidence);
CREATE INDEX IF NOT EXISTS idx_relationships_updated ON knowledge_graph_relationships(updated_at);

-- ===========================================
-- TEMPORAL ANALYSIS RESULTS
-- ===========================================

-- Store results of temporal analysis and trends
CREATE TABLE IF NOT EXISTS temporal_analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,  -- 'agent', 'task', 'capability'
    entity_id VARCHAR(255) NOT NULL,
    analysis_type VARCHAR(50) NOT NULL,  -- 'trend', 'change_point', 'causality'
    time_range TSRANGE NOT NULL,
    results JSONB NOT NULL,
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for temporal analysis
CREATE INDEX IF NOT EXISTS idx_temporal_entity ON temporal_analysis_results(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_temporal_type ON temporal_analysis_results(analysis_type);
CREATE INDEX IF NOT EXISTS idx_temporal_range ON temporal_analysis_results(time_range);
CREATE INDEX IF NOT EXISTS idx_temporal_created ON temporal_analysis_results(created_at);

-- ===========================================
-- PROVENANCE TRACKING
-- ===========================================

-- Track memory operations for explainability
CREATE TABLE IF NOT EXISTS memory_provenance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation VARCHAR(50) NOT NULL,  -- 'store', 'retrieve', 'search', etc.
    memory_id UUID REFERENCES agent_experiences(id) ON DELETE SET NULL,
    agent_id VARCHAR(255),
    context JSONB DEFAULT '{}',
    reasoning TEXT[],
    confidence FLOAT CHECK (confidence >= 0.0 AND confidence <= 1.0),
    processing_time_ms INTEGER,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for provenance tracking
CREATE INDEX IF NOT EXISTS idx_provenance_operation ON memory_provenance(operation);
CREATE INDEX IF NOT EXISTS idx_provenance_memory ON memory_provenance(memory_id);
CREATE INDEX IF NOT EXISTS idx_provenance_agent ON memory_provenance(agent_id);
CREATE INDEX IF NOT EXISTS idx_provenance_timestamp ON memory_provenance(timestamp);

-- ===========================================
-- CONTEXT OFFLOADING
-- ===========================================

-- Store offloaded context for memory compression
CREATE TABLE IF NOT EXISTS offloaded_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_memory_id UUID REFERENCES agent_experiences(id) ON DELETE CASCADE,
    context_type VARCHAR(50) NOT NULL,  -- 'episodic', 'semantic', 'working'
    compressed_content TEXT NOT NULL,
    compression_ratio FLOAT,
    retrieval_count INTEGER DEFAULT 0,
    last_retrieved TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for context offloading
CREATE INDEX IF NOT EXISTS idx_offloaded_memory ON offloaded_contexts(original_memory_id);
CREATE INDEX IF NOT EXISTS idx_offloaded_type ON offloaded_contexts(context_type);
CREATE INDEX IF NOT EXISTS idx_offloaded_expires ON offloaded_contexts(expires_at);
CREATE INDEX IF NOT EXISTS idx_offloaded_retrieved ON offloaded_contexts(last_retrieved);

-- ===========================================
-- MEMORY SYSTEM METRICS
-- ===========================================

-- Track memory system performance and health
CREATE TABLE IF NOT EXISTS memory_system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value FLOAT NOT NULL,
    metric_unit VARCHAR(20),
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for metrics
CREATE INDEX IF NOT EXISTS idx_metrics_name ON memory_system_metrics(metric_name);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON memory_system_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_metrics_labels ON memory_system_metrics USING gin(labels);

-- ===========================================
-- ENUM TYPE DEFINITIONS
-- ===========================================

-- Entity types enum (for better query performance)
DO $$ BEGIN
    CREATE TYPE entity_type AS ENUM (
        'agent', 'task', 'capability', 'domain', 'tool',
        'outcome', 'concept', 'person', 'organization', 'location', 'technology'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Relationship types enum
DO $$ BEGIN
    CREATE TYPE relationship_type AS ENUM (
        'performs', 'requires', 'enables', 'conflicts', 'improves',
        'learns_from', 'collaborates_with', 'manages', 'creates', 'uses',
        'contains', 'related_to', 'causes', 'prevents', 'similar_to'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Memory operation types enum
DO $$ BEGIN
    CREATE TYPE memory_operation AS ENUM (
        'store', 'retrieve', 'update', 'delete', 'search',
        'reason', 'consolidate', 'decay', 'offload'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ===========================================
-- VIEWS FOR COMMON QUERIES
-- ===========================================

-- View for agent performance over time
CREATE OR REPLACE VIEW agent_performance_trends AS
SELECT
    ae.agent_id,
    DATE_TRUNC('day', ae.timestamp) as date,
    AVG((ae.outcome->>'performance_score')::float) as avg_performance,
    AVG((ae.outcome->>'execution_time_ms')::float) as avg_execution_time,
    COUNT(*) as experience_count,
    COUNT(CASE WHEN ae.outcome->>'success' = 'true' THEN 1 END)::float / COUNT(*)::float as success_rate
FROM agent_experiences ae
GROUP BY ae.agent_id, DATE_TRUNC('day', ae.timestamp)
ORDER BY ae.agent_id, date;

-- View for capability learning patterns
CREATE OR REPLACE VIEW capability_learning_patterns AS
SELECT
    ae.agent_id,
    jsonb_array_elements_text(ae.outcome->'learned_capabilities') as capability,
    DATE_TRUNC('week', ae.timestamp) as week,
    COUNT(*) as learning_events,
    AVG((ae.outcome->>'performance_score')::float) as avg_performance
FROM agent_experiences ae
WHERE jsonb_array_length(ae.outcome->'learned_capabilities') > 0
GROUP BY ae.agent_id, capability, DATE_TRUNC('week', ae.timestamp)
ORDER BY ae.agent_id, capability, week;

-- View for memory access patterns
CREATE OR REPLACE VIEW memory_access_patterns AS
SELECT
    me.memory_id,
    ae.agent_id,
    ae.context->>'task_type' as task_type,
    me.importance_score,
    me.decay_factor,
    me.access_count,
    me.last_accessed,
    AGE(NOW(), me.created_at) as memory_age
FROM memory_embeddings me
JOIN agent_experiences ae ON me.memory_id = ae.id
ORDER BY me.last_accessed DESC;

-- ===========================================
-- UTILITY FUNCTIONS
-- ===========================================

-- Function to calculate memory relevance score
CREATE OR REPLACE FUNCTION calculate_memory_relevance(
    importance FLOAT,
    decay FLOAT,
    recency_hours FLOAT,
    access_count INTEGER
) RETURNS FLOAT AS $$
BEGIN
    -- Combine importance, decay, recency, and access patterns
    RETURN importance * decay *
           GREATEST(0.5, 1.0 - (recency_hours / 168.0)) *  -- 7 day recency factor
           (1.0 + LOG(GREATEST(1, access_count)) / 10.0);  -- Access frequency bonus
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to find similar memories by embedding
CREATE OR REPLACE FUNCTION find_similar_memories(
    query_embedding VECTOR(768),
    similarity_threshold FLOAT DEFAULT 0.7,
    max_results INTEGER DEFAULT 10
) RETURNS TABLE(
    memory_id UUID,
    similarity_score FLOAT,
    importance_score FLOAT,
    relevance_score FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        me.memory_id,
        (1.0 - (me.embedding <=> query_embedding)) as similarity_score,
        me.importance_score,
        calculate_memory_relevance(
            me.importance_score,
            me.decay_factor,
            EXTRACT(EPOCH FROM (NOW() - me.last_accessed)) / 3600.0,
            me.access_count
        ) as relevance_score
    FROM memory_embeddings me
    WHERE (1.0 - (me.embedding <=> query_embedding)) >= similarity_threshold
    ORDER BY relevance_score DESC, similarity_score DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to update memory access statistics
CREATE OR REPLACE FUNCTION update_memory_access(memory_uuid UUID) RETURNS VOID AS $$
BEGIN
    UPDATE memory_embeddings
    SET access_count = access_count + 1,
        last_accessed = NOW()
    WHERE memory_id = memory_uuid;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- TRIGGERS FOR AUTOMATIC MAINTENANCE
-- ===========================================

-- Trigger to automatically update memory access on retrieval
CREATE OR REPLACE FUNCTION trigger_memory_access() RETURNS TRIGGER AS $$
BEGIN
    -- Update access statistics when memory is retrieved
    PERFORM update_memory_access(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Note: This trigger would be added to agent_experiences table when memory system is active
-- CREATE TRIGGER memory_access_trigger AFTER SELECT ON agent_experiences
--     FOR EACH ROW EXECUTE FUNCTION trigger_memory_access();

-- ===========================================
-- CLEANUP AND MAINTENANCE
-- ===========================================

-- Function to clean up expired offloaded contexts
CREATE OR REPLACE FUNCTION cleanup_expired_contexts() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM offloaded_contexts
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to apply memory decay
CREATE OR REPLACE FUNCTION apply_memory_decay_batch(
    decay_rate FLOAT DEFAULT 0.95,
    max_age_hours INTEGER DEFAULT 168
) RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE memory_embeddings
    SET decay_factor = GREATEST(decay_factor * decay_rate, 0.1)
    WHERE last_accessed < NOW() - (max_age_hours || ' hours')::INTERVAL
      AND decay_factor > 0.1;

    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- PERFORMANCE OPTIMIZATION
-- ===========================================

-- Create partial indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_embeddings_high_importance ON memory_embeddings(importance_score)
WHERE importance_score > 1.5;

CREATE INDEX IF NOT EXISTS idx_embeddings_recent_access ON memory_embeddings(last_accessed)
WHERE last_accessed > NOW() - INTERVAL '7 days';

CREATE INDEX IF NOT EXISTS idx_relationships_strong ON knowledge_graph_relationships(strength)
WHERE strength > 1.2;

-- ===========================================
-- MONITORING AND HEALTH CHECKS
-- ===========================================

-- Function to get memory system health metrics
CREATE OR REPLACE FUNCTION get_memory_system_health() RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'total_memories', (SELECT COUNT(*) FROM agent_experiences),
        'embedded_memories', (SELECT COUNT(*) FROM memory_embeddings),
        'knowledge_entities', (SELECT COUNT(*) FROM knowledge_graph_entities),
        'knowledge_relationships', (SELECT COUNT(*) FROM knowledge_graph_relationships),
        'avg_importance', (SELECT AVG(importance_score) FROM memory_embeddings),
        'avg_decay', (SELECT AVG(decay_factor) FROM memory_embeddings),
        'oldest_memory', (SELECT MIN(created_at) FROM agent_experiences),
        'newest_memory', (SELECT MAX(created_at) FROM agent_experiences),
        'expired_contexts', (SELECT COUNT(*) FROM offloaded_contexts WHERE expires_at < NOW())
    ) INTO result;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- MIGRATION HELPERS
-- ===========================================

-- Function to migrate existing experiences to memory embeddings
CREATE OR REPLACE FUNCTION migrate_experiences_to_embeddings() RETURNS INTEGER AS $$
DECLARE
    migrated_count INTEGER := 0;
    experience_record RECORD;
BEGIN
    -- Note: This would need to be called with actual embedding generation
    -- For now, just create placeholder records
    FOR experience_record IN
        SELECT id FROM agent_experiences
        WHERE id NOT IN (SELECT memory_id FROM memory_embeddings)
        LIMIT 1000  -- Batch processing
    LOOP
        INSERT INTO memory_embeddings (memory_id, embedding, importance_score, decay_factor)
        VALUES (experience_record.id, NULL, 1.0, 1.0);

        migrated_count := migrated_count + 1;
    END LOOP;

    RETURN migrated_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE memory_embeddings IS 'Vector embeddings for semantic memory search with decay and importance tracking';
COMMENT ON TABLE knowledge_graph_entities IS 'Entities in the knowledge graph (agents, tasks, capabilities, etc.)';
COMMENT ON TABLE knowledge_graph_relationships IS 'Relationships between knowledge graph entities';
COMMENT ON TABLE temporal_analysis_results IS 'Cached results of temporal analysis and trend detection';
COMMENT ON TABLE memory_provenance IS 'Audit trail of memory operations for explainability';
COMMENT ON TABLE offloaded_contexts IS 'Compressed/archived contexts for memory efficiency';
COMMENT ON TABLE memory_system_metrics IS 'Performance and health metrics for the memory system';
