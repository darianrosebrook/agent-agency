# Database Troubleshooting Guide

This guide provides comprehensive troubleshooting for PostgreSQL database issues in Agent Agency V3.

## Table of Contents

- [Connection Issues](#connection-issues)
- [Migration Problems](#migration-problems)
- [Performance Issues](#performance-issues)
- [Data Consistency](#data-consistency)
- [Monitoring & Diagnostics](#monitoring--diagnostics)
- [Backup & Recovery](#backup--recovery)

## Connection Issues

### Connection Refused

**Symptoms:**
- `Connection refused` or `no route to host` errors
- Database client initialization failures

**Solutions:**

1. **Check PostgreSQL service status:**
   ```bash
   # Docker container
   docker ps | grep postgres

   # System service
   systemctl status postgresql
   ```

2. **Verify connection parameters:**
   ```bash
   # Test connection
   psql -h localhost -p 5432 -U agent_agency -d agent_agency

   # Check environment variables
   echo $DATABASE_URL
   ```

3. **Firewall configuration:**
   ```bash
   # Check if port is open
   nc -zv localhost 5432

   # Docker port mapping
   docker port agent-agency-db
   ```

### Authentication Failed

**Symptoms:**
- `authentication failed` or `password authentication failed`

**Solutions:**

1. **Verify credentials:**
   ```sql
   -- Connect as superuser and check user
   \c postgres
   SELECT * FROM pg_roles WHERE rolname = 'agent_agency';
   ```

2. **Update password:**
   ```sql
   ALTER USER agent_agency PASSWORD 'new_secure_password';
   ```

3. **Check pg_hba.conf:**
   ```bash
   # Find configuration file
   psql -U postgres -c "SHOW hba_file;"

   # Check authentication methods
   cat /var/lib/postgresql/data/pg_hba.conf
   ```

### Connection Pool Exhausted

**Symptoms:**
- `pool exhausted` or `too many connections` errors
- Slow response times under load

**Solutions:**

1. **Monitor active connections:**
   ```sql
   SELECT count(*) as active_connections
   FROM pg_stat_activity
   WHERE state = 'active';
   ```

2. **Adjust pool configuration:**
   ```bash
   # Environment variables
   DATABASE_MAX_CONNECTIONS=30
   DATABASE_MIN_IDLE_CONNECTIONS=5
   ```

3. **Check for connection leaks:**
   ```sql
   -- Long-running connections
   SELECT pid, usename, client_addr, backend_start, state, query
   FROM pg_stat_activity
   WHERE state = 'idle in transaction'
   ORDER BY backend_start;
   ```

## Migration Problems

### Migration Lock Issues

**Symptoms:**
- `migration table locked` errors
- Stalled migration processes

**Solutions:**

1. **Check for running migrations:**
   ```sql
   SELECT * FROM agent_agency_migrations
   WHERE status = 'running';
   ```

2. **Force unlock (CAUTION):**
   ```sql
   -- Only if you're sure no migration is running
   UPDATE agent_agency_migrations
   SET status = 'failed', completed_at = NOW()
   WHERE status = 'running';
   ```

3. **Manual migration:**
   ```bash
   # Run specific migration
   cargo run --bin migration_runner -- --migration 010
   ```

### Schema Conflicts

**Symptoms:**
- `relation already exists` or `column already exists` errors

**Solutions:**

1. **Check current schema:**
   ```sql
   \dt  -- List all tables
   \d+ caws_validations  -- Inspect specific table
   ```

2. **Skip conflicting migration:**
   ```sql
   -- Mark migration as complete without running
   INSERT INTO agent_agency_migrations (migration_id, status, completed_at)
   VALUES ('010', 'completed', NOW());
   ```

3. **Drop conflicting objects (CAUTION):**
   ```sql
   -- Only in development/testing
   DROP TABLE IF EXISTS conflicting_table CASCADE;
   ```

### Extension Missing

**Symptoms:**
- `extension "pgvector" does not exist` errors

**Solutions:**

1. **Install pgvector extension:**
   ```sql
   CREATE EXTENSION IF NOT EXISTS pgvector;
   CREATE EXTENSION IF NOT EXISTS uuid_ossp;
   ```

2. **Verify extension availability:**
   ```sql
   SELECT * FROM pg_available_extensions WHERE name LIKE '%vector%';
   ```

3. **Docker container setup:**
   ```bash
   # Use pgvector image
   docker run -d --name agent-agency-db pgvector/pgvector:pg15
   ```

## Performance Issues

### Slow Queries

**Symptoms:**
- Query execution time > 100ms (SLA violation)
- High CPU usage on database server

**Solutions:**

1. **Identify slow queries:**
   ```sql
   SELECT query, calls, total_time, mean_time, rows
   FROM pg_stat_statements
   ORDER BY total_time DESC
   LIMIT 10;
   ```

2. **Check query plans:**
   ```sql
   EXPLAIN ANALYZE SELECT * FROM caws_validations WHERE task_id = 'some-uuid';
   ```

3. **Add missing indexes:**
   ```sql
   CREATE INDEX CONCURRENTLY idx_caws_task_created ON caws_validations(task_id, validated_at);
   ```

4. **Optimize table statistics:**
   ```sql
   ANALYZE VERBOSE caws_validations;
   ```

### High Memory Usage

**Symptoms:**
- Database memory usage > 80%
- Out of memory errors

**Solutions:**

1. **Check memory configuration:**
   ```sql
   SHOW shared_buffers;
   SHOW work_mem;
   SHOW maintenance_work_mem;
   ```

2. **Monitor memory usage:**
   ```sql
   SELECT name, setting, unit
   FROM pg_settings
   WHERE name LIKE '%mem%' OR name LIKE '%buffer%';
   ```

3. **Adjust configuration:**
   ```ini
   # postgresql.conf
   shared_buffers = 256MB
   work_mem = 4MB
   maintenance_work_mem = 64MB
   ```

### Lock Contention

**Symptoms:**
- Transactions waiting for locks
- Deadlock errors

**Solutions:**

1. **Monitor locks:**
   ```sql
   SELECT blocked_locks.pid AS blocked_pid,
          blocking_locks.pid AS blocking_pid,
          blocked_activity.usename AS blocked_user,
          blocking_activity.usename AS blocking_user,
          blocked_activity.query AS blocked_query,
          blocking_activity.query AS blocking_query
   FROM pg_locks blocked_locks
   JOIN pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
   JOIN pg_locks blocking_locks
       ON blocking_locks.locktype = blocked_locks.locktype
       AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
       AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
       AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
       AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
       AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
       AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
       AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
       AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
       AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
       AND blocking_locks.pid != blocked_locks.pid
   JOIN pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
   WHERE NOT blocked_locks.granted;
   ```

2. **Reduce transaction scope:**
   ```rust
   // Use shorter transactions
   let mut tx = client.begin().await?;
   // Do minimal work
   tx.commit().await?;
   ```

3. **Use advisory locks sparingly:**
   ```sql
   -- Avoid long-held advisory locks
   SELECT pg_advisory_lock(12345);
   -- Do work quickly
   SELECT pg_advisory_unlock(12345);
   ```

## Data Consistency

### Foreign Key Violations

**Symptoms:**
- `foreign key constraint violated` errors

**Solutions:**

1. **Check constraint violations:**
   ```sql
   SELECT conname, conrelid::regclass, confrelid::regclass
   FROM pg_constraint
   WHERE contype = 'f';
   ```

2. **Identify orphaned records:**
   ```sql
   -- Find records without valid foreign keys
   SELECT * FROM child_table ct
   LEFT JOIN parent_table pt ON ct.parent_id = pt.id
   WHERE pt.id IS NULL;
   ```

3. **Fix data inconsistencies:**
   ```sql
   -- Remove orphaned records (CAUTION)
   DELETE FROM child_table
   WHERE parent_id NOT IN (SELECT id FROM parent_table);
   ```

### Duplicate Key Violations

**Symptoms:**
- `duplicate key value violates unique constraint` errors

**Solutions:**

1. **Check unique constraints:**
   ```sql
   \d+ table_name  -- Shows constraints
   ```

2. **Find duplicates:**
   ```sql
   SELECT column_name, count(*)
   FROM table_name
   GROUP BY column_name
   HAVING count(*) > 1;
   ```

3. **Resolve duplicates:**
   ```sql
   -- Keep the latest record
   DELETE FROM table_name a USING (
     SELECT MIN(ctid) as ctid, column_name
     FROM table_name
     GROUP BY column_name HAVING COUNT(*) > 1
   ) b
   WHERE a.column_name = b.column_name
   AND a.ctid <> b.ctid;
   ```

### Data Type Mismatches

**Symptoms:**
- `invalid input syntax` or type conversion errors

**Solutions:**

1. **Check column types:**
   ```sql
   \d+ table_name
   ```

2. **Validate data before insertion:**
   ```sql
   -- Check for invalid UUIDs
   SELECT * FROM table_name
   WHERE NOT (id ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$');
   ```

## Monitoring & Diagnostics

### Health Checks

**Automated health monitoring:**

```sql
-- Database connectivity
SELECT 1 as health_check;

-- Table existence
SELECT tablename FROM pg_tables WHERE schemaname = 'public';

-- Extension availability
SELECT * FROM pg_extension WHERE extname IN ('pgvector', 'uuid_ossp');
```

**Application-level health checks:**

```rust
use agent_agency_database::DatabaseClient;

async fn health_check(db_client: &DatabaseClient) -> Result<(), anyhow::Error> {
    // Test basic connectivity
    db_client.execute_query("SELECT 1").await?;

    // Test component-specific tables
    db_client.execute_query("SELECT count(*) FROM caws_validations").await?;
    db_client.execute_query("SELECT count(*) FROM source_integrity_records").await?;

    Ok(())
}
```

### Performance Metrics

**Key metrics to monitor:**

```sql
-- Query performance
SELECT
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch
FROM pg_stat_user_tables
ORDER BY seq_tup_read DESC;

-- Index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Cache hit ratio
SELECT
    sum(blks_hit) * 100 / (sum(blks_hit) + sum(blks_read)) as cache_hit_ratio
FROM pg_stat_database;
```

### Logging Configuration

**Enable detailed query logging:**

```ini
# postgresql.conf
log_statement = 'all'
log_duration = on
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_min_duration_statement = 100  # Log queries > 100ms
```

## Backup & Recovery

### Automated Backups

**Daily backup script:**

```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="/var/backups/agent-agency"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/agent_agency_$TIMESTAMP.sql"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -U agent_agency -h localhost -d agent_agency > $BACKUP_FILE

# Compress backup
gzip $BACKUP_FILE

# Clean old backups (keep last 7 days)
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_FILE.gz"
```

### Point-in-Time Recovery

**WAL archiving setup:**

```ini
# postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/archive/%f'
```

**Recovery configuration:**

```ini
# recovery.conf
restore_command = 'cp /var/lib/postgresql/archive/%f %p'
recovery_target_time = '2024-01-15 10:00:00'
```

### Disaster Recovery Testing

**Regular recovery testing:**

```bash
#!/bin/bash
# test-recovery.sh

# Stop current instance
docker stop agent-agency-db

# Start recovery instance
docker run -d \
  --name agent-agency-recovery \
  -v /path/to/backups:/backups \
  -v /path/to/archive:/var/lib/postgresql/archive \
  pgvector/pgvector:pg15

# Test recovery
docker exec agent-agency-recovery pg_restore -U agent_agency -d agent_agency /backups/latest.sql.gz

# Validate data integrity
docker exec agent-agency-recovery psql -U agent_agency -d agent_agency -c "SELECT count(*) FROM caws_validations;"
```

## Component-Specific Issues

### CAWS Checker Issues

**Symptoms:**
- Validation results not persisting
- Compliance history queries failing

**Debugging:**
```sql
-- Check table structure
\d+ caws_validations

-- Verify recent validations
SELECT * FROM caws_validations ORDER BY validated_at DESC LIMIT 5;

-- Check for invalid data
SELECT * FROM caws_validations WHERE violations IS NULL OR suggestions IS NULL;
```

### Source Integrity Issues

**Symptoms:**
- Integrity verification failures
- Alert generation problems

**Debugging:**
```sql
-- Check record counts
SELECT
    integrity_status,
    count(*) as record_count
FROM source_integrity_records
GROUP BY integrity_status;

-- Verify verification history
SELECT
    sir.source_id,
    count(siv.*) as verification_count
FROM source_integrity_records sir
LEFT JOIN source_integrity_verifications siv ON sir.id = siv.source_integrity_id
GROUP BY sir.id, sir.source_id;
```

### Council Learning Issues

**Symptoms:**
- Historical data queries slow or failing
- Resource prediction inaccuracies

**Debugging:**
```sql
-- Check data volume
SELECT count(*) FROM task_resource_history;

-- Verify index usage
SELECT * FROM pg_stat_user_indexes WHERE tablename = 'task_resource_history';

-- Check data distribution
SELECT
    task_type,
    count(*) as record_count,
    avg(execution_time_ms) as avg_execution_time,
    avg(cpu_usage_percent) as avg_cpu_usage
FROM task_resource_history
GROUP BY task_type;
```

### Claim Extraction Issues

**Symptoms:**
- Knowledge base queries failing
- Entity disambiguation problems

**Debugging:**
```sql
-- Check knowledge base tables
SELECT count(*) FROM external_knowledge_entities;

-- Verify embedding dimensions
SELECT
    entity_id,
    array_length(embedding_vector, 1) as embedding_dimension
FROM knowledge_embeddings
LIMIT 5;

-- Check usage statistics
SELECT
    source,
    count(*) as usage_count
FROM knowledge_usage_stats
GROUP BY source;
```

### Analytics Dashboard Issues

**Symptoms:**
- Cache misses high
- Dashboard performance slow

**Debugging:**
```sql
-- Check cache statistics
SELECT
    count(*) as total_cache_entries,
    count(*) FILTER (WHERE expires_at > NOW()) as valid_entries,
    avg(access_count) as avg_access_count
FROM analytics_cache;

-- Verify eviction is working
SELECT * FROM analytics_cache
ORDER BY last_accessed_at ASC
LIMIT 5;

-- Check cache hit patterns
SELECT
    cache_key,
    access_count,
    last_accessed_at
FROM analytics_cache
ORDER BY access_count DESC
LIMIT 10;
```

## Emergency Procedures

### Complete Database Reset

**⚠️ DESTRUCTIVE - Use only in development/testing**

```bash
#!/bin/bash
# reset-database.sh

# Stop all connections
docker exec agent-agency-db psql -U postgres -c "
  SELECT pg_terminate_backend(pid)
  FROM pg_stat_activity
  WHERE datname = 'agent_agency' AND pid <> pg_backend_pid();
"

# Drop and recreate database
docker exec agent-agency-db psql -U postgres -c "
  DROP DATABASE IF EXISTS agent_agency;
  CREATE DATABASE agent_agency OWNER agent_agency;
"

# Re-enable extensions
docker exec agent-agency-db psql -U agent_agency -d agent_agency -c "
  CREATE EXTENSION IF NOT EXISTS pgvector;
  CREATE EXTENSION IF NOT EXISTS uuid_ossp;
"

# Run migrations
cd /path/to/agent-agency/iterations/v3
cargo run --bin migration_runner

echo "Database reset complete"
```

### Application Circuit Breaker

**When database is unresponsive:**

```rust
use std::time::Duration;

// Implement circuit breaker pattern
#[derive(Debug)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct DatabaseCircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<std::time::Instant>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl DatabaseCircuitBreaker {
    async fn execute_with_circuit_breaker<F, T>(&mut self, operation: F) -> Result<T, anyhow::Error>
    where
        F: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                } else {
                    return Err(anyhow::anyhow!("Circuit breaker is OPEN"));
                }
            }
            _ => {}
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            last_failure.elapsed() > self.recovery_timeout
        } else {
            false
        }
    }

    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }
}
```

## Support Resources

- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [pgvector Extension](https://github.com/pgvector/pgvector)
- [Connection Pooling Best Practices](https://github.com/launchbadge/sqlx/blob/main/README.md)
- [Database Performance Tuning](https://www.postgresql.org/docs/current/performance-tips.html)

For additional support, check the project issue tracker or contact the development team.
