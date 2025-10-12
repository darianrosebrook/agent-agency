-- Migration: Create Knowledge Graph Schema for Agent Governance
-- Version: 006
-- Description: Hybrid vector-graph architecture for agent capabilities, relationships, and CAWS provenance
-- Author: @darianrosebrook
-- Date: 2025-10-12
--
-- This migration creates:
-- - Agent capability graph with pgvector embeddings
-- - Agent relationship graph with typed relationships
-- - CAWS provenance graph with cryptographic integrity
-- - Supporting indexes, triggers, and views

BEGIN;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE EXTENSION IF NOT EXISTS "vector";

CREATE EXTENSION IF NOT EXISTS "pg_trgm";
-- For fuzzy text matching

-- ============================================================================
-- CUSTOM TYPES
-- ============================================================================

-- Entity types for knowledge graph
CREATE TYPE entity_type AS ENUM (
    'CAPABILITY',      -- Agent capabilities
    'AGENT',           -- Agent entities
    'TASK',            -- Task entities
    'VERDICT',         -- CAWS verdicts
    'TECHNOLOGY',      -- Technology/tool entities
    'CONCEPT',         -- Abstract concepts
    'WAIVER',          -- CAWS waivers
    'GATE'             -- Quality gates
);

-- Relationship types for agent graph
CREATE TYPE relationship_type AS ENUM (
    'COLLABORATES_WITH',  -- Agents that work together
    'SIMILAR_TO',          -- Similar capabilities or approaches
    'DERIVED_FROM',        -- Agent forked/improved from another
    'VALIDATES',           -- Agent validates another's work
    'DEPENDS_ON',          -- Agent requires another's output
    'COMPETES_WITH',       -- Agents with overlapping capabilities
    'INFLUENCES',          -- One agent's performance affects another
    'REPLACES',            -- Agent supersedes another
    'ENHANCES'             -- Agent enhances another's capabilities
);

-- ============================================================================
-- AGENT CAPABILITIES GRAPH
-- ============================================================================

CREATE TABLE agent_capabilities_graph (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

-- Reference to agent
agent_id VARCHAR(255) NOT NULL REFERENCES agent_profiles (id) ON DELETE CASCADE,

-- Capability information
capability_type entity_type NOT NULL DEFAULT 'CAPABILITY',
    capability_name VARCHAR(500) NOT NULL,
    canonical_name VARCHAR(500) NOT NULL,  -- Auto-normalized name
    aliases TEXT[] DEFAULT '{}',            -- Alternative names

-- Quality metrics
confidence DECIMAL(3, 2) NOT NULL CHECK (
    confidence >= 0.7
    AND confidence <= 1.0
),
extraction_confidence DECIMAL(3, 2) NOT NULL CHECK (
    extraction_confidence >= 0.0
    AND extraction_confidence <= 1.0
),
validation_status VARCHAR(20) DEFAULT 'unvalidated' CHECK (
    validation_status IN (
        'validated',
        'unvalidated',
        'rejected'
    )
),

-- Vector embedding for semantic search
embedding vector (768),

-- Evidence tracking
source_tasks UUID[] DEFAULT '{}',       -- Tasks demonstrating this capability
    demonstration_count INTEGER DEFAULT 1 CHECK (demonstration_count > 0),
    success_rate DECIMAL(4,3) DEFAULT 1.0 CHECK (success_rate >= 0.0 AND success_rate <= 1.0),

-- Full-text search
search_vector tsvector GENERATED ALWAYS AS (
    setweight (
        to_tsvector ('english', capability_name),
        'A'
    ) || setweight (
        to_tsvector (
            'english',
            coalesce(
                array_to_string (aliases, ' '),
                ''
            )
        ),
        'B'
    )
) STORED,

-- Temporal information
first_observed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
last_demonstrated TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Flexible metadata
metadata JSONB DEFAULT '{}',

-- Constraints
CONSTRAINT non_empty_name CHECK (length(trim(capability_name)) > 0),
    CONSTRAINT non_empty_canonical CHECK (length(trim(canonical_name)) > 0),
    CONSTRAINT unique_agent_capability UNIQUE (agent_id, canonical_name)
);

-- Indexes for agent capabilities
CREATE INDEX idx_capabilities_agent ON agent_capabilities_graph (agent_id);

CREATE INDEX idx_capabilities_type ON agent_capabilities_graph (capability_type);

CREATE INDEX idx_capabilities_canonical ON agent_capabilities_graph (canonical_name);

CREATE INDEX idx_capabilities_confidence ON agent_capabilities_graph (confidence DESC);

CREATE INDEX idx_capabilities_search ON agent_capabilities_graph USING GIN (search_vector);

CREATE INDEX idx_capabilities_last_updated ON agent_capabilities_graph (last_updated DESC);

-- HNSW index for fast semantic search
CREATE INDEX idx_capabilities_embedding ON agent_capabilities_graph USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- ============================================================================
-- AGENT RELATIONSHIPS GRAPH
-- ============================================================================

CREATE TABLE agent_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

-- Relationship endpoints
source_agent_id VARCHAR(255) NOT NULL REFERENCES agent_profiles (id) ON DELETE CASCADE,
target_agent_id VARCHAR(255) NOT NULL REFERENCES agent_profiles (id) ON DELETE CASCADE,

-- Relationship properties
type relationship_type NOT NULL,
is_directional BOOLEAN DEFAULT false,

-- Quality metrics
confidence DECIMAL(3, 2) NOT NULL CHECK (
    confidence >= 0.5
    AND confidence <= 1.0
),
strength DECIMAL(3, 2) NOT NULL DEFAULT 1.0 CHECK (
    strength >= 0.0
    AND strength <= 1.0
),

-- Evidence tracking
cooccurrence_count INTEGER DEFAULT 1 CHECK (cooccurrence_count > 0),
    supporting_tasks UUID[] DEFAULT '{}',
    extraction_context TEXT,

-- Statistical measures
mutual_information DECIMAL(10, 6),
pointwise_mutual_information DECIMAL(10, 6),

-- Temporal information
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
last_observed TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Flexible metadata
metadata JSONB DEFAULT '{}',

-- Constraints
CONSTRAINT no_self_relationships CHECK (source_agent_id != target_agent_id),
    CONSTRAINT valid_confidence CHECK (confidence >= 0.5),
    CONSTRAINT unique_relationship UNIQUE (source_agent_id, target_agent_id, type)
);

-- Indexes for agent relationships (optimized for graph traversal)
CREATE INDEX idx_relationships_source ON agent_relationships (source_agent_id);

CREATE INDEX idx_relationships_target ON agent_relationships (target_agent_id);

CREATE INDEX idx_relationships_type ON agent_relationships(type);

CREATE INDEX idx_relationships_confidence ON agent_relationships (confidence DESC);

CREATE INDEX idx_relationships_strength ON agent_relationships (strength DESC);

CREATE INDEX idx_relationships_created ON agent_relationships (created_at DESC);

-- Composite indexes for common graph queries
CREATE INDEX idx_relationships_source_type ON agent_relationships(source_agent_id, type);

CREATE INDEX idx_relationships_target_type ON agent_relationships(target_agent_id, type);

CREATE INDEX idx_relationships_bidirectional ON agent_relationships(source_agent_id, target_agent_id, type);

-- ============================================================================
-- CAWS PROVENANCE GRAPH
-- ============================================================================

CREATE TABLE caws_provenance_graph (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

-- Entity classification
entity_type VARCHAR(50) NOT NULL CHECK (
    entity_type IN (
        'verdict',
        'waiver',
        'gate',
        'spec',
        'budget',
        'policy'
    )
),
entity_id VARCHAR(255) NOT NULL,

-- Graph structure (for hash chain)
parent_entity_id UUID REFERENCES caws_provenance_graph (id) ON DELETE RESTRICT,

-- Cryptographic integrity
hash_chain VARCHAR(128) NOT NULL, -- SHA-256 of parent + current data
signature VARCHAR(255) NOT NULL, -- ed25519 signature

-- Constitutional binding
constitutional_refs TEXT[] DEFAULT '{}', -- ["CAWS:Section4.2", "CAWS:Section5.1"]
    spec_hash VARCHAR(128),                  -- Hash of working-spec.yaml at creation time

-- Semantic search
embedding vector (768), -- For governance discovery
description TEXT,

-- Quality scores (from CAWS evaluation)
evidence_completeness DECIMAL(3, 2) CHECK (
    evidence_completeness >= 0.0
    AND evidence_completeness <= 1.0
),
budget_adherence DECIMAL(3, 2) CHECK (
    budget_adherence >= 0.0
    AND budget_adherence <= 1.0
),
gate_integrity DECIMAL(3, 2) CHECK (
    gate_integrity >= 0.0
    AND gate_integrity <= 1.0
),
provenance_clarity DECIMAL(3, 2) CHECK (
    provenance_clarity >= 0.0
    AND provenance_clarity <= 1.0
),

-- Temporal information
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Flexible metadata
metadata JSONB DEFAULT '{}',

-- Constraints
CONSTRAINT unique_entity UNIQUE (entity_type, entity_id),
    CONSTRAINT non_empty_hash CHECK (length(hash_chain) = 64),
    CONSTRAINT non_empty_signature CHECK (length(signature) > 0)
);

-- Indexes for CAWS provenance
CREATE INDEX idx_provenance_entity_type ON caws_provenance_graph (entity_type);

CREATE INDEX idx_provenance_entity_id ON caws_provenance_graph (entity_id);

CREATE INDEX idx_provenance_parent ON caws_provenance_graph (parent_entity_id);

CREATE INDEX idx_provenance_created ON caws_provenance_graph (created_at DESC);

CREATE INDEX idx_provenance_constitutional ON caws_provenance_graph USING GIN (constitutional_refs);

-- HNSW index for semantic governance discovery
CREATE INDEX idx_provenance_embedding ON caws_provenance_graph USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- ============================================================================
-- ENTITY-CHUNK PROVENANCE MAPPING
-- ============================================================================

CREATE TABLE entity_chunk_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

-- References
entity_id UUID NOT NULL REFERENCES agent_capabilities_graph (id) ON DELETE CASCADE,
chunk_id UUID NOT NULL, -- Reference to source data (task, benchmark, etc.)
chunk_type VARCHAR(50) NOT NULL CHECK (
    chunk_type IN (
        'task',
        'benchmark',
        'research',
        'training'
    )
),

-- Mapping details
mention_text TEXT NOT NULL,
mention_context TEXT,
start_position INTEGER,
end_position INTEGER,

-- Extraction details
extraction_method VARCHAR(50) NOT NULL CHECK (
    extraction_method IN (
        'manual',
        'nlp',
        'llm',
        'rule-based'
    )
),
extraction_confidence DECIMAL(3, 2) NOT NULL CHECK (
    extraction_confidence >= 0.0
    AND extraction_confidence <= 1.0
),

-- Temporal
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT unique_entity_chunk_mention UNIQUE (entity_id, chunk_id, mention_text),
    CONSTRAINT valid_position_range CHECK (
        (start_position IS NULL AND end_position IS NULL) OR 
        (start_position IS NOT NULL AND end_position IS NOT NULL AND start_position <= end_position)
    )
);

-- Indexes for entity-chunk mappings
CREATE INDEX idx_entity_chunks_entity ON entity_chunk_mappings (entity_id);

CREATE INDEX idx_entity_chunks_chunk ON entity_chunk_mappings (chunk_id);

CREATE INDEX idx_entity_chunks_type ON entity_chunk_mappings (chunk_type);

CREATE INDEX idx_entity_chunks_method ON entity_chunk_mappings (extraction_method);

-- ============================================================================
-- FUNCTIONS AND TRIGGERS
-- ============================================================================

-- Function to normalize entity names
CREATE OR REPLACE FUNCTION normalize_entity_name(input_name TEXT)
RETURNS TEXT AS $$
BEGIN
    -- Normalize: trim, lowercase, remove extra spaces
    RETURN lower(trim(regexp_replace(input_name, '\s+', ' ', 'g')));
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Trigger to set canonical name
CREATE OR REPLACE FUNCTION set_canonical_name()
RETURNS TRIGGER AS $$
BEGIN
    NEW.canonical_name = normalize_entity_name(NEW.capability_name);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_capability_canonical_name
    BEFORE INSERT OR UPDATE ON agent_capabilities_graph
    FOR EACH ROW EXECUTE FUNCTION set_canonical_name();

-- Function to update last_updated timestamp
CREATE OR REPLACE FUNCTION update_last_updated_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_capabilities_last_updated 
    BEFORE UPDATE ON agent_capabilities_graph 
    FOR EACH ROW EXECUTE FUNCTION update_last_updated_column();

CREATE TRIGGER update_relationships_last_updated 
    BEFORE UPDATE ON agent_relationships 
    FOR EACH ROW EXECUTE FUNCTION update_last_updated_column();

-- Function to prevent duplicate relationships
CREATE OR REPLACE FUNCTION prevent_duplicate_relationships()
RETURNS TRIGGER AS $$
BEGIN
    -- Check for existing relationship in either direction (unless directional)
    IF EXISTS (
        SELECT 1 FROM agent_relationships 
        WHERE (
            (source_agent_id = NEW.source_agent_id AND target_agent_id = NEW.target_agent_id) OR
            (NOT NEW.is_directional AND source_agent_id = NEW.target_agent_id AND target_agent_id = NEW.source_agent_id)
        ) AND type = NEW.type AND id != COALESCE(NEW.id, uuid_generate_v4())
    ) THEN
        RAISE EXCEPTION 'Duplicate relationship already exists between agents % and % of type %', 
            NEW.source_agent_id, NEW.target_agent_id, NEW.type;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_duplicate_relationships_trigger
    BEFORE INSERT OR UPDATE ON agent_relationships
    FOR EACH ROW EXECUTE FUNCTION prevent_duplicate_relationships();

-- Function to compute hash chain for provenance
CREATE OR REPLACE FUNCTION compute_provenance_hash(
    parent_hash VARCHAR(128),
    entity_type VARCHAR(50),
    entity_id VARCHAR(255),
    metadata JSONB
) RETURNS VARCHAR(128) AS $$
BEGIN
    RETURN encode(
        digest(
            COALESCE(parent_hash, '') || entity_type || entity_id || metadata::TEXT,
            'sha256'
        ),
        'hex'
    );
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- VIEWS FOR COMMON QUERIES
-- ============================================================================

-- View for agent capability statistics
CREATE VIEW agent_capability_summary AS
SELECT
    agent_id,
    COUNT(*) as capability_count,
    AVG(confidence) as avg_confidence,
    AVG(success_rate) as avg_success_rate,
    MAX(last_demonstrated) as last_activity
FROM agent_capabilities_graph
GROUP BY
    agent_id;

-- View for agent relationship statistics
CREATE VIEW agent_relationship_summary AS
SELECT
    source_agent_id,
    type,
    COUNT(*) as relationship_count,
    AVG(confidence) as avg_confidence,
    AVG(strength) as avg_strength
FROM agent_relationships
GROUP BY
    source_agent_id,
    type;

-- View for highly connected agents (hubs)
CREATE VIEW agent_connectivity AS
SELECT
    agent_id,
    COALESCE(outbound.outbound_count, 0) as outbound_relationships,
    COALESCE(inbound.inbound_count, 0) as inbound_relationships,
    COALESCE(outbound.outbound_count, 0) + COALESCE(inbound.inbound_count, 0) as total_relationships
FROM
    agent_profiles
    LEFT JOIN (
        SELECT
            source_agent_id as agent_id,
            COUNT(*) as outbound_count
        FROM agent_relationships
        GROUP BY
            source_agent_id
    ) outbound ON agent_profiles.id = outbound.agent_id
    LEFT JOIN (
        SELECT
            target_agent_id as agent_id,
            COUNT(*) as inbound_count
        FROM agent_relationships
        GROUP BY
            target_agent_id
    ) inbound ON agent_profiles.id = inbound.agent_id
ORDER BY total_relationships DESC;

-- View for CAWS provenance chains

CREATE VIEW caws_provenance_chains AS
WITH RECURSIVE chain AS (
    -- Base case: root provenance nodes
    SELECT 
        id,
        entity_type,
        entity_id,
        parent_entity_id,
        hash_chain,
        constitutional_refs,
        created_at,
        1 as depth,
        ARRAY[id] as path
    FROM caws_provenance_graph
    WHERE parent_entity_id IS NULL
    
    UNION ALL

-- Recursive case: child nodes
SELECT 
        p.id,
        p.entity_type,
        p.entity_id,
        p.parent_entity_id,
        p.hash_chain,
        p.constitutional_refs,
        p.created_at,
        c.depth + 1 as depth,
        c.path || p.id as path
    FROM caws_provenance_graph p
    JOIN chain c ON p.parent_entity_id = c.id
    WHERE NOT p.id = ANY(c.path) -- Prevent cycles
)
SELECT * FROM chain;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON
TABLE agent_capabilities_graph IS 'Knowledge graph of agent capabilities with vector embeddings for semantic search';

COMMENT ON
TABLE agent_relationships IS 'Typed relationships between agents with confidence scores and evidence tracking';

COMMENT ON
TABLE caws_provenance_graph IS 'Cryptographically-secured provenance graph for CAWS governance with hash chains';

COMMENT ON
TABLE entity_chunk_mappings IS 'Provenance mapping from capabilities back to source tasks and benchmarks';

COMMENT ON COLUMN agent_capabilities_graph.embedding IS '768-dimensional vector for semantic similarity search via HNSW index';

COMMENT ON COLUMN agent_relationships.confidence IS 'Confidence in relationship existence (0.5-1.0, minimum 0.5 to store)';

COMMENT ON COLUMN caws_provenance_graph.hash_chain IS 'SHA-256 hash chain for immutability verification';

COMMENT ON COLUMN caws_provenance_graph.signature IS 'ed25519 signature for cryptographic authenticity';

COMMIT;

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
