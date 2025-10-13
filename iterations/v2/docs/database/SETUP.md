# Database Setup Guide

**Version**: 1.0.0  
**Last Updated**: October 13, 2025  
**Author**: @darianrosebrook  
**Purpose**: Complete guide for setting up PostgreSQL database for Agent Agency V2

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Installation](#installation)
4. [Database Setup](#database-setup)
5. [Configuration](#configuration)
6. [Running Migrations](#running-migrations)
7. [Verification](#verification)
8. [Troubleshooting](#troubleshooting)
9. [Backup and Restore](#backup-and-restore)
10. [Advanced Topics](#advanced-topics)

---

## Overview

Agent Agency V2 uses **PostgreSQL** as its primary database for:
- Agent registry and capabilities
- Task queue and assignments
- Knowledge graphs and queries
- Performance tracking metrics
- Verification and validation results
- Provenance and audit trails

**Database Requirements**:
- PostgreSQL 12.0 or higher
- 500MB minimum disk space (1GB recommended)
- UTF-8 encoding
- C collation

---

## Prerequisites

### System Requirements

| Component | Minimum | Recommended |
| --- | --- | --- |
| **PostgreSQL Version** | 12.0 | 14.0+ |
| **RAM** | 512MB | 2GB+ |
| **Disk Space** | 500MB | 1GB+ |
| **CPU** | 1 core | 2+ cores |

### Required Tools

- **psql**: PostgreSQL command-line client
- **createuser**: PostgreSQL user creation utility
- **createdb**: PostgreSQL database creation utility
- **Node.js**: 16.0+ (for running migrations)
- **npm**: 8.0+ (for dependency management)

---

## Installation

### macOS

#### Option 1: Homebrew (Recommended)

```bash
# Install PostgreSQL
brew install postgresql@14

# Start PostgreSQL service
brew services start postgresql@14

# Add to PATH (add to ~/.zshrc or ~/.bashrc)
export PATH="/usr/local/opt/postgresql@14/bin:$PATH"

# Reload shell configuration
source ~/.zshrc
```

#### Option 2: Postgres.app

1. Download from [https://postgresapp.com/](https://postgresapp.com/)
2. Drag to Applications folder
3. Open Postgres.app
4. Click "Initialize" to create a PostgreSQL cluster
5. Add to PATH: `sudo mkdir -p /etc/paths.d && echo /Applications/Postgres.app/Contents/Versions/latest/bin | sudo tee /etc/paths.d/postgresapp`

### Linux (Ubuntu/Debian)

```bash
# Update package list
sudo apt update

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Check status
sudo systemctl status postgresql
```

### Linux (CentOS/RHEL/Fedora)

```bash
# Install PostgreSQL
sudo dnf install postgresql-server postgresql-contrib

# Initialize database cluster
sudo postgresql-setup --initdb

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### Windows

1. Download installer from [https://www.postgresql.org/download/windows/](https://www.postgresql.org/download/windows/)
2. Run installer and follow setup wizard
3. Note the password you set for the `postgres` user
4. Add PostgreSQL bin directory to PATH: `C:\Program Files\PostgreSQL\14\bin`

### Docker (Cross-Platform)

```bash
# Pull PostgreSQL image
docker pull postgres:14

# Run PostgreSQL container
docker run --name agent-agency-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  -v agent-agency-data:/var/lib/postgresql/data \
  -d postgres:14

# Verify container is running
docker ps | grep agent-agency-postgres
```

---

## Database Setup

### Step 1: Verify PostgreSQL Installation

```bash
# Check PostgreSQL version
psql --version
# Expected output: psql (PostgreSQL) 14.x

# Check if PostgreSQL is running
pg_isready
# Expected output: /tmp:5432 - accepting connections
```

### Step 2: Create PostgreSQL Superuser

The application uses a `postgres` superuser role for test database operations.

```bash
# Check if postgres role exists
psql -U $USER -d postgres -c "SELECT 1 FROM pg_roles WHERE rolname='postgres'" | grep -q "1 row"

# If role doesn't exist, create it
createuser -s postgres
```

**What this does**:
- Creates a superuser named `postgres`
- `-s` flag grants superuser privileges
- No password is set (localhost trust authentication)

### Step 3: Create Test Database

```bash
# Create the test database
createdb agent_agency_v2_test -O postgres

# Verify database was created
psql -U postgres -d postgres -c "\l" | grep agent_agency_v2_test
```

**Expected output**:
```
agent_agency_v2_test | postgres | UTF8 | C | C |
```

### Step 4: Verify Database Access

```bash
# Connect to test database
psql -U postgres -d agent_agency_v2_test

# Inside psql, run:
SELECT current_database();
# Expected: agent_agency_v2_test

# Check available extensions
\dx

# Exit psql
\q
```

---

## Configuration

### Environment Variables

Create a `.env.test` file in the project root:

```bash
# Create .env.test
cat > .env.test << 'EOF'
# Test Environment Configuration

# Database Configuration
DATABASE_URL=postgresql://postgres@localhost:5432/agent_agency_v2_test
PGHOST=localhost
PGPORT=5432
PGDATABASE=agent_agency_v2_test
PGUSER=postgres
PGPASSWORD=

# Connection Pool Configuration
DB_POOL_MIN=2
DB_POOL_MAX=10
DB_POOL_IDLE_TIMEOUT=30000
DB_POOL_CONNECTION_TIMEOUT=5000

# Test Configuration
NODE_ENV=test
LOG_LEVEL=error

# Feature Flags
ENABLE_METRICS=false
ENABLE_TRACING=false
EOF
```

### Configuration Options

| Variable                      | Default   | Description                       |
| ----------------------------- | --------- | --------------------------------- |
| `DATABASE_URL`                | (required) | Full PostgreSQL connection string |
| `PGHOST`                      | localhost | Database server hostname          |
| `PGPORT`                      | 5432      | Database server port              |
| `PGDATABASE`                  | (required) | Database name                     |
| `PGUSER`                      | postgres  | Database user                     |
| `PGPASSWORD`                  | (empty)   | Database password (if required)   |
| `DB_POOL_MIN`                 | 2         | Minimum connections in pool       |
| `DB_POOL_MAX`                 | 10        | Maximum connections in pool       |
| `DB_POOL_IDLE_TIMEOUT`        | 30000     | Idle connection timeout (ms)      |
| `DB_POOL_CONNECTION_TIMEOUT`  | 5000      | Connection attempt timeout (ms)   |

### Connection Pooling

The application uses `pg-pool` for connection pooling:

```typescript
// Connection pool is initialized in tests/setup.ts
const pool = new Pool({
  host: process.env.PGHOST,
  port: parseInt(process.env.PGPORT || '5432'),
  database: process.env.PGDATABASE,
  user: process.env.PGUSER,
  password: process.env.PGPASSWORD,
  min: parseInt(process.env.DB_POOL_MIN || '2'),
  max: parseInt(process.env.DB_POOL_MAX || '10'),
  idleTimeoutMillis: parseInt(process.env.DB_POOL_IDLE_TIMEOUT || '30000'),
  connectionTimeoutMillis: parseInt(process.env.DB_POOL_CONNECTION_TIMEOUT || '5000'),
});
```

---

## Running Migrations

### Migration Overview

Migrations are SQL files in the `migrations/` directory that create and modify database schema.

**Available Migrations** (as of October 2025):
1. `001_create_agent_registry_tables.sql` (9.5KB) - Agent profiles and capabilities
2. `002_create_task_queue_tables.sql` (12KB) - Task queue and assignments
3. `003_create_knowledge_tables.sql` (14KB) - Knowledge queries and results
4. `004_create_performance_tracking_tables.sql` (20KB) - Performance metrics
5. `004_create_verification_tables.sql` (18KB) - Verification results
6. `004_create_web_tables.sql` (18KB) - Web navigation data
7. `005_task_research_provenance.sql` (2.2KB) - Research provenance tracking
8. `006_create_knowledge_graph_schema.sql` (18KB) - Knowledge graph structures
9. `006_create_learning_tables.sql` (9.3KB) - Learning and iteration data
10. `007_add_multi_tenant_isolation.sql` (18KB) - Multi-tenant support
11. `008_create_hybrid_search_views.sql` (20KB) - Hybrid search capabilities

### Running All Migrations

```bash
# Navigate to project root
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v2

# Run migrations in order
for migration in \
  migrations/001_create_agent_registry_tables.sql \
  migrations/002_create_task_queue_tables.sql \
  migrations/003_create_knowledge_tables.sql \
  migrations/005_task_research_provenance.sql \
  migrations/006_create_knowledge_graph_schema.sql \
  migrations/007_add_multi_tenant_isolation.sql; do
  echo "Running $migration..."
  psql -U postgres -d agent_agency_v2_test -f "$migration"
done
```

**Note**: Migration 008 has dependencies that may not be met. It's optional for basic functionality.

### Running Individual Migrations

```bash
# Run a specific migration
psql -U postgres -d agent_agency_v2_test -f migrations/001_create_agent_registry_tables.sql

# Check if successful
psql -U postgres -d agent_agency_v2_test -c "\dt"
```

### Migration Rollback

Most migrations don't include rollback scripts. To rollback:

```bash
# Drop the test database
dropdb agent_agency_v2_test

# Recreate it
createdb agent_agency_v2_test -O postgres

# Re-run migrations up to the desired point
psql -U postgres -d agent_agency_v2_test -f migrations/001_create_agent_registry_tables.sql
# ... (run subsequent migrations as needed)
```

---

## Verification

### Step 1: Check Database Health

```bash
# Test database connection
psql -U postgres -d agent_agency_v2_test -c "SELECT 1 AS health"
# Expected: health = 1
```

### Step 2: Verify Tables

```bash
# List all tables
psql -U postgres -d agent_agency_v2_test -c "\dt"

# Expected output (after running migrations):
# agent_capabilities
# agent_profiles
# knowledge_queries
# task_assignments
# task_queue
# task_research_provenance
# ... (and others depending on migrations run)
```

### Step 3: Check Table Schemas

```bash
# Describe a specific table
psql -U postgres -d agent_agency_v2_test -c "\d agent_profiles"

# Check table row counts
psql -U postgres -d agent_agency_v2_test -c "
  SELECT schemaname, tablename, n_tup_ins AS rows_inserted
  FROM pg_stat_user_tables
  ORDER BY tablename;
"
```

### Step 4: Test with Application

```bash
# Run unit tests (should connect to database)
npm run test:unit -- --testPathPattern="agent-registry"

# Run integration tests
npm run test:integration

# Expected: Tests connect to database successfully
```

### Step 5: Connection Pool Health Check

```bash
# Check active connections
psql -U postgres -d postgres -c "
  SELECT 
    datname, 
    COUNT(*) as connections
  FROM pg_stat_activity
  WHERE datname = 'agent_agency_v2_test'
  GROUP BY datname;
"
```

---

## Troubleshooting

### Issue 1: "role 'postgres' does not exist"

**Error**:
```
error: role "postgres" does not exist
```

**Solution**:
```bash
# Create the postgres role
createuser -s postgres

# Verify role was created
psql -U $USER -d postgres -c "\du" | grep postgres
```

---

### Issue 2: "database 'agent_agency_v2_test' does not exist"

**Error**:
```
FATAL: database "agent_agency_v2_test" does not exist
```

**Solution**:
```bash
# Create the database
createdb agent_agency_v2_test -O postgres

# Verify creation
psql -U postgres -l | grep agent_agency_v2_test
```

---

### Issue 3: "connection refused" or "could not connect"

**Error**:
```
could not connect to server: Connection refused
Is the server running on host "localhost" (::1) and accepting
TCP/IP connections on port 5432?
```

**Solutions**:

**Check if PostgreSQL is running**:
```bash
pg_isready
# If not ready, start it:

# macOS (Homebrew)
brew services start postgresql@14

# Linux (systemd)
sudo systemctl start postgresql

# Docker
docker start agent-agency-postgres
```

**Check PostgreSQL port**:
```bash
# Verify PostgreSQL is listening on port 5432
lsof -i :5432
# or
netstat -an | grep 5432
```

**Check PostgreSQL logs**:
```bash
# macOS (Homebrew)
tail -f /usr/local/var/log/postgresql@14.log

# Linux
sudo tail -f /var/log/postgresql/postgresql-14-main.log

# Docker
docker logs agent-agency-postgres
```

---

### Issue 4: "permission denied" errors

**Error**:
```
ERROR: permission denied for table agent_profiles
```

**Solution**:
```bash
# Grant permissions to postgres user
psql -U postgres -d agent_agency_v2_test -c "
  GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
  GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
"
```

---

### Issue 5: Migration fails with dependency errors

**Error**:
```
ERROR: relation "agent_capabilities_graph" does not exist
```

**Solution**:

This typically happens with migration 008 which has dependencies. Either:

**Option A**: Skip migration 008 (not critical for basic functionality)

**Option B**: Ensure prerequisite tables exist by running earlier migrations first

---

### Issue 6: Too many open connections

**Error**:
```
FATAL: sorry, too many clients already
```

**Solution**:

```bash
# Check current connections
psql -U postgres -d postgres -c "
  SELECT COUNT(*) FROM pg_stat_activity;
"

# Check max connections setting
psql -U postgres -d postgres -c "SHOW max_connections;"

# Kill idle connections
psql -U postgres -d postgres -c "
  SELECT pg_terminate_backend(pid)
  FROM pg_stat_activity
  WHERE datname = 'agent_agency_v2_test'
    AND state = 'idle'
    AND state_change < NOW() - INTERVAL '5 minutes';
"

# Or restart PostgreSQL
brew services restart postgresql@14  # macOS
sudo systemctl restart postgresql    # Linux
```

---

## Backup and Restore

### Creating Backups

#### Full Database Backup

```bash
# Backup entire database to file
pg_dump -U postgres agent_agency_v2_test > backup_$(date +%Y%m%d_%H%M%S).sql

# Compressed backup
pg_dump -U postgres agent_agency_v2_test | gzip > backup_$(date +%Y%m%d_%H%M%S).sql.gz
```

#### Schema-Only Backup

```bash
# Backup only schema (no data)
pg_dump -U postgres --schema-only agent_agency_v2_test > schema_backup.sql
```

#### Data-Only Backup

```bash
# Backup only data (no schema)
pg_dump -U postgres --data-only agent_agency_v2_test > data_backup.sql
```

#### Specific Table Backup

```bash
# Backup specific table
pg_dump -U postgres -t agent_profiles agent_agency_v2_test > agent_profiles_backup.sql
```

### Restoring Backups

#### Restore Full Database

```bash
# Drop and recreate database
dropdb agent_agency_v2_test
createdb agent_agency_v2_test -O postgres

# Restore from backup
psql -U postgres agent_agency_v2_test < backup_20251013_150000.sql

# Restore from compressed backup
gunzip -c backup_20251013_150000.sql.gz | psql -U postgres agent_agency_v2_test
```

#### Restore Specific Tables

```bash
# Restore only specific table
psql -U postgres agent_agency_v2_test < agent_profiles_backup.sql
```

### Automated Backup Script

Create `scripts/backup_database.sh`:

```bash
#!/bin/bash
# Automated database backup script

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DATABASE="agent_agency_v2_test"
USER="postgres"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Create backup
echo "Creating backup: ${BACKUP_DIR}/backup_${TIMESTAMP}.sql.gz"
pg_dump -U "$USER" "$DATABASE" | gzip > "${BACKUP_DIR}/backup_${TIMESTAMP}.sql.gz"

# Keep only last 7 days of backups
find "$BACKUP_DIR" -name "backup_*.sql.gz" -mtime +7 -delete

echo "Backup complete!"
```

Make executable and run:
```bash
chmod +x scripts/backup_database.sh
./scripts/backup_database.sh
```

---

## Advanced Topics

### Connection String Formats

**Standard Format**:
```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**Examples**:
```bash
# Local connection, no password
postgresql://postgres@localhost:5432/agent_agency_v2_test

# With password
postgresql://postgres:mypassword@localhost:5432/agent_agency_v2_test

# SSL connection
postgresql://postgres@remote.host:5432/agent_agency_v2_test?sslmode=require

# Unix socket connection
postgresql:///agent_agency_v2_test?host=/var/run/postgresql
```

### Performance Tuning

#### Increase Shared Buffers

```bash
# Edit postgresql.conf
# Location varies by installation:
# - macOS (Homebrew): /usr/local/var/postgresql@14/postgresql.conf
# - Linux: /etc/postgresql/14/main/postgresql.conf

# Add or modify:
shared_buffers = 256MB  # Default is usually 128MB
effective_cache_size = 1GB
work_mem = 16MB
maintenance_work_mem = 128MB

# Restart PostgreSQL
brew services restart postgresql@14  # macOS
sudo systemctl restart postgresql    # Linux
```

#### Enable Query Logging

```bash
# Edit postgresql.conf
log_statement = 'all'
log_duration = on
log_min_duration_statement = 100  # Log queries > 100ms

# Or enable for current session:
psql -U postgres -d agent_agency_v2_test -c "
  SET log_statement = 'all';
  SET log_duration = on;
"
```

### Multi-Database Setup

For running tests in parallel:

```bash
# Create additional test databases
for i in {1..4}; do
  createdb "agent_agency_v2_test_$i" -O postgres
  # Run migrations for each
  psql -U postgres -d "agent_agency_v2_test_$i" -f migrations/001_create_agent_registry_tables.sql
  # ... (run other migrations)
done

# Update test configuration to use different databases per worker
# In jest.config.js or test setup:
const dbName = `agent_agency_v2_test_${process.env.JEST_WORKER_ID || '1'}`;
```

### SSL/TLS Configuration

For production or remote connections:

```bash
# Edit postgresql.conf
ssl = on
ssl_cert_file = '/path/to/server.crt'
ssl_key_file = '/path/to/server.key'

# Update connection string
DATABASE_URL=postgresql://postgres@localhost:5432/agent_agency_v2_test?sslmode=require
```

### Monitoring Queries

```bash
# View active queries
psql -U postgres -d postgres -c "
  SELECT
    pid,
    now() - pg_stat_activity.query_start AS duration,
    query,
    state
  FROM pg_stat_activity
  WHERE state != 'idle'
  ORDER BY duration DESC;
"

# View slow queries (requires pg_stat_statements extension)
psql -U postgres -d agent_agency_v2_test -c "
  CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

  SELECT
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    max_exec_time
  FROM pg_stat_statements
  ORDER BY mean_exec_time DESC
  LIMIT 10;
"
```

---

## Quick Reference

### Common Commands

```bash
# List databases
psql -U postgres -l

# Connect to database
psql -U postgres -d agent_agency_v2_test

# List tables (inside psql)
\dt

# Describe table
\d agent_profiles

# List users/roles
\du

# Show current database
SELECT current_database();

# Show database size
SELECT pg_size_pretty(pg_database_size('agent_agency_v2_test'));

# Exit psql
\q
```

### Test Commands

```bash
# Run all tests with database
npm test

# Run only unit tests
npm run test:unit

# Run only integration tests
npm run test:integration

# Run with coverage
npm run test:coverage
```

### Maintenance Commands

```bash
# Vacuum database (cleanup)
psql -U postgres -d agent_agency_v2_test -c "VACUUM ANALYZE;"

# Reindex database
psql -U postgres -d agent_agency_v2_test -c "REINDEX DATABASE agent_agency_v2_test;"

# Check database statistics
psql -U postgres -d agent_agency_v2_test -c "
  SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del
  FROM pg_stat_user_tables;
"
```

---

## Additional Resources

### Official Documentation

- **PostgreSQL Docs**: [https://www.postgresql.org/docs/](https://www.postgresql.org/docs/)
- **pg-pool Docs**: [https://node-postgres.com/apis/pool](https://node-postgres.com/apis/pool)
- **psql Commands**: [https://www.postgresql.org/docs/current/app-psql.html](https://www.postgresql.org/docs/current/app-psql.html)

### Internal Documentation

- **Priority 2 Progress**: `docs/status/PRIORITY_2_PROGRESS.md`
- **Coverage Investigation**: `docs/status/COVERAGE_INVESTIGATION.md`
- **Component Status**: `COMPONENT_STATUS_INDEX.md`

### Support

For issues specific to Agent Agency V2:
1. Check `docs/status/COVERAGE_INVESTIGATION.md` for database-related findings
2. Review test setup in `tests/setup.ts`
3. Check connection pool configuration in `src/database/ConnectionPoolManager.ts`

---

**Document Version**: 1.0.0  
**Last Updated**: October 13, 2025  
**Maintained by**: @darianrosebrook  
**Questions?** Open an issue or check the troubleshooting section above.

