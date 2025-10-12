# V2 Database Migration Execution Plan

**Author**: @darianrosebrook  
**Date**: 2025-10-12  
**Status**: Ready for Execution  
**Risk Tier**: 1 (Critical Infrastructure)

---

## Executive Summary

This document outlines the step-by-step execution plan for migrating V2's database to a hybrid vector-graph architecture with multi-tenant isolation. The migration introduces three major changes:

1. **Knowledge Graph Schema** (Migration 006): Adds agent capabilities graph, relationship graph, and CAWS provenance graph with pgvector embeddings
2. **Multi-Tenant Isolation** (Migration 007): Implements Row Level Security (RLS) for tenant data separation
3. **Hybrid Search Architecture** (Migration 008): Creates unified search views and graph traversal functions

**Total Estimated Duration**: 4-6 hours (including validation)  
**Rollback SLO**: 30 minutes  
**Data Loss Tolerance**: Zero

---

## Pre-Migration Checklist

### Environment Validation

- [ ] **PostgreSQL Version**: >= 13 with pgvector extension available
- [ ] **Database Backup**: Full backup completed and verified
- [ ] **Backup Restoration Tested**: Backup successfully restored to test database
- [ ] **Extension Availability**: `pgvector`, `uuid-ossp`, `pg_trgm` extensions installable
- [ ] **Disk Space**: At least 20GB free (estimated 10GB for indexes + 10GB safety margin)
- [ ] **Connection Pool**: Can handle 50+ concurrent connections
- [ ] **Maintenance Window**: 6-hour window scheduled with stakeholder approval

### Application Preparation

- [ ] **Application Shutdown**: All V2 services gracefully stopped
- [ ] **Connection Drain**: All database connections closed
- [ ] **Health Checks Disabled**: Prevent reconnection attempts during migration
- [ ] **Monitoring Alerts**: Silenced for migration window
- [ ] **Rollback Scripts**: Prepared and tested on staging database

### Team Readiness

- [ ] **Migration Lead**: On-call and ready to execute
- [ ] **Database Administrator**: Available for troubleshooting
- [ ] **Application Owner**: Available for post-migration testing
- [ ] **Communication Channel**: Incident channel active
- [ ] **Runbook**: This document reviewed by all participants

---

## Migration Execution Steps

### Phase 1: Pre-Migration Validation (15 minutes)

#### 1.1 Create Full Database Backup

```bash
# Create timestamped backup
BACKUP_FILE="v2_backup_$(date +%Y%m%d_%H%M%S).sql"
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME -F c -f $BACKUP_FILE

# Verify backup integrity
pg_restore --list $BACKUP_FILE | wc -l
# Should show non-zero count of database objects

# Store backup securely
aws s3 cp $BACKUP_FILE s3://v2-backups/migrations/

# Record backup metadata
echo "Backup: $BACKUP_FILE" > migration_metadata.txt
echo "Timestamp: $(date)" >> migration_metadata.txt
echo "DB Size: $(du -h $BACKUP_FILE)" >> migration_metadata.txt
```

#### 1.2 Verify Database State

```sql
-- Check current schema version
SELECT MAX(version) FROM schema_migrations;
-- Expected: 005 (last migration before this change)

-- Count existing records (for post-migration validation)
SELECT
    'agent_profiles' as table_name, COUNT(*) as row_count
FROM agent_profiles
UNION ALL
SELECT 'performance_events', COUNT(*) FROM performance_events
UNION ALL
SELECT 'benchmark_datasets', COUNT(*) FROM benchmark_datasets;

-- Save counts for later validation
\copy (SELECT 'agent_profiles', COUNT(*) FROM agent_profiles) TO '/tmp/pre_migration_counts.csv' CSV
```

#### 1.3 Test Migrations on Staging

```bash
# Restore backup to staging database
pg_restore -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB -c $BACKUP_FILE

# Run migrations on staging
psql -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB -f migrations/006_create_knowledge_graph_schema.sql
psql -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB -f migrations/007_add_multi_tenant_isolation.sql
psql -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB -f migrations/008_create_hybrid_search_views.sql

# Verify staging migration success
psql -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB -c "\dt"
# Should show new tables: agent_capabilities_graph, agent_relationships, etc.

# Test RLS policies on staging
psql -h $STAGING_DB_HOST -U $DB_USER -d $STAGING_DB << EOF
SET app.current_tenant = 'default-tenant';
SELECT COUNT(*) FROM agent_profiles; -- Should see all data
EOF
```

**Go/No-Go Decision Point**: If staging migration fails, do not proceed to production. Investigate and fix issues.

---

### Phase 2: Execute Migration 006 - Knowledge Graph Schema (45 minutes)

#### 2.1 Begin Transaction and Execute Migration

```sql
-- Execute migration 006
\i migrations/006_create_knowledge_graph_schema.sql
```

**What This Does:**

- Creates `entity_type` and `relationship_type` ENUMs
- Creates `agent_capabilities_graph` table with 768-dim vector column
- Creates `agent_relationships` table with typed relationships
- Creates `caws_provenance_graph` table with hash chain support
- Creates `entity_chunk_mappings` for provenance tracking
- Creates HNSW indexes on vector columns (longest operation)
- Creates triggers for canonical name normalization
- Creates views for capability/relationship summaries

**Expected Duration**: 30-45 minutes (index creation is slow)

#### 2.2 Validate Migration 006

```sql
-- Verify tables created
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public'
AND table_name IN (
    'agent_capabilities_graph',
    'agent_relationships',
    'caws_provenance_graph',
    'entity_chunk_mappings'
);
-- Expected: 4 rows

-- Verify indexes created
SELECT indexname
FROM pg_indexes
WHERE tablename = 'agent_capabilities_graph'
AND indexname LIKE '%embedding%';
-- Expected: idx_capabilities_embedding (HNSW)

-- Verify triggers created
SELECT trigger_name
FROM information_schema.triggers
WHERE event_object_table = 'agent_capabilities_graph';
-- Expected: set_capability_canonical_name, update_capabilities_last_updated

-- Test canonical name normalization
INSERT INTO agent_capabilities_graph (
    agent_id, capability_name, confidence, extraction_confidence, tenant_id
) VALUES (
    'test-agent-1', '  TypeScript   Coding  ', 0.85, 0.90, 'default-tenant'
) RETURNING canonical_name;
-- Expected: 'typescript coding'

-- Clean up test data
DELETE FROM agent_capabilities_graph WHERE agent_id = 'test-agent-1';
```

**Go/No-Go Decision Point**: If validation fails, rollback migration 006 and investigate.

---

### Phase 3: Execute Migration 007 - Multi-Tenant Isolation (60 minutes)

#### 3.1 Execute Migration

```sql
-- Execute migration 007
\i migrations/007_add_multi_tenant_isolation.sql
```

**What This Does:**

- Creates `isolation_level`, `privacy_level`, `retention_policy` ENUMs
- Creates `tenants` and `tenant_privacy_config` tables
- Adds `tenant_id` columns to existing tables (agent_profiles, performance_events, etc.)
- Creates default tenant and backfills tenant_id = 'default-tenant'
- Makes tenant_id NOT NULL and adds foreign key constraints
- Enables Row Level Security (RLS) on all tenant-scoped tables
- Creates RLS policies for strict, shared, and federated isolation
- Creates `tenant_access_log` table for audit logging

**Expected Duration**: 45-60 minutes (backfilling large tables can be slow)

#### 3.2 Validate Migration 007

```sql
-- Verify tenant columns added
SELECT column_name
FROM information_schema.columns
WHERE table_name = 'agent_profiles'
AND column_name = 'tenant_id';
-- Expected: 1 row

-- Verify default tenant created
SELECT id, name, isolation_level
FROM tenants
WHERE id = 'default-tenant';
-- Expected: 1 row with isolation_level = 'strict'

-- Verify backfill completed
SELECT COUNT(*)
FROM agent_profiles
WHERE tenant_id IS NULL;
-- Expected: 0 (all should be backfilled)

-- Verify RLS enabled
SELECT tablename, rowsecurity
FROM pg_tables
WHERE tablename = 'agent_profiles';
-- Expected: rowsecurity = true

-- Test RLS policy enforcement
SET app.current_tenant = 'default-tenant';
SELECT COUNT(*) FROM agent_profiles;
-- Expected: All records visible

SET app.current_tenant = 'non-existent-tenant';
SELECT COUNT(*) FROM agent_profiles;
-- Expected: 0 records (policy blocks access)

-- Reset session
RESET app.current_tenant;
```

**Go/No-Go Decision Point**: If RLS policies don't work correctly, this is a critical security issue. Rollback immediately.

---

### Phase 4: Execute Migration 008 - Hybrid Search Architecture (30 minutes)

#### 4.1 Execute Migration

```sql
-- Execute migration 008
\i migrations/008_create_hybrid_search_views.sql
```

**What This Does:**

- Creates `hybrid_search_index` view combining all searchable entities
- Creates `hybrid_search_materialized` materialized view with HNSW index
- Creates `traverse_agent_relationships` function for graph traversal
- Creates `find_agent_path` function for shortest path queries
- Creates `hybrid_search` function combining vector + graph search
- Creates `graph_search_sessions` table for performance tracking
- Creates analytics views (slow queries, popular queries, performance by type)
- Creates helper functions (find_similar_capabilities, compute_agent_centrality)

**Expected Duration**: 20-30 minutes

#### 4.2 Validate Migration 008

```sql
-- Verify views created
SELECT table_name, table_type
FROM information_schema.tables
WHERE table_name LIKE '%hybrid_search%';
-- Expected: 2 rows (view + materialized view)

-- Verify materialized view populated
SELECT COUNT(*) FROM hybrid_search_materialized;
-- Expected: Some count (will be low initially, increases as embeddings added)

-- Verify functions created
SELECT routine_name
FROM information_schema.routines
WHERE routine_name IN (
    'hybrid_search',
    'traverse_agent_relationships',
    'find_agent_path',
    'find_similar_capabilities'
);
-- Expected: 4 rows

-- Test graph traversal function (no data yet, but should not error)
SELECT * FROM traverse_agent_relationships('non-existent-agent', 2);
-- Expected: 0 rows (function works, no data yet)

-- Refresh materialized view
REFRESH MATERIALIZED VIEW hybrid_search_materialized;
-- Expected: Success
```

**Go/No-Go Decision Point**: If functions fail to create or execute, rollback and investigate.

---

### Phase 5: Post-Migration Validation (30 minutes)

#### 5.1 Data Integrity Checks

```sql
-- Verify no data loss in existing tables
SELECT
    'agent_profiles' as table_name,
    COUNT(*) as current_count,
    (SELECT COUNT(*) FROM pre_migration_counts WHERE table_name = 'agent_profiles') as expected_count
FROM agent_profiles;
-- Expected: current_count = expected_count

-- Verify all agents have tenant_id
SELECT COUNT(*)
FROM agent_profiles
WHERE tenant_id IS NULL;
-- Expected: 0

-- Verify foreign key constraints
SELECT
    COUNT(*) as orphaned_capabilities
FROM agent_capabilities_graph acg
LEFT JOIN agent_profiles ap ON acg.agent_id = ap.id
WHERE ap.id IS NULL;
-- Expected: 0 (no orphaned records)
```

#### 5.2 Performance Benchmarks

```sql
-- Benchmark vector search (requires embeddings)
-- Note: Will be fast with HNSW index
EXPLAIN ANALYZE
SELECT * FROM agent_capabilities_graph
WHERE embedding IS NOT NULL
ORDER BY embedding <=> '[0.1, 0.2, ...]'::vector(768)
LIMIT 10;
-- Expected: Uses idx_capabilities_embedding, execution time < 100ms

-- Benchmark graph traversal
EXPLAIN ANALYZE
SELECT * FROM traverse_agent_relationships('any-agent-id', 2);
-- Expected: execution time < 200ms for 2 hops

-- Benchmark RLS overhead
SET app.current_tenant = 'default-tenant';
EXPLAIN ANALYZE
SELECT * FROM agent_profiles WHERE id = 'any-agent-id';
-- Expected: overhead < 10ms compared to non-RLS query
```

#### 5.3 Application Integration Test

```bash
# Start application in test mode
NODE_ENV=test npm start

# Run integration test suite
npm run test:integration:database

# Expected: All tests pass
```

---

### Phase 6: Enable Application Access (15 minutes)

#### 6.1 Update Application Configuration

```typescript
// Update database client configuration
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  max: 50, // Increased for multi-tenant load
  ssl: { rejectUnauthorized: false },
});

// Add tenant context helper
export async function withTenantContext<T>(
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

#### 6.2 Restart Application

```bash
# Re-enable health checks
curl http://localhost:3000/health
# Expected: 200 OK

# Start application services
npm start

# Monitor application logs for database connection errors
tail -f logs/application.log
```

#### 6.3 Smoke Tests

```bash
# Test agent profile retrieval
curl http://localhost:3000/api/agents/agent-123 \
  -H "X-Tenant-ID: default-tenant"
# Expected: 200 OK with agent data

# Test capability search (once embeddings generated)
curl http://localhost:3000/api/search/capabilities \
  -H "X-Tenant-ID: default-tenant" \
  -d '{"query": "TypeScript coding"}'
# Expected: 200 OK with search results
```

---

## Rollback Procedures

### Scenario 1: Migration 006 Fails

```sql
-- Rollback migration 006
BEGIN;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS entity_chunk_mappings CASCADE;
DROP TABLE IF EXISTS caws_provenance_graph CASCADE;
DROP TABLE IF EXISTS agent_relationships CASCADE;
DROP TABLE IF EXISTS agent_capabilities_graph CASCADE;

-- Drop types
DROP TYPE IF EXISTS relationship_type;
DROP TYPE IF EXISTS entity_type;

-- Drop functions
DROP FUNCTION IF EXISTS normalize_entity_name CASCADE;
DROP FUNCTION IF EXISTS set_canonical_name CASCADE;
DROP FUNCTION IF EXISTS prevent_duplicate_relationships CASCADE;
DROP FUNCTION IF EXISTS compute_provenance_hash CASCADE;

-- Drop views
DROP VIEW IF EXISTS agent_capability_summary CASCADE;
DROP VIEW IF EXISTS agent_relationship_summary CASCADE;
DROP VIEW IF EXISTS agent_connectivity CASCADE;
DROP VIEW IF EXISTS caws_provenance_chains CASCADE;

COMMIT;

-- Restore from backup if necessary
-- pg_restore -h $DB_HOST -U $DB_USER -d $DB_NAME -c $BACKUP_FILE
```

### Scenario 2: Migration 007 Fails (RLS Issues)

```sql
-- Rollback migration 007
BEGIN;

-- Disable RLS on all tables
ALTER TABLE agent_profiles DISABLE ROW LEVEL SECURITY;
ALTER TABLE performance_events DISABLE ROW LEVEL SECURITY;
ALTER TABLE benchmark_datasets DISABLE ROW LEVEL SECURITY;
ALTER TABLE agent_capabilities_graph DISABLE ROW LEVEL SECURITY;
ALTER TABLE performance_anomalies DISABLE ROW LEVEL SECURITY;
ALTER TABLE rl_training_batches DISABLE ROW LEVEL SECURITY;

-- Drop RLS policies
DROP POLICY IF EXISTS tenant_strict_isolation ON agent_profiles;
DROP POLICY IF EXISTS tenant_shared_access ON agent_profiles;
DROP POLICY IF EXISTS tenant_federated_access ON performance_events;
DROP POLICY IF EXISTS tenant_benchmark_isolation ON benchmark_datasets;
DROP POLICY IF EXISTS tenant_capability_isolation ON agent_capabilities_graph;
DROP POLICY IF EXISTS tenant_anomaly_isolation ON performance_anomalies;
DROP POLICY IF EXISTS tenant_rl_batch_isolation ON rl_training_batches;

-- Remove tenant_id columns (dangerous - data loss risk)
-- Only do if migration completely failed
ALTER TABLE agent_profiles DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE performance_events DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE benchmark_datasets DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE agent_capabilities_graph DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE performance_anomalies DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE rl_training_batches DROP COLUMN IF EXISTS tenant_id;

-- Drop tenant tables
DROP TABLE IF EXISTS tenant_access_log CASCADE;
DROP TABLE IF EXISTS tenant_privacy_config CASCADE;
DROP TABLE IF EXISTS tenants CASCADE;

-- Drop types
DROP TYPE IF EXISTS retention_policy;
DROP TYPE IF EXISTS privacy_level;
DROP TYPE IF EXISTS isolation_level;

COMMIT;
```

### Scenario 3: Migration 008 Fails

```sql
-- Rollback migration 008
BEGIN;

-- Drop search session table
DROP TABLE IF EXISTS graph_search_sessions CASCADE;

-- Drop materialized view and views
DROP MATERIALIZED VIEW IF EXISTS hybrid_search_materialized CASCADE;
DROP VIEW IF EXISTS hybrid_search_index CASCADE;
DROP VIEW IF EXISTS slow_search_queries CASCADE;
DROP VIEW IF EXISTS popular_search_queries CASCADE;
DROP VIEW IF EXISTS search_performance_by_type CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS refresh_hybrid_search_index CASCADE;
DROP FUNCTION IF EXISTS traverse_agent_relationships CASCADE;
DROP FUNCTION IF EXISTS find_agent_path CASCADE;
DROP FUNCTION IF EXISTS hybrid_search CASCADE;
DROP FUNCTION IF EXISTS find_similar_capabilities CASCADE;
DROP FUNCTION IF EXISTS find_similar_caws_verdicts CASCADE;
DROP FUNCTION IF EXISTS compute_agent_centrality CASCADE;
DROP FUNCTION IF EXISTS log_search_session CASCADE;

COMMIT;
```

### Scenario 4: Complete Rollback from Backup

```bash
# If all else fails, restore from backup
# WARNING: This will lose all data changes since backup

# Stop application
systemctl stop v2-app

# Drop database
psql -h $DB_HOST -U $DB_USER -d postgres -c "DROP DATABASE $DB_NAME;"

# Create fresh database
psql -h $DB_HOST -U $DB_USER -d postgres -c "CREATE DATABASE $DB_NAME;"

# Restore from backup
pg_restore -h $DB_HOST -U $DB_USER -d $DB_NAME -c $BACKUP_FILE

# Verify restoration
psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "SELECT COUNT(*) FROM agent_profiles;"

# Restart application
systemctl start v2-app
```

---

## Post-Migration Tasks

### Immediate (Within 24 hours)

- [ ] **Monitor Performance**: Track P95 latencies for vector search, graph traversal, and RLS queries
- [ ] **Monitor Errors**: Check application logs for database errors
- [ ] **Verify Tenant Isolation**: Audit logs for cross-tenant access attempts
- [ ] **Generate Embeddings**: Begin backfilling embeddings for existing capabilities
- [ ] **Refresh Materialized View**: Run `REFRESH MATERIALIZED VIEW hybrid_search_materialized;`
- [ ] **Update Documentation**: Mark migration as complete in changelog

### Short-Term (Within 1 week)

- [ ] **Performance Tuning**: Adjust HNSW index parameters if search is slow
- [ ] **Connection Pool Tuning**: Adjust pool size based on actual load
- [ ] **RLS Policy Optimization**: Profile RLS overhead, optimize if needed
- [ ] **Index Maintenance**: Run `VACUUM ANALYZE` on new tables
- [ ] **Backup Schedule**: Ensure automated backups include new tables

### Long-Term (Within 1 month)

- [ ] **Tenant Migration**: Move existing projects into proper tenant isolation
- [ ] **Federated Learning**: Enable cross-tenant learning with privacy config
- [ ] **Graph Analytics**: Implement agent centrality scoring
- [ ] **Search Analytics**: Analyze search performance, optimize popular queries
- [ ] **Documentation**: Complete query pattern examples, schema ER diagrams

---

## Success Criteria

✅ **Migration Successful When:**

- All three migrations executed without errors
- All validation queries pass
- No data loss detected (row counts match pre-migration)
- RLS policies enforce tenant isolation
- Vector search performs under 100ms P95
- Graph traversal performs under 200ms P95
- Application starts and connects successfully
- Integration tests pass
- No rollback required

❌ **Migration Failed If:**

- Any migration transaction fails
- Data integrity checks fail
- RLS policies don't block cross-tenant access
- Performance benchmarks exceed SLAs
- Application cannot connect or query database
- Rollback required

---

## Monitoring Dashboard

Post-migration, monitor these metrics:

| Metric                       | Target | Alert Threshold |
| ---------------------------- | ------ | --------------- |
| Vector Search P95            | <100ms | >150ms          |
| Graph Traversal P95          | <200ms | >300ms          |
| RLS Overhead                 | <10ms  | >20ms           |
| Connection Pool Utilization  | <80%   | >90%            |
| Failed Queries               | <0.1%  | >1%             |
| Cross-Tenant Access Attempts | 0      | >0              |
| Disk Space Free              | >20GB  | <10GB           |

---

## Contact Information

**Migration Lead**: [Name]  
**Database Administrator**: [Name]  
**Incident Channel**: #v2-migration-incident  
**Escalation Path**: [On-call rotation]

---

## Appendix A: Estimated Timeline

| Phase                     | Duration             | Start | End   |
| ------------------------- | -------------------- | ----- | ----- |
| Pre-Migration Validation  | 15 min               | 00:00 | 00:15 |
| Migration 006 Execution   | 45 min               | 00:15 | 01:00 |
| Migration 007 Execution   | 60 min               | 01:00 | 02:00 |
| Migration 008 Execution   | 30 min               | 02:00 | 02:30 |
| Post-Migration Validation | 30 min               | 02:30 | 03:00 |
| Application Integration   | 15 min               | 03:00 | 03:15 |
| **Total**                 | **195 min (3h 15m)** |       |       |

**Buffer for Issues**: +2-3 hours  
**Maximum Duration**: 6 hours (maintenance window)

---

## Appendix B: Staging Environment Testing Results

```
[To be filled in after staging migration]

- Staging migration completed: [DATE]
- Issues encountered: [NONE / LIST]
- Performance benchmarks: [RESULTS]
- Rollback tested: [SUCCESS / FAIL]
- Approved for production: [YES / NO]
```

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-12  
**Next Review**: Post-migration retrospective

