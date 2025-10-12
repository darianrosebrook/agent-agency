<!-- cb8d62f4-7bb6-4513-98d9-2cbfb4511073 c0017744-4ae5-4cd1-b464-5880288f0533 -->
# V2 Database Layer CAWS Working Specification

## Overview

Create a production-grade CAWS working spec for V2's database layer that combines pgvector semantic search, knowledge graph relationships, and multi-tenant isolation patterns. This spec will formalize the database architecture needed for CAWS-compliant agent orchestration with discoverable governance.

## Phase 1: Documentation Analysis & Pattern Extraction (Read-Only)

### 1.1 Compare Existing Database Implementations

Create comprehensive comparison document analyzing three implementations:

**POC Implementation** (`iterations/poc/migrations/`):

- Multi-tenant memory system with pgvector (001_create_multi_tenant_schema.sql)
- Federated learning with privacy-preserving patterns
- Entity relationships with knowledge graphs (001_create_core_schema.sql)
- Row Level Security (RLS) for tenant isolation
- Pattern: Isolation levels (strict, shared, federated)

**Obsidian-RAG Implementation** (`obsidian-rag/apps/kv_database/`):

- Knowledge graph with entity extraction (schema.sql, migrations/001_*.sql)
- Hybrid vector + graph search architecture
- Entity types: PERSON, ORGANIZATION, CONCEPT, TECHNOLOGY
- Relationship types: WORKS_FOR, PART_OF, RELATED_TO, INFLUENCES
- Search session tracking and performance caching
- Pattern: Entity-chunk mappings for provenance

**V2 Current Implementation** (`iterations/v2/migrations/`):

- Performance tracking tables (004_create_performance_tracking_tables.sql)
- Agent registry with capabilities (001_create_agent_registry_tables.sql)
- Task queue and research provenance (002, 005)
- Pattern: JSONB flexibility with structured indexes

Output: `iterations/v2/docs/database/DATABASE-PATTERN-COMPARISON.md`

### 1.2 Extract Key Learnings

Document specific patterns that V2 should adopt:

**From Obsidian-RAG:**

- Knowledge graph entity types and relationships
- Vector similarity with HNSW indexes
- Entity-chunk mapping pattern for provenance
- Search session analytics
- Canonical name normalization triggers
- Similarity caching for performance

**From POC:**

- Multi-tenant isolation with RLS
- Federated learning tables
- Privacy levels (basic, differential, secure)
- Offloaded context compression
- Tenant-scoped queries with access policies

Output: `iterations/v2/docs/database/PATTERN-LEARNINGS.md`

## Phase 2: Database Architecture Design

### 2.1 Design Knowledge Graph Schema for Agent Governance

Create SQL schema for agent capability and provenance graph:

**Tables to Add:**

```sql
-- Agent Capability Graph
CREATE TABLE agent_capabilities_graph (
    id UUID PRIMARY KEY,
    agent_id VARCHAR(255) REFERENCES agent_profiles(id),
    capability_type entity_type, -- Reuse from obsidian-rag
    capability_name VARCHAR(500),
    canonical_name VARCHAR(500),
    confidence DECIMAL(3,2),
    embedding vector(768), -- pgvector for semantic search
    source_tasks UUID[], -- Which tasks demonstrated this
    validation_status VARCHAR(20),
    created_at TIMESTAMPTZ,
    metadata JSONB
);

-- Agent Relationship Graph
CREATE TABLE agent_relationships (
    id UUID PRIMARY KEY,
    source_agent_id VARCHAR(255),
    target_agent_id VARCHAR(255),
    relationship_type relationship_type, -- COLLABORATES_WITH, SIMILAR_TO, COMPETES_WITH
    strength DECIMAL(3,2),
    confidence DECIMAL(3,2),
    cooccurrence_count INTEGER,
    supporting_tasks UUID[],
    created_at TIMESTAMPTZ,
    metadata JSONB
);

-- CAWS Provenance Graph
CREATE TABLE caws_provenance_graph (
    id UUID PRIMARY KEY,
    entity_type VARCHAR(50), -- 'verdict', 'waiver', 'gate', 'spec'
    entity_id VARCHAR(255),
    parent_entity_id UUID REFERENCES caws_provenance_graph(id),
    hash_chain VARCHAR(128), -- SHA-256 of parent for immutability
    signature VARCHAR(255), -- ed25519 signature
    constitutional_refs TEXT[], -- ["CAWS:Section4.2"]
    embedding vector(768), -- For semantic discovery
    created_at TIMESTAMPTZ,
    metadata JSONB
);
```

Output: `iterations/v2/migrations/006_create_knowledge_graph_schema.sql`

### 2.2 Design Multi-Tenant Isolation

Add tenant isolation to existing tables:

```sql
-- Add tenant isolation to existing tables
ALTER TABLE agent_profiles ADD COLUMN tenant_id VARCHAR(255);
ALTER TABLE performance_events ADD COLUMN tenant_id VARCHAR(255);
ALTER TABLE benchmark_datasets ADD COLUMN tenant_id VARCHAR(255);

-- Create tenant management table
CREATE TABLE tenants (
    id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255),
    name VARCHAR(255),
    isolation_level VARCHAR(50) CHECK (isolation_level IN ('strict', 'shared', 'federated')),
    access_policies JSONB,
    sharing_rules JSONB,
    data_retention JSONB,
    encryption_enabled BOOLEAN,
    created_at TIMESTAMPTZ
);

-- Enable RLS on performance tables
ALTER TABLE performance_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE benchmark_datasets ENABLE ROW LEVEL SECURITY;
```

Output: `iterations/v2/migrations/007_add_multi_tenant_isolation.sql`

### 2.3 Design Hybrid Vector Search Architecture

```sql
-- Create unified search view combining vector + graph
CREATE VIEW hybrid_search_index AS
SELECT 
    'agent_capability' as entity_type,
    id,
    capability_name as name,
    embedding,
    confidence,
    metadata
FROM agent_capabilities_graph
UNION ALL
SELECT
    'caws_provenance' as entity_type,
    id,
    entity_id as name,
    embedding,
    1.0 as confidence,
    metadata
FROM caws_provenance_graph;

-- Create graph traversal function
CREATE OR REPLACE FUNCTION traverse_agent_relationships(
    start_agent_id VARCHAR(255),
    max_hops INTEGER DEFAULT 2
) RETURNS TABLE(...) AS $$
-- Implementation for multi-hop graph queries
$$;
```

Output: `iterations/v2/migrations/008_create_hybrid_search_views.sql`

## Phase 3: Create CAWS Working Specification

### 3.1 Create Database Layer Working Spec

Create comprehensive CAWS spec at `iterations/v2/.caws/database-layer-spec.yaml`:

```yaml
id: V2-DATABASE-LAYER
title: V2 Database Layer - Hybrid Vector-Graph with Multi-Tenant Isolation
risk_tier: 1  # Critical infrastructure
mode: feature
change_budget:
  max_files: 25
  max_loc: 3000
blast_radius:
  modules:
    - iterations/v2/migrations/
    - iterations/v2/src/database/
    - iterations/v2/src/orchestrator/DatabaseClient.ts
  data_migration: true  # New tables and RLS policies
operational_rollback_slo: 30m
threats:
  - Data loss during migration
  - RLS policy misconfiguration exposing tenant data
  - Vector index performance degradation
  - Graph traversal performance issues
  - Provenance chain corruption
scope:
  in:
    - iterations/v2/migrations/006_*.sql
    - iterations/v2/migrations/007_*.sql
    - iterations/v2/migrations/008_*.sql
    - iterations/v2/src/database/KnowledgeGraphClient.ts
    - iterations/v2/src/database/MultiTenantClient.ts
  out:
    - iterations/poc/
    - obsidian-rag/
invariants:
  - All existing data remains accessible after migration
  - Tenant isolation must be enforced at database level via RLS
  - Provenance chains maintain cryptographic integrity
  - Vector search performance <100ms P95
  - Graph traversal <200ms for 2-hop queries
acceptance:
  - id: DB-VEC-001
    given: Agent capability stored with embedding
    when: Semantic search query executed
    then: Similar capabilities returned within 100ms using HNSW index
  - id: DB-GRAPH-001
    given: Agent relationships in graph
    when: 2-hop traversal query executed
    then: Related agents discovered within 200ms with confidence scores
  - id: DB-TENANT-001
    given: Multiple tenants with data
    when: Tenant A queries database
    then: Tenant B data completely inaccessible via RLS policies
  - id: DB-PROV-001
    given: CAWS verdict recorded
    when: Provenance chain validated
    then: Hash chain integrity verified and constitutional refs discoverable
non_functional:
  perf:
    vector_search_p95_ms: 100
    graph_traversal_2hop_ms: 200
    tenant_isolation_overhead_ms: 10
    provenance_write_ms: 50
  security:
    - row-level-security
    - tenant-isolation
    - data-encryption-at-rest
    - provenance-chain-integrity
  reliability:
    migration_success_rate: 1.0
    data_integrity: 1.0
contracts:
  - type: sql
    path: iterations/v2/migrations/006_create_knowledge_graph_schema.sql
  - type: sql
    path: iterations/v2/migrations/007_add_multi_tenant_isolation.sql
  - type: typescript
    path: iterations/v2/src/types/database-types.ts
```

Output: `iterations/v2/.caws/database-layer-spec.yaml`

### 3.2 Create Database Schema Documentation

Comprehensive documentation of complete schema:

- Entity-Relationship diagrams
- Table descriptions and constraints
- Index strategy and performance characteristics
- Migration order and dependencies
- Rollback procedures
- Query patterns and examples

Output: `iterations/v2/docs/database/SCHEMA-DOCUMENTATION.md`

### 3.3 Create Database Type Definitions

TypeScript types matching the schema:

```typescript
// iterations/v2/src/types/database-types.ts

export interface AgentCapabilityGraph {
  id: string;
  agentId: string;
  capabilityType: EntityType;
  capabilityName: string;
  canonicalName: string;
  confidence: number;
  embedding: number[]; // 768-dim vector
  sourceTasks: string[];
  validationStatus: 'validated' | 'unvalidated' | 'rejected';
  createdAt: Date;
  metadata: Record<string, unknown>;
}

export interface AgentRelationship {
  id: string;
  sourceAgentId: string;
  targetAgentId: string;
  relationshipType: RelationshipType;
  strength: number;
  confidence: number;
  cooccurrenceCount: number;
  supportingTasks: string[];
  createdAt: Date;
  metadata: Record<string, unknown>;
}

export interface CAWSProvenanceNode {
  id: string;
  entityType: 'verdict' | 'waiver' | 'gate' | 'spec';
  entityId: string;
  parentEntityId?: string;
  hashChain: string;
  signature: string;
  constitutionalRefs: string[];
  embedding: number[];
  createdAt: Date;
  metadata: Record<string, unknown>;
}

export enum EntityType {
  CAPABILITY = 'CAPABILITY',
  AGENT = 'AGENT',
  TASK = 'TASK',
  VERDICT = 'VERDICT',
  TECHNOLOGY = 'TECHNOLOGY'
}

export enum RelationshipType {
  COLLABORATES_WITH = 'COLLABORATES_WITH',
  SIMILAR_TO = 'SIMILAR_TO',
  DERIVED_FROM = 'DERIVED_FROM',
  VALIDATES = 'VALIDATES',
  DEPENDS_ON = 'DEPENDS_ON'
}
```

Output: `iterations/v2/src/types/database-types.ts`

## Phase 4: Implementation Recommendations

### 4.1 Create Migration Plan Document

Detailed migration execution plan:

1. Backup existing database
2. Run migrations 006, 007, 008 in sequence
3. Backfill tenant_id for existing data
4. Enable RLS policies incrementally
5. Validate data integrity
6. Performance test vector and graph queries
7. Rollback procedures if issues found

Output: `iterations/v2/docs/database/MIGRATION-PLAN.md`

### 4.2 Create Query Pattern Examples

Document common query patterns:

**Semantic Capability Discovery:**

```sql
-- Find agents with similar capabilities
SELECT * FROM agent_capabilities_graph
ORDER BY embedding <=> '[query_embedding]'::vector
LIMIT 10;
```

**Graph Relationship Traversal:**

```sql
-- Find related agents within 2 hops
SELECT * FROM traverse_agent_relationships('agent-123', 2);
```

**CAWS Provenance Discovery:**

```sql
-- Find all waivers related to a verdict
SELECT * FROM caws_provenance_graph
WHERE parent_entity_id IN (
  SELECT id FROM caws_provenance_graph 
  WHERE entity_id = 'VERDICT-001'
);
```

Output: `iterations/v2/docs/database/QUERY-PATTERNS.md`

## Success Criteria

- CAWS database layer spec created and validated
- All patterns documented with comparison analysis
- Migration SQL files ready for implementation
- TypeScript types aligned with schema
- Query patterns documented and performance-tested
- Multi-tenant isolation verified via RLS
- Vector search <100ms P95
- Graph traversal <200ms for 2-hop queries
- Zero data loss during migration
- Complete rollback procedures documented

## Files to Create

1. `iterations/v2/docs/database/DATABASE-PATTERN-COMPARISON.md`
2. `iterations/v2/docs/database/PATTERN-LEARNINGS.md`
3. `iterations/v2/migrations/006_create_knowledge_graph_schema.sql`
4. `iterations/v2/migrations/007_add_multi_tenant_isolation.sql`
5. `iterations/v2/migrations/008_create_hybrid_search_views.sql`
6. `iterations/v2/.caws/database-layer-spec.yaml`
7. `iterations/v2/docs/database/SCHEMA-DOCUMENTATION.md`
8. `iterations/v2/src/types/database-types.ts`
9. `iterations/v2/docs/database/MIGRATION-PLAN.md`
10. `iterations/v2/docs/database/QUERY-PATTERNS.md`

## Key Design Decisions

1. **Hybrid Architecture**: Vector search for semantic discovery + graph for relationships = discoverable governance
2. **Multi-Tenant First**: RLS policies at database level, not application level
3. **CAWS Native**: Provenance as first-class graph entities with hash chains
4. **Performance**: HNSW indexes for vector search, specialized indexes for graph traversal
5. **Extensibility**: JSONB metadata for future flexibility without schema changes