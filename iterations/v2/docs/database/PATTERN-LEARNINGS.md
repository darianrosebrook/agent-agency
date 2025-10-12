# Database Pattern Learnings for V2

**Author**: @darianrosebrook  
**Date**: 2025-10-12  
**Purpose**: Extracted patterns from POC and Obsidian-RAG for V2 implementation

---

## Overview

This document extracts specific, actionable patterns that V2 must adopt to achieve hybrid vector-graph architecture with multi-tenant isolation and CAWS governance. Each pattern includes implementation guidance, code examples, and performance considerations.

---

## Patterns from Obsidian-RAG

### Pattern 1: HNSW Vector Indexes for Semantic Search

**Problem**: Need sub-100ms semantic similarity search across thousands of entities

**Solution**: Use HNSW (Hierarchical Navigable Small World) indexes instead of IVFFlat

**Implementation:**

```sql
-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create table with vector column
CREATE TABLE agent_capabilities_graph (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    capability_name VARCHAR(500) NOT NULL,
    embedding vector(768), -- 768-dimensional embeddings
    -- other columns...
);

-- Create HNSW index with optimized parameters
CREATE INDEX idx_capabilities_embedding
    ON agent_capabilities_graph
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Query example
SELECT
    id,
    capability_name,
    embedding <=> '[query_vector]'::vector AS distance
FROM agent_capabilities_graph
ORDER BY embedding <=> '[query_vector]'::vector
LIMIT 10;
```

**Parameters:**

- `m = 16`: Number of connections per layer (higher = better recall, more memory)
- `ef_construction = 64`: Build-time accuracy (higher = better index, slower build)
- Query `ef_search = 40` (set at query time for speed vs. accuracy trade-off)

**Performance Characteristics:**

- Build time: ~8 minutes for 100K vectors (acceptable for V2's scale)
- Query time: 10-20ms P95 for semantic search
- Recall: ~98% at k=10
- Storage: 2.8x vector data size

**V2 Application:**

- Agent capability semantic search
- CAWS verdict similarity for governance discovery
- Task requirement matching

---

### Pattern 2: Typed Knowledge Graph with Strong Constraints

**Problem**: Need queryable relationships between agents, tasks, and capabilities with quality assurance

**Solution**: Use ENUMs for relationship types and strong constraints for data quality

**Implementation:**

```sql
-- Define relationship types as ENUM
CREATE TYPE relationship_type AS ENUM (
    'COLLABORATES_WITH',  -- Agents that work together
    'SIMILAR_TO',          -- Similar capabilities or approaches
    'DERIVED_FROM',        -- Agent forked/improved from another
    'VALIDATES',           -- Agent validates another's work
    'DEPENDS_ON',          -- Agent requires another's output
    'COMPETES_WITH',       -- Agents with overlapping capabilities
    'INFLUENCES'           -- One agent's performance affects another
);

-- Relationships table with full metadata
CREATE TABLE agent_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Relationship endpoints
    source_agent_id VARCHAR(255) NOT NULL,
    target_agent_id VARCHAR(255) NOT NULL,

    -- Type and directionality
    type relationship_type NOT NULL,
    is_directional BOOLEAN DEFAULT false,

    -- Quality metrics
    confidence DECIMAL(3,2) NOT NULL CHECK (confidence >= 0.5 AND confidence <= 1.0),
    strength DECIMAL(3,2) NOT NULL DEFAULT 1.0 CHECK (strength >= 0.0 AND strength <= 1.0),

    -- Evidence
    cooccurrence_count INTEGER DEFAULT 1 CHECK (cooccurrence_count > 0),
    supporting_tasks UUID[], -- Tasks that demonstrated this relationship
    extraction_context TEXT,

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT no_self_relationships CHECK (source_agent_id != target_agent_id),
    CONSTRAINT unique_relationship UNIQUE (source_agent_id, target_agent_id, type)
);

-- Composite indexes for graph traversal
CREATE INDEX idx_relationships_source_type
    ON agent_relationships(source_agent_id, type);
CREATE INDEX idx_relationships_target_type
    ON agent_relationships(target_agent_id, type);
CREATE INDEX idx_relationships_bidirectional
    ON agent_relationships(source_agent_id, target_agent_id, type);
```

**Constraints Rationale:**

- `confidence >= 0.5`: Only high-confidence relationships stored
- `cooccurrence_count > 0`: Must have been observed at least once
- `no_self_relationships`: Prevent circular self-references
- `unique_relationship`: One relationship type per agent pair

**V2 Application:**

- Agent collaboration patterns for routing
- Capability similarity for fallback selection
- Performance correlation analysis

---

### Pattern 3: Entity-Chunk Provenance Mapping

**Problem**: Need to trace entities back to source data for explainability

**Solution**: Maintain bidirectional mapping between entities and source chunks

**Implementation:**

```sql
CREATE TABLE entity_chunk_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- References
    entity_id UUID NOT NULL REFERENCES agent_capabilities_graph(id) ON DELETE CASCADE,
    chunk_id UUID NOT NULL, -- Reference to source data (task, benchmark, etc.)

    -- Extraction details
    mention_text TEXT NOT NULL,     -- How it was observed
    mention_context TEXT,           -- Surrounding context
    start_position INTEGER,         -- Position in source
    end_position INTEGER,

    -- Quality
    extraction_method VARCHAR(50) NOT NULL,
    extraction_confidence DECIMAL(3,2) NOT NULL CHECK (extraction_confidence >= 0.0 AND extraction_confidence <= 1.0),

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_entity_chunk_mention UNIQUE (entity_id, chunk_id, mention_text),
    CONSTRAINT valid_position_range CHECK (
        (start_position IS NULL AND end_position IS NULL) OR
        (start_position IS NOT NULL AND end_position IS NOT NULL AND start_position <= end_position)
    )
);

CREATE INDEX idx_entity_chunks_entity ON entity_chunk_mappings(entity_id);
CREATE INDEX idx_entity_chunks_chunk ON entity_chunk_mappings(chunk_id);
```

**Query Pattern:**

```sql
-- Find all tasks where agent demonstrated a capability
SELECT
    t.id AS task_id,
    t.description,
    ecm.mention_text,
    ecm.extraction_confidence
FROM entity_chunk_mappings ecm
JOIN tasks t ON t.id = ecm.chunk_id::text
WHERE ecm.entity_id = 'capability-uuid'
ORDER BY ecm.extraction_confidence DESC;
```

**V2 Application:**

- "Why does agent X have capability Y?" provenance
- Training data attribution for RL
- Capability validation from performance history

---

### Pattern 4: Canonical Name Normalization

**Problem**: Entities may have variations in naming ("TypeScript" vs "typescript" vs "TS")

**Solution**: Automatic canonical name normalization via triggers

**Implementation:**

```sql
-- Normalization function
CREATE OR REPLACE FUNCTION normalize_entity_name(input_name TEXT)
RETURNS TEXT AS $$
BEGIN
    -- Normalize: trim, lowercase, remove extra spaces
    RETURN lower(trim(regexp_replace(input_name, '\s+', ' ', 'g')));
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Table with canonical names
CREATE TABLE agent_capabilities_graph (
    id UUID PRIMARY KEY,
    capability_name VARCHAR(500) NOT NULL,
    canonical_name VARCHAR(500) NOT NULL,
    aliases TEXT[] DEFAULT '{}',
    -- ...
);

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
```

**Benefit**: Automatic deduplication and consistent queries

**V2 Application:**

- Capability name normalization
- CAWS verdict deduplication
- Agent name standardization

---

### Pattern 5: Search Session Analytics

**Problem**: Need to optimize search performance and understand query patterns

**Solution**: Track search sessions with performance metrics

**Implementation:**

```sql
CREATE TABLE graph_search_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Query information
    query_text TEXT NOT NULL,
    query_hash VARCHAR(64) NOT NULL, -- SHA-256 for deduplication
    search_type VARCHAR(20) NOT NULL CHECK (search_type IN ('vector', 'graph', 'hybrid')),

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
    user_id VARCHAR(100),
    session_id VARCHAR(100),

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_sessions_query_hash ON graph_search_sessions(query_hash);
CREATE INDEX idx_search_sessions_execution_time ON graph_search_sessions(execution_time_ms);
```

**Analytics Queries:**

```sql
-- Identify slow queries
SELECT
    query_text,
    AVG(execution_time_ms) as avg_time,
    COUNT(*) as frequency
FROM graph_search_sessions
WHERE created_at > NOW() - INTERVAL '7 days'
GROUP BY query_text
HAVING AVG(execution_time_ms) > 200
ORDER BY avg_time DESC;
```

**V2 Application:**

- Capability search optimization
- CAWS governance query performance
- Routing decision analytics

---

### Pattern 6: Similarity Caching for Performance

**Problem**: Expensive similarity calculations performed repeatedly

**Solution**: Cache computed similarities with TTL management

**Implementation:**

```sql
CREATE TABLE entity_similarity_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Entity pair (ordered to prevent duplicates)
    entity1_id UUID NOT NULL,
    entity2_id UUID NOT NULL,

    -- Similarity metrics
    cosine_similarity DECIMAL(5,4) NOT NULL CHECK (cosine_similarity >= -1.0 AND cosine_similarity <= 1.0),
    semantic_similarity DECIMAL(5,4),

    -- Cache metadata
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    computation_method VARCHAR(50) NOT NULL DEFAULT 'cosine',

    -- Constraints ensure ordered pairs (entity1_id < entity2_id)
    CONSTRAINT unique_entity_pair UNIQUE (entity1_id, entity2_id),
    CONSTRAINT ordered_entity_pair CHECK (entity1_id < entity2_id)
);

CREATE INDEX idx_similarity_cache_similarity ON entity_similarity_cache(cosine_similarity DESC);

-- Cache lookup function
CREATE OR REPLACE FUNCTION get_cached_similarity(
    e1_id UUID,
    e2_id UUID,
    ttl_hours INTEGER DEFAULT 24
) RETURNS DECIMAL(5,4) AS $$
DECLARE
    cached_sim DECIMAL(5,4);
BEGIN
    -- Ensure ordered pair
    IF e1_id > e2_id THEN
        SELECT get_cached_similarity(e2_id, e1_id, ttl_hours) INTO cached_sim;
        RETURN cached_sim;
    END IF;

    -- Lookup from cache
    SELECT cosine_similarity INTO cached_sim
    FROM entity_similarity_cache
    WHERE entity1_id = e1_id
      AND entity2_id = e2_id
      AND computed_at > NOW() - (ttl_hours || ' hours')::INTERVAL;

    RETURN cached_sim;
END;
$$ LANGUAGE plpgsql;
```

**Cache Invalidation Strategy:**

```sql
-- Periodic cleanup of old cache entries
DELETE FROM entity_similarity_cache
WHERE computed_at < NOW() - INTERVAL '7 days';

-- Or create a cleanup function
CREATE OR REPLACE FUNCTION cleanup_similarity_cache(
    retention_days INTEGER DEFAULT 7
) RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM entity_similarity_cache
    WHERE computed_at < NOW() - (retention_days || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
```

**V2 Application:**

- Agent similarity for routing
- Capability overlap calculation
- CAWS verdict similarity

---

## Patterns from POC

### Pattern 7: Row Level Security for Multi-Tenancy

**Problem**: Need database-level guarantee of tenant data isolation

**Solution**: Enable RLS with policies that filter by tenant context

**Implementation:**

```sql
-- Tenant isolation levels
CREATE TYPE isolation_level AS ENUM ('strict', 'shared', 'federated');

-- Tenant management table
CREATE TABLE tenants (
    id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    isolation_level isolation_level NOT NULL,
    access_policies JSONB DEFAULT '[]',
    sharing_rules JSONB DEFAULT '[]',
    data_retention JSONB NOT NULL,
    encryption_enabled BOOLEAN DEFAULT false,
    audit_logging BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enable RLS on tenant-scoped tables
ALTER TABLE agent_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE performance_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE benchmark_datasets ENABLE ROW LEVEL SECURITY;

-- Strict isolation policy
CREATE POLICY tenant_strict_isolation ON agent_profiles
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        AND (SELECT isolation_level FROM tenants WHERE id = tenant_id) = 'strict'
    );

-- Shared isolation policy (can read shared data)
CREATE POLICY tenant_shared_access ON agent_profiles
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        OR (
            (SELECT isolation_level FROM tenants WHERE id = tenant_id) = 'shared'
            AND tenant_id IN (
                SELECT unnest(sharing_rules::jsonb->>'allowed_tenants')
                FROM tenants
                WHERE id = current_setting('app.current_tenant', true)::VARCHAR
            )
        )
    );

-- Federated isolation policy (cross-tenant learning allowed)
CREATE POLICY tenant_federated_access ON performance_events
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        OR (SELECT isolation_level FROM tenants WHERE id = current_setting('app.current_tenant', true)::VARCHAR) = 'federated'
    );
```

**Application Code Integration:**

```typescript
// Set tenant context at connection time
async function executeTenantQuery<T>(
  tenantId: string,
  query: string,
  params: any[]
): Promise<T> {
  const client = await pool.connect();
  try {
    // Set tenant context for RLS
    await client.query(`SET LOCAL app.current_tenant = $1`, [tenantId]);

    // Execute query - RLS automatically filters
    const result = await client.query(query, params);
    return result.rows;
  } finally {
    client.release();
  }
}
```

**Benefits:**

- Cannot bypass tenant isolation through application bugs
- Automatic filtering of all queries
- Supports three isolation modes

**Trade-offs:**

- ~5-10ms overhead per query for policy evaluation
- Requires session variable management
- Complicates connection pooling

**V2 Application:**

- Agent registry tenant isolation
- Performance tracking data separation
- Benchmark dataset access control

---

### Pattern 8: Privacy Levels for Federated Learning

**Problem**: Need configurable privacy preservation for cross-tenant data sharing

**Solution**: Privacy level enum with differential privacy configurations

**Implementation:**

```sql
CREATE TYPE privacy_level AS ENUM ('basic', 'differential', 'secure');

CREATE TABLE tenant_privacy_config (
    tenant_id VARCHAR(255) PRIMARY KEY REFERENCES tenants(id),
    privacy_level privacy_level NOT NULL DEFAULT 'differential',
    noise_magnitude DECIMAL(5,4) DEFAULT 0.01,  -- For differential privacy
    k_anonymity INTEGER DEFAULT 5,               -- Minimum group size
    epsilon DECIMAL(5,4) DEFAULT 1.0,            -- Privacy budget
    delta DECIMAL(10,8) DEFAULT 0.00001,         -- Privacy parameter
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Function to apply differential privacy noise
CREATE OR REPLACE FUNCTION add_dp_noise(
    value DECIMAL,
    tenant_id VARCHAR(255)
) RETURNS DECIMAL AS $$
DECLARE
    config RECORD;
    noise DECIMAL;
BEGIN
    -- Get privacy config
    SELECT * INTO config FROM tenant_privacy_config WHERE tenant_id = tenant_id;

    -- Skip if basic privacy
    IF config.privacy_level = 'basic' THEN
        RETURN value;
    END IF;

    -- Add Laplace noise for differential privacy
    noise := config.noise_magnitude * (random() - 0.5) * 2;

    RETURN value + noise;
END;
$$ LANGUAGE plpgsql;
```

**Aggregation Query with Privacy:**

```sql
-- Aggregate with differential privacy
SELECT
    capability_type,
    add_dp_noise(AVG(confidence)::DECIMAL, 'tenant-123') as avg_confidence_private,
    COUNT(*) as count
FROM agent_capabilities_graph
WHERE (SELECT isolation_level FROM tenants WHERE id = tenant_id) = 'federated'
GROUP BY capability_type
HAVING COUNT(*) >= (SELECT k_anonymity FROM tenant_privacy_config WHERE tenant_id = 'tenant-123');
```

**V2 Application:**

- Cross-tenant agent learning
- Federated performance benchmarking
- Privacy-preserving RL training data

---

### Pattern 9: Compressed Context Offloading

**Problem**: Large context data exceeds practical database storage limits

**Solution**: Store compressed summaries with pointers to full context

**Implementation:**

```sql
CREATE TABLE offloaded_contexts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id),

    -- Summary for quick access
    summary TEXT NOT NULL,
    summary_embedding vector(768),

    -- Compression metadata
    original_size_bytes INTEGER NOT NULL,
    compressed_size_bytes INTEGER NOT NULL,
    compression_ratio DECIMAL(5,2) GENERATED ALWAYS AS (
        compressed_size_bytes::DECIMAL / NULLIF(original_size_bytes, 0)
    ) STORED,

    -- External storage reference
    storage_location VARCHAR(1000),  -- S3/file path
    storage_checksum VARCHAR(128),   -- SHA-256 for integrity

    -- Access tracking
    last_accessed TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    CONSTRAINT valid_compression CHECK (compressed_size_bytes <= original_size_bytes)
);

CREATE INDEX idx_offloaded_contexts_tenant ON offloaded_contexts(tenant_id);
CREATE INDEX idx_offloaded_contexts_summary_embedding
    ON offloaded_contexts USING hnsw (summary_embedding vector_cosine_ops);
```

**Retrieval Pattern:**

```typescript
async function retrieveContext(contextId: string): Promise<string> {
  // Get metadata from database
  const { storage_location, storage_checksum } = await db.query(
    "SELECT storage_location, storage_checksum FROM offloaded_contexts WHERE id = $1",
    [contextId]
  );

  // Fetch from external storage (S3, filesystem, etc.)
  const compressed = await fetchFromStorage(storage_location);

  // Verify integrity
  const checksum = sha256(compressed);
  if (checksum !== storage_checksum) {
    throw new Error("Context integrity check failed");
  }

  // Decompress and return
  return await decompress(compressed);
}
```

**V2 Application:**

- Large task context storage
- Benchmark dataset compression
- Historical performance data archival

---

### Pattern 10: Audit Logging with Event Sourcing

**Problem**: Need complete audit trail of all database operations

**Solution**: Event sourcing pattern with immutable audit log

**Implementation:**

```sql
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255),

    -- Event details
    event_type VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,

    -- Operation details
    operation VARCHAR(20) NOT NULL CHECK (operation IN ('create', 'update', 'delete', 'access')),
    actor_id VARCHAR(255),
    actor_role VARCHAR(50),

    -- Changes
    old_values JSONB,
    new_values JSONB,

    -- Context
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(255),

    -- Temporal (immutable)
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Make audit log append-only (no updates/deletes)
CREATE RULE audit_log_no_update AS ON UPDATE TO audit_log DO INSTEAD NOTHING;
CREATE RULE audit_log_no_delete AS ON DELETE TO audit_log DO INSTEAD NOTHING;

CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_occurred ON audit_log(occurred_at DESC);
CREATE INDEX idx_audit_log_tenant ON audit_log(tenant_id);
```

**Trigger for Automatic Audit Logging:**

```sql
CREATE OR REPLACE FUNCTION audit_changes()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_log (
        tenant_id,
        event_type,
        entity_type,
        entity_id,
        operation,
        actor_id,
        old_values,
        new_values
    ) VALUES (
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        TG_TABLE_NAME || '_' || TG_OP,
        TG_TABLE_NAME,
        COALESCE(NEW.id::TEXT, OLD.id::TEXT),
        CASE TG_OP
            WHEN 'INSERT' THEN 'create'
            WHEN 'UPDATE' THEN 'update'
            WHEN 'DELETE' THEN 'delete'
        END,
        current_setting('app.current_user', true),
        CASE WHEN TG_OP = 'DELETE' THEN to_jsonb(OLD) ELSE NULL END,
        CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW) ELSE NULL END
    );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Apply to tables needing audit trail
CREATE TRIGGER audit_agent_profiles
    AFTER INSERT OR UPDATE OR DELETE ON agent_profiles
    FOR EACH ROW EXECUTE FUNCTION audit_changes();
```

**V2 Application:**

- CAWS verdict audit trail
- Agent registration tracking
- Performance data lineage

---

## Combined Patterns for V2

### Pattern 11: Hybrid Vector-Graph Search

**Problem**: Need both semantic similarity and relationship-based discovery

**Solution**: Unified search interface combining vector and graph queries

**Implementation:**

```sql
-- Unified search view
CREATE VIEW hybrid_search_index AS
SELECT
    'agent_capability' as entity_type,
    id,
    capability_name as name,
    embedding,
    confidence,
    metadata,
    NULL as relationship_data
FROM agent_capabilities_graph
UNION ALL
SELECT
    'caws_provenance' as entity_type,
    id,
    entity_id as name,
    embedding,
    1.0 as confidence,
    metadata,
    NULL as relationship_data
FROM caws_provenance_graph;

-- Hybrid search function
CREATE OR REPLACE FUNCTION hybrid_search(
    query_embedding vector(768),
    query_text TEXT,
    max_results INTEGER DEFAULT 10,
    include_graph_hops INTEGER DEFAULT 2
) RETURNS TABLE(
    entity_id UUID,
    entity_type VARCHAR(50),
    name VARCHAR(500),
    relevance_score DECIMAL(5,4),
    source VARCHAR(20)
) AS $$
BEGIN
    -- Step 1: Vector similarity search
    RETURN QUERY
    SELECT
        id as entity_id,
        entity_type,
        name,
        (1 - (embedding <=> query_embedding))::DECIMAL(5,4) as relevance_score,
        'vector'::VARCHAR(20) as source
    FROM hybrid_search_index
    WHERE embedding IS NOT NULL
    ORDER BY embedding <=> query_embedding
    LIMIT max_results;

    -- Step 2: Graph traversal from top vector results
    IF include_graph_hops > 0 THEN
        RETURN QUERY
        SELECT DISTINCT
            acg.id as entity_id,
            'agent_capability'::VARCHAR(50) as entity_type,
            acg.capability_name as name,
            (ar.confidence * ar.strength)::DECIMAL(5,4) as relevance_score,
            'graph'::VARCHAR(20) as source
        FROM agent_relationships ar
        JOIN agent_capabilities_graph acg ON acg.agent_id = ar.target_agent_id
        WHERE ar.source_agent_id IN (
            SELECT id FROM hybrid_search_index
            WHERE embedding IS NOT NULL
            ORDER BY embedding <=> query_embedding
            LIMIT 5
        )
        AND ar.confidence >= 0.7
        LIMIT max_results;
    END IF;
END;
$$ LANGUAGE plpgsql;
```

**Usage:**

```sql
-- Hybrid search for agent capabilities
SELECT * FROM hybrid_search(
    '[query_vector]'::vector,
    'TypeScript code generation',
    max_results => 20,
    include_graph_hops => 2
)
ORDER BY relevance_score DESC;
```

**V2 Application:**

- Agent capability discovery
- CAWS governance search
- Related task finding

---

## Performance Tuning Recommendations

### 1. Index Strategy

```sql
-- Vector indexes: HNSW for semantic search
CREATE INDEX CONCURRENTLY idx_capabilities_hnsw
    ON agent_capabilities_graph
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Graph indexes: Composite for traversal
CREATE INDEX CONCURRENTLY idx_relationships_source_target
    ON agent_relationships(source_agent_id, target_agent_id);

-- Time-series indexes: Partial for hot data
CREATE INDEX CONCURRENTLY idx_events_recent
    ON performance_events(timestamp DESC)
    WHERE timestamp > NOW() - INTERVAL '7 days';

-- JSONB indexes: GIN for containment queries
CREATE INDEX CONCURRENTLY idx_metadata_gin
    ON agent_capabilities_graph USING GIN (metadata);
```

### 2. Query Optimization

```sql
-- Use prepared statements for repeated queries
PREPARE capability_search (vector(768), INTEGER) AS
    SELECT * FROM agent_capabilities_graph
    ORDER BY embedding <=> $1
    LIMIT $2;

-- Set query planner parameters for vector search
SET LOCAL hnsw.ef_search = 40;  -- Lower for speed, higher for accuracy

-- Enable parallel query execution
SET LOCAL max_parallel_workers_per_gather = 4;
```

### 3. Connection Pooling

```typescript
// Configure connection pool for RLS
const pool = new Pool({
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
  application_name: "v2-arbiter",
});

// Helper for tenant-scoped queries
async function withTenantContext<T>(
  tenantId: string,
  callback: (client: PoolClient) => Promise<T>
): Promise<T> {
  const client = await pool.connect();
  try {
    await client.query(`SET LOCAL app.current_tenant = $1`, [tenantId]);
    return await callback(client);
  } finally {
    client.release();
  }
}
```

---

## Migration Strategy

### 1. Add Columns to Existing Tables

```sql
-- Migration: Add tenant_id to existing tables
BEGIN;

-- Add tenant_id column (nullable initially)
ALTER TABLE agent_profiles ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255);
ALTER TABLE performance_events ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255);
ALTER TABLE benchmark_datasets ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255);

-- Backfill with default tenant
UPDATE agent_profiles SET tenant_id = 'default-tenant' WHERE tenant_id IS NULL;
UPDATE performance_events SET tenant_id = 'default-tenant' WHERE tenant_id IS NULL;
UPDATE benchmark_datasets SET tenant_id = 'default-tenant' WHERE tenant_id IS NULL;

-- Make NOT NULL after backfill
ALTER TABLE agent_profiles ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE performance_events ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE benchmark_datasets ALTER COLUMN tenant_id SET NOT NULL;

-- Add foreign key constraints
ALTER TABLE agent_profiles ADD CONSTRAINT fk_agent_tenant
    FOREIGN KEY (tenant_id) REFERENCES tenants(id);

COMMIT;
```

### 2. Enable RLS Incrementally

```sql
-- Enable RLS one table at a time, test thoroughly
BEGIN;

-- Enable RLS
ALTER TABLE agent_profiles ENABLE ROW LEVEL SECURITY;

-- Create policies
CREATE POLICY tenant_access ON agent_profiles
    USING (tenant_id = current_setting('app.current_tenant', true)::VARCHAR);

-- Test with sample queries before committing
-- If issues found, ROLLBACK instead of COMMIT

COMMIT;
```

### 3. Migrate to Vector Search

```sql
-- Add embedding column to existing tables
ALTER TABLE agent_capabilities_graph ADD COLUMN embedding vector(768);

-- Backfill embeddings (do in batches to avoid long locks)
-- Use application code to generate embeddings
UPDATE agent_capabilities_graph
SET embedding = generate_embedding(capability_name)
WHERE id IN (
    SELECT id FROM agent_capabilities_graph
    WHERE embedding IS NULL
    LIMIT 1000
);

-- Create index CONCURRENTLY (doesn't block writes)
CREATE INDEX CONCURRENTLY idx_capabilities_embedding
    ON agent_capabilities_graph
    USING hnsw (embedding vector_cosine_ops);
```

---

## Testing Strategy

### 1. RLS Testing

```sql
-- Test tenant isolation
SET app.current_tenant = 'tenant-A';
SELECT COUNT(*) FROM agent_profiles; -- Should only see tenant A data

SET app.current_tenant = 'tenant-B';
SELECT COUNT(*) FROM agent_profiles; -- Should only see tenant B data

-- Attempt to bypass (should fail or return empty)
SELECT COUNT(*) FROM agent_profiles WHERE tenant_id = 'tenant-A'; -- Empty
```

### 2. Vector Search Performance Testing

```sql
-- Measure query performance
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM agent_capabilities_graph
ORDER BY embedding <=> '[test_vector]'::vector
LIMIT 10;

-- Should show:
-- - Index Scan using idx_capabilities_embedding
-- - Execution time < 100ms
```

### 3. Graph Traversal Testing

```sql
-- Test relationship queries
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM agent_relationships
WHERE source_agent_id = 'test-agent'
AND type = 'COLLABORATES_WITH';

-- Should use index idx_relationships_source_type
```

---

## Summary

V2 must adopt these patterns to achieve production-grade hybrid vector-graph database:

**Critical Patterns** (Must Have):

1. HNSW vector indexes for <100ms semantic search
2. Typed knowledge graph with strong constraints
3. Row Level Security for multi-tenant isolation
4. Entity-chunk provenance mapping
5. Hybrid vector-graph search interface

**Important Patterns** (Should Have): 6. Canonical name normalization 7. Search session analytics 8. Similarity caching 9. Privacy levels for federated learning 10. Audit logging with event sourcing

**Nice to Have** (Future Enhancements): 11. Compressed context offloading

**Next Steps:**

1. Implement migration 006: Knowledge graph schema
2. Implement migration 007: Multi-tenant isolation
3. Implement migration 008: Hybrid search views
4. Create TypeScript types matching schema
5. Write query pattern documentation

