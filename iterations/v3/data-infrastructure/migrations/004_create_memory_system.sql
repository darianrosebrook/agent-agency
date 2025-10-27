-- Create complete memory system schema
-- Migration: 004_create_memory_system.sql

-- ===========================================
-- MEMORY EMBEDDINGS
-- ===========================================

-- Store vector embeddings for semantic memory search
CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id UUID PRIMARY KEY REFERENCES agent_experiences(id) ON DELETE CASCADE,
    embedding VECTOR(768),  -- pgvector extension for embeddings
    workspace_id UUID NULL, -- NULL = global memory, UUID = workspace-scoped
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
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_workspace ON memory_embeddings(workspace_id);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_importance ON memory_embeddings(importance_score);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_decay ON memory_embeddings(decay_factor);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_access ON memory_embeddings(last_accessed);

-- ===========================================
-- KNOWLEDGE GRAPH ENTITIES
-- ===========================================

-- Core entities in the knowledge graph
CREATE TABLE IF NOT EXISTS knowledge_graph_entities (
    id VARCHAR(255) PRIMARY KEY,
    workspace_id UUID NULL, -- NULL = global entity, UUID = workspace-scoped
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
CREATE INDEX IF NOT EXISTS idx_entities_workspace ON knowledge_graph_entities(workspace_id);
CREATE INDEX IF NOT EXISTS idx_entities_type ON knowledge_graph_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON knowledge_graph_entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_embedding ON knowledge_graph_entities
USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_entities_confidence ON knowledge_graph_entities(confidence);
CREATE INDEX IF NOT EXISTS idx_entities_updated ON knowledge_graph_entities(updated_at);

-- ===========================================
-- KNOWLEDGE GRAPH RELATIONSHIPS
-- ===========================================

-- Relationships between entities (can span workspaces via cross-links)
CREATE TABLE IF NOT EXISTS knowledge_graph_relationships (
    id VARCHAR(255) PRIMARY KEY,
    workspace_id UUID NULL, -- NULL = global relationship, UUID = workspace-scoped
    source_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    target_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    relationship_type INTEGER NOT NULL,  -- 0=Performs, 1=Requires, etc.
    properties JSONB DEFAULT '{}',
    strength FLOAT DEFAULT 1.0 CHECK (strength >= 0.0 AND strength <= 2.0),
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    bidirectional BOOLEAN DEFAULT FALSE,
    cross_workspace BOOLEAN DEFAULT FALSE, -- True for relationships spanning workspaces
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    source_memories UUID[] DEFAULT '{}',
    CONSTRAINT different_entities CHECK (source_entity != target_entity)
);

-- Indexes for knowledge graph relationships
CREATE INDEX IF NOT EXISTS idx_relationships_workspace ON knowledge_graph_relationships(workspace_id);
CREATE INDEX IF NOT EXISTS idx_relationships_cross_workspace ON knowledge_graph_relationships(cross_workspace);
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
    workspace_id UUID NULL, -- NULL = global analysis, UUID = workspace-scoped
    entity_type VARCHAR(50) NOT NULL,  -- 'agent', 'task', 'capability'
    entity_id VARCHAR(255) NOT NULL,
    analysis_type VARCHAR(50) NOT NULL,  -- 'trend', 'change_point', 'causality'
    time_range TSRANGE NOT NULL,
    results JSONB NOT NULL,
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for temporal analysis
CREATE INDEX IF NOT EXISTS idx_temporal_workspace ON temporal_analysis_results(workspace_id);
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
    workspace_id UUID NULL, -- NULL = global operation, UUID = workspace-scoped
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
CREATE INDEX IF NOT EXISTS idx_provenance_workspace ON memory_provenance(workspace_id);
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
    workspace_id UUID NULL, -- NULL = global offload, UUID = workspace-scoped
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
CREATE INDEX IF NOT EXISTS idx_offloaded_workspace ON offloaded_contexts(workspace_id);
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
    workspace_id UUID NULL, -- NULL = global metrics, UUID = workspace-scoped
    metric_name VARCHAR(100) NOT NULL,
    metric_value FLOAT NOT NULL,
    metric_unit VARCHAR(20),
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for metrics
CREATE INDEX IF NOT EXISTS idx_metrics_workspace ON memory_system_metrics(workspace_id);
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

-- Function to update memory access statistics
CREATE OR REPLACE FUNCTION update_memory_access(memory_uuid UUID) RETURNS VOID AS $$
BEGIN
    UPDATE memory_embeddings
    SET access_count = access_count + 1,
        last_accessed = NOW()
    WHERE memory_id = memory_uuid;
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

-- Log the migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('004', 'Create memory system tables and functions', NOW())
ON CONFLICT (version) DO NOTHING;
