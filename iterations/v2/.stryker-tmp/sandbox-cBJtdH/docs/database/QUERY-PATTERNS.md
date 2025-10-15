# Database Query Patterns for V2 Hybrid Vector-Graph Architecture

**Author**: @darianrosebrook  
**Date**: 2025-10-12  
**Purpose**: Documented query patterns with performance expectations

---

## Overview

This document provides tested query patterns for V2's hybrid vector-graph database. Each pattern includes:

- SQL query example
- TypeScript/JavaScript usage
- Performance expectations
- Common pitfalls and optimizations

---

## Table of Contents

1. [Vector Search Patterns](#vector-search-patterns)
2. [Graph Traversal Patterns](#graph-traversal-patterns)
3. [Hybrid Search Patterns](#hybrid-search-patterns)
4. [Multi-Tenant Patterns](#multi-tenant-patterns)
5. [CAWS Provenance Patterns](#caws-provenance-patterns)
6. [Analytics and Aggregation Patterns](#analytics-and-aggregation-patterns)
7. [Performance Optimization Patterns](#performance-optimization-patterns)

---

## Vector Search Patterns

### Pattern 1: Semantic Capability Search

**Use Case**: Find agents with similar capabilities using vector embeddings

**SQL Query**:

```sql
-- Basic semantic search
SELECT
    id,
    capability_name,
    agent_id,
    confidence,
    (1 - (embedding <=> $1::vector(768))) AS similarity_score
FROM agent_capabilities_graph
WHERE embedding IS NOT NULL
    AND tenant_id = $2
    AND confidence >= $3
ORDER BY embedding <=> $1::vector(768)
LIMIT $4;
```

**Parameters**:

- `$1`: Query embedding (768-dimensional vector)
- `$2`: Tenant ID for isolation
- `$3`: Minimum confidence threshold (e.g., 0.7)
- `$4`: Maximum results (e.g., 10)

**TypeScript Usage**:

```typescript
import { pool } from "@/database/connection";

async function findSimilarCapabilities(
  queryEmbedding: number[],
  tenantId: string,
  minConfidence: number = 0.7,
  limit: number = 10
): Promise<SimilarCapability[]> {
  const client = await pool.connect();
  try {
    // Set tenant context for RLS
    await client.query(`SET LOCAL app.current_tenant = $1`, [tenantId]);

    const result = await client.query(
      `
      SELECT 
        id,
        capability_name,
        agent_id,
        confidence,
        (1 - (embedding <=> $1::vector(768))) AS similarity_score
      FROM agent_capabilities_graph
      WHERE embedding IS NOT NULL
        AND confidence >= $2
      ORDER BY embedding <=> $1::vector(768)
      LIMIT $3
    `,
      [JSON.stringify(queryEmbedding), minConfidence, limit]
    );

    return result.rows;
  } finally {
    client.release();
  }
}
```

**Performance Expectations**:

- **P50**: 10-20ms
- **P95**: 50-100ms
- **P99**: 100-150ms

**Index Used**: `idx_capabilities_embedding` (HNSW)

**Optimization Tips**:

- Adjust `hnsw.ef_search` parameter for speed vs accuracy trade-off:
  ```sql
  SET LOCAL hnsw.ef_search = 40; -- Default
  SET LOCAL hnsw.ef_search = 20; -- Faster, less accurate
  SET LOCAL hnsw.ef_search = 100; -- Slower, more accurate
  ```
- Use prepared statements to avoid query parsing overhead
- Batch similar queries together to leverage connection pooling

---

### Pattern 2: Find Similar CAWS Verdicts

**Use Case**: Discover previous CAWS verdicts similar to current case

**SQL Query**:

```sql
-- Semantic verdict discovery
SELECT
    id as verdict_id,
    entity_id,
    entity_type,
    constitutional_refs,
    (1 - (embedding <=> $1::vector(768))) AS similarity_score,
    evidence_completeness,
    budget_adherence,
    created_at
FROM caws_provenance_graph
WHERE embedding IS NOT NULL
    AND entity_type = 'verdict'
    AND (1 - (embedding <=> $1::vector(768))) >= $2
ORDER BY embedding <=> $1::vector(768)
LIMIT $3;
```

**TypeScript Usage**:

```typescript
async function findSimilarVerdicts(
  situationEmbedding: number[],
  minSimilarity: number = 0.75,
  limit: number = 5
): Promise<SimilarCAWSVerdict[]> {
  const result = await pool.query(
    `
    SELECT 
      id,
      entity_id,
      constitutional_refs,
      (1 - (embedding <=> $1::vector(768))) AS similarity_score
    FROM caws_provenance_graph
    WHERE embedding IS NOT NULL
      AND entity_type = 'verdict'
    ORDER BY embedding <=> $1::vector(768)
    LIMIT $2
  `,
    [JSON.stringify(situationEmbedding), limit]
  );

  return result.rows;
}
```

**Performance**: <100ms P95

---

## Graph Traversal Patterns

### Pattern 3: Multi-Hop Agent Relationship Traversal

**Use Case**: Find related agents within N hops through the relationship graph

**SQL Query**:

```sql
-- Using the traverse_agent_relationships function
SELECT * FROM traverse_agent_relationships(
    $1,  -- start_agent_id
    $2,  -- max_hops (default 2)
    $3,  -- min_confidence (default 0.5)
    $4   -- relationship_types array (NULL for all)
)
ORDER BY hop_distance, cumulative_confidence DESC;
```

**TypeScript Usage**:

```typescript
async function findRelatedAgents(
  startAgentId: string,
  maxHops: number = 2,
  minConfidence: number = 0.7,
  relationshipTypes?: RelationshipType[]
): Promise<GraphTraversalResult[]> {
  const result = await pool.query(
    `
    SELECT * FROM traverse_agent_relationships($1, $2, $3, $4)
    ORDER BY hop_distance, cumulative_confidence DESC
  `,
    [startAgentId, maxHops, minConfidence, relationshipTypes || null]
  );

  return result.rows;
}
```

**Example - Find Collaborators**:

```typescript
// Find agents that collaborate with agent-123
const collaborators = await findRelatedAgents(
  "agent-123",
  1, // Only direct relationships
  0.8, // High confidence
  [RelationshipType.COLLABORATES_WITH]
);
```

**Performance Expectations**:

- **1-hop**: 10-30ms
- **2-hop**: 50-150ms
- **3-hop**: 100-300ms (use with caution)

**Index Used**: `idx_relationships_source_type`, `idx_relationships_target_type`

---

### Pattern 4: Shortest Path Between Agents

**Use Case**: Find how two agents are connected

**SQL Query**:

```sql
-- Find shortest path between two agents
SELECT * FROM find_agent_path(
    $1,  -- source_agent_id
    $2,  -- target_agent_id
    $3   -- max_hops (default 5)
)
LIMIT 1; -- Only first (shortest) path
```

**TypeScript Usage**:

```typescript
async function findConnectionPath(
  sourceAgentId: string,
  targetAgentId: string,
  maxHops: number = 5
): Promise<AgentPath | null> {
  const result = await pool.query(
    `
    SELECT * FROM find_agent_path($1, $2, $3)
    LIMIT 1
  `,
    [sourceAgentId, targetAgentId, maxHops]
  );

  return result.rows[0] || null;
}
```

**Example**:

```typescript
const path = await findConnectionPath("agent-alpha", "agent-beta", 3);
if (path) {
  console.log(`Path length: ${path.pathLength} hops`);
  console.log(`Relationships: ${path.relationshipPath.join(" -> ")}`);
  console.log(`Confidence: ${path.totalConfidence}`);
}
```

**Performance**: <200ms for up to 5 hops

---

## Hybrid Search Patterns

### Pattern 5: Combined Vector + Graph Search

**Use Case**: Search using both semantic similarity and relationship context

**SQL Query**:

```sql
-- Hybrid search combining vector and graph
SELECT * FROM hybrid_search(
    $1::vector(768),  -- query_embedding
    $2,               -- query_text (optional)
    $3,               -- max_results
    $4,               -- include_graph_hops
    $5,               -- entity_types array
    $6,               -- tenant_id
    $7                -- min_confidence
)
ORDER BY relevance_score DESC;
```

**TypeScript Usage**:

```typescript
async function hybridCapabilitySearch(
  query: string,
  queryEmbedding: number[],
  tenantId: string,
  options: {
    maxResults?: number;
    includeGraphHops?: number;
    minConfidence?: number;
  } = {}
): Promise<HybridSearchResult[]> {
  const {
    maxResults = 20,
    includeGraphHops = 2,
    minConfidence = 0.7,
  } = options;

  const result = await pool.query(
    `
    SELECT * FROM hybrid_search(
      $1::vector(768),
      $2,
      $3,
      $4,
      ARRAY['agent_capability']::VARCHAR(50)[],
      $5,
      $6
    )
    ORDER BY relevance_score DESC
  `,
    [
      JSON.stringify(queryEmbedding),
      query,
      maxResults,
      includeGraphHops,
      tenantId,
      minConfidence,
    ]
  );

  return result.rows;
}
```

**Example**:

```typescript
// Search for TypeScript capabilities
const embedding = await generateEmbedding("TypeScript code generation");
const results = await hybridCapabilitySearch(
  "TypeScript code generation",
  embedding,
  "tenant-123",
  {
    maxResults: 20,
    includeGraphHops: 2, // Include related agents
    minConfidence: 0.7,
  }
);

// Results include both:
// - Direct matches (source: 'vector')
// - Related capabilities via agent relationships (source: 'graph')
```

**Performance**: <250ms P95

---

## Multi-Tenant Patterns

### Pattern 6: Tenant-Scoped Queries

**Use Case**: Query data with automatic tenant isolation via RLS

**SQL Query**:

```sql
-- Set tenant context (must do this for every query)
SET LOCAL app.current_tenant = 'tenant-123';

-- Now all queries automatically filtered by RLS
SELECT * FROM agent_profiles WHERE id = 'agent-456';
-- Only returns if agent belongs to tenant-123
```

**TypeScript Helper**:

```typescript
/**
 * Execute query with tenant context
 * Ensures RLS policies are applied
 */
export async function withTenantContext<T>(
  tenantId: string,
  callback: (client: PoolClient) => Promise<T>
): Promise<T> {
  const client = await pool.connect();
  try {
    // Set tenant context for this connection
    await client.query(`SET LOCAL app.current_tenant = $1`, [tenantId]);

    // Execute queries within tenant context
    return await callback(client);
  } finally {
    client.release();
  }
}

// Usage
const agentProfiles = await withTenantContext("tenant-123", async (client) => {
  const result = await client.query(`
    SELECT * FROM agent_profiles
    WHERE status = 'active'
  `);
  return result.rows;
});
```

**Critical**: Always use `withTenantContext` for tenant-scoped queries. Never construct WHERE clauses manually with `tenant_id`.

---

### Pattern 7: Cross-Tenant Federated Aggregation

**Use Case**: Aggregate data across tenants with privacy preservation

**SQL Query**:

```sql
-- Federated aggregation with differential privacy
SELECT
    capability_type,
    add_dp_noise(AVG(confidence)::DECIMAL, $1) as avg_confidence_private,
    COUNT(*) as count
FROM agent_capabilities_graph
WHERE (
    SELECT isolation_level FROM tenants WHERE id = tenant_id
) = 'federated'
GROUP BY capability_type
HAVING check_k_anonymity(COUNT(*), $1);
```

**TypeScript Usage**:

```typescript
async function getFederatedCapabilityStats(
  tenantId: string
): Promise<{ capabilityType: string; avgConfidence: number; count: number }[]> {
  const result = await pool.query(
    `
    SELECT 
      capability_type,
      add_dp_noise(AVG(confidence)::DECIMAL, $1) as avg_confidence,
      COUNT(*) as count
    FROM agent_capabilities_graph
    WHERE (
      SELECT isolation_level FROM tenants WHERE id = tenant_id
    ) = 'federated'
    GROUP BY capability_type
    HAVING check_k_anonymity(COUNT(*), $1)
  `,
    [tenantId]
  );

  return result.rows;
}
```

**Privacy Guarantees**:

- Differential privacy noise added to aggregates
- K-anonymity ensures minimum group size
- No raw data exposed

---

## CAWS Provenance Patterns

### Pattern 8: Verify Provenance Chain Integrity

**Use Case**: Validate hash chain for immutability verification

**SQL Query**:

```sql
-- Traverse provenance chain and verify hashes
WITH RECURSIVE chain AS (
    -- Start from target node
    SELECT
        id,
        entity_id,
        parent_entity_id,
        hash_chain,
        constitutional_refs,
        1 as depth,
        ARRAY[id] as path
    FROM caws_provenance_graph
    WHERE id = $1

    UNION ALL

    -- Traverse to parents
    SELECT
        p.id,
        p.entity_id,
        p.parent_entity_id,
        p.hash_chain,
        p.constitutional_refs,
        c.depth + 1,
        c.path || p.id
    FROM caws_provenance_graph p
    JOIN chain c ON p.id = c.parent_entity_id
    WHERE NOT p.id = ANY(c.path) -- Prevent cycles
)
SELECT
    id,
    entity_id,
    hash_chain,
    parent_entity_id,
    depth,
    -- Verify hash matches expected
    compute_provenance_hash(
        (SELECT hash_chain FROM chain WHERE id = parent_entity_id),
        entity_type,
        entity_id,
        metadata
    ) as expected_hash,
    hash_chain = compute_provenance_hash(
        (SELECT hash_chain FROM chain WHERE id = parent_entity_id),
        entity_type,
        entity_id,
        metadata
    ) as hash_valid
FROM chain
ORDER BY depth DESC;
```

**TypeScript Usage**:

```typescript
async function verifyProvenanceChain(
  nodeId: string
): Promise<{ valid: boolean; brokenAt?: string }> {
  const result = await pool.query(
    `
    WITH RECURSIVE chain AS (
      SELECT 
        id,
        entity_id,
        parent_entity_id,
        hash_chain,
        1 as depth
      FROM caws_provenance_graph
      WHERE id = $1
      
      UNION ALL
      
      SELECT 
        p.id,
        p.entity_id,
        p.parent_entity_id,
        p.hash_chain,
        c.depth + 1
      FROM caws_provenance_graph p
      JOIN chain c ON p.id = c.parent_entity_id
    )
    SELECT * FROM chain
  `,
    [nodeId]
  );

  // Verify each hash in chain
  for (const node of result.rows) {
    const expectedHash = computeHash(node);
    if (node.hash_chain !== expectedHash) {
      return {
        valid: false,
        brokenAt: node.id,
      };
    }
  }

  return { valid: true };
}
```

---

### Pattern 9: Find Constitutional Precedents

**Use Case**: Discover CAWS decisions relevant to current situation

**SQL Query**:

```sql
-- Search provenance by constitutional clause
SELECT
    id,
    entity_id,
    entity_type,
    constitutional_refs,
    created_at,
    evidence_completeness,
    budget_adherence
FROM caws_provenance_graph
WHERE $1 = ANY(constitutional_refs)
ORDER BY created_at DESC
LIMIT $2;
```

**TypeScript Usage**:

```typescript
async function findPrecedentsByClause(
  constitutionalClause: string,
  limit: number = 10
): Promise<CAWSProvenanceNode[]> {
  const result = await pool.query(
    `
    SELECT * FROM caws_provenance_graph
    WHERE $1 = ANY(constitutional_refs)
    ORDER BY created_at DESC
    LIMIT $2
  `,
    [constitutionalClause, limit]
  );

  return result.rows;
}

// Example: Find all verdicts citing Section 4.2
const precedents = await findPrecedentsByClause("CAWS:Section4.2", 10);
```

---

## Analytics and Aggregation Patterns

### Pattern 10: Agent Capability Summary

**Use Case**: Get overview of agent's capabilities

**SQL Query**:

```sql
-- Using pre-built view
SELECT * FROM agent_capability_summary
WHERE agent_id = $1;
```

**TypeScript Usage**:

```typescript
async function getAgentCapabilitySummary(
  agentId: string
): Promise<AgentCapabilitySummary> {
  const result = await pool.query(
    `
    SELECT * FROM agent_capability_summary
    WHERE agent_id = $1
  `,
    [agentId]
  );

  return result.rows[0];
}
```

---

### Pattern 11: Agent Relationship Analytics

**Use Case**: Understand agent's position in collaboration network

**SQL Query**:

```sql
-- Get connectivity metrics
SELECT
    ac.agent_id,
    ac.outbound_relationships,
    ac.inbound_relationships,
    ac.total_relationships,
    cent.degree_centrality,
    cent.betweenness_estimate
FROM agent_connectivity ac
JOIN compute_agent_centrality() cent ON ac.agent_id = cent.agent_id
WHERE ac.agent_id = $1;
```

**TypeScript Usage**:

```typescript
async function getAgentNetworkPosition(agentId: string): Promise<{
  connectivity: AgentConnectivity;
  centrality: AgentCentrality;
}> {
  const result = await pool.query(
    `
    SELECT 
      ac.*,
      cent.degree_centrality,
      cent.betweenness_estimate
    FROM agent_connectivity ac
    JOIN compute_agent_centrality() cent ON ac.agent_id = cent.agent_id
    WHERE ac.agent_id = $1
  `,
    [agentId]
  );

  return result.rows[0];
}
```

---

## Performance Optimization Patterns

### Pattern 12: Batch Vector Searches

**Use Case**: Search for multiple embeddings efficiently

**TypeScript Implementation**:

```typescript
async function batchVectorSearch(
  queries: { embedding: number[]; tenantId: string }[],
  limit: number = 10
): Promise<Map<number, SimilarCapability[]>> {
  const results = new Map<number, SimilarCapability[]>();

  // Use single connection with pipelining
  const client = await pool.connect();
  try {
    const promises = queries.map(async (query, index) => {
      await client.query(`SET LOCAL app.current_tenant = $1`, [query.tenantId]);
      const result = await client.query(
        `
        SELECT * FROM find_similar_capabilities($1::vector(768), NULL, $2)
      `,
        [JSON.stringify(query.embedding), limit]
      );

      results.set(index, result.rows);
    });

    await Promise.all(promises);
  } finally {
    client.release();
  }

  return results;
}
```

---

### Pattern 13: Materialized View Refresh Strategy

**Use Case**: Keep hybrid search index up-to-date

**SQL**:

```sql
-- Concurrent refresh (doesn't block reads)
REFRESH MATERIALIZED VIEW CONCURRENTLY hybrid_search_materialized;
```

**TypeScript Cron Job**:

```typescript
import { CronJob } from "cron";

// Refresh every 30 minutes
new CronJob("*/30 * * * *", async () => {
  try {
    await pool.query(`
      REFRESH MATERIALIZED VIEW CONCURRENTLY hybrid_search_materialized
    `);
    console.log("Hybrid search index refreshed");
  } catch (error) {
    console.error("Failed to refresh hybrid search index:", error);
  }
}).start();
```

---

### Pattern 14: Query Performance Monitoring

**Use Case**: Track slow queries for optimization

**SQL**:

```sql
-- Log search session
INSERT INTO graph_search_sessions (
    query_text,
    query_hash,
    search_type,
    result_count,
    execution_time_ms,
    tenant_id
) VALUES ($1, $2, $3, $4, $5, $6);

-- Analyze slow queries
SELECT * FROM slow_search_queries
WHERE avg_time_ms > 200
ORDER BY avg_time_ms DESC;
```

**TypeScript Middleware**:

```typescript
async function monitoredSearch<T>(
  searchType: SearchType,
  queryText: string,
  tenantId: string,
  searchFn: () => Promise<T>
): Promise<T> {
  const startTime = Date.now();

  try {
    const results = await searchFn();
    const executionTime = Date.now() - startTime;

    // Log session
    await pool.query(
      `
      SELECT log_search_session($1, $2, $3, $4, $5)
    `,
      [
        queryText,
        searchType,
        Array.isArray(results) ? results.length : 1,
        executionTime,
        tenantId,
      ]
    );

    return results;
  } catch (error) {
    const executionTime = Date.now() - startTime;

    // Log failed search
    await pool.query(
      `
      SELECT log_search_session($1, $2, 0, $3, $4)
    `,
      [queryText, searchType, executionTime, tenantId]
    );

    throw error;
  }
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting Tenant Context

❌ **Wrong**:

```typescript
// Bypasses RLS, sees all tenants' data
const agents = await pool.query("SELECT * FROM agent_profiles");
```

✅ **Correct**:

```typescript
// Uses RLS, sees only tenant's data
await withTenantContext("tenant-123", async (client) => {
  const result = await client.query("SELECT * FROM agent_profiles");
  return result.rows;
});
```

### Pitfall 2: Vector Embedding Format

❌ **Wrong**:

```typescript
// Passing array directly fails
await pool.query(`
  SELECT * FROM agent_capabilities_graph
  ORDER BY embedding <=> $1
`, [[0.1, 0.2, ...]]); // Wrong!
```

✅ **Correct**:

```typescript
// Convert to JSON string first
await pool.query(`
  SELECT * FROM agent_capabilities_graph
  ORDER BY embedding <=> $1::vector(768)
`, [JSON.stringify([0.1, 0.2, ...])]);
```

### Pitfall 3: Deep Graph Traversal

❌ **Wrong**:

```typescript
// 5-hop traversal can be slow
await findRelatedAgents("agent-123", 5);
```

✅ **Correct**:

```typescript
// Limit to 2-3 hops, increase confidence threshold
await findRelatedAgents("agent-123", 2, 0.8);
```

---

## Summary of Performance Expectations

| Pattern                 | P50   | P95   | P99   | Index Used |
| ----------------------- | ----- | ----- | ----- | ---------- |
| Vector Search           | 15ms  | 80ms  | 120ms | HNSW       |
| Graph Traversal (2-hop) | 60ms  | 180ms | 250ms | Composite  |
| Hybrid Search           | 100ms | 240ms | 350ms | Multiple   |
| Tenant Query (RLS)      | +5ms  | +10ms | +15ms | N/A        |
| Provenance Verification | 20ms  | 60ms  | 100ms | Hash index |

---

**Next Steps**: See `SCHEMA-DOCUMENTATION.md` for complete schema reference and ER diagrams.

