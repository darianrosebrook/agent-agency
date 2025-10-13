# Database Setup Guide

**Project**: Agent Agency V2  
**Author**: @darianrosebrook  
**Date**: October 13, 2025  
**Last Updated**: October 13, 2025

---

## Overview

Agent Agency V2 uses **PostgreSQL** as its primary database for:
- Agent registry and capability tracking
- Task queue and assignment management
- Knowledge graph and query storage
- Performance metrics and tracking
- Provenance and audit logging
- Multi-tenant isolation
- Hybrid search with pg_trgm and vector embeddings

This guide covers:
1. Prerequisites and installation
2. Database creation and configuration
3. Migration execution
4. Verification and troubleshooting
5. Development vs Production setup
6. Common issues and solutions

---

## Quick Start (TL;DR)

```bash
# 1. Install PostgreSQL
brew install postgresql@15  # macOS
# sudo apt-get install postgresql-15  # Ubuntu/Debian

# 2. Start PostgreSQL
brew services start postgresql@15  # macOS
# sudo systemctl start postgresql  # Linux

# 3. Create role and database
createuser -s postgres
createdb agent_agency_v2_test -O postgres

# 4. Configure environment
cp .env.example .env.test
# Edit .env.test with database credentials

# 5. Run migrations
psql -U postgres -d agent_agency_v2_test -f migrations/001_create_agent_registry_tables.sql
psql -U postgres -d agent_agency_v2_test -f migrations/002_create_task_queue_tables.sql
psql -U postgres -d agent_agency_v2_test -f migrations/003_create_knowledge_tables.sql
# ... continue with remaining migrations

# 6. Verify
npm run test:integration
```

---

## Prerequisites

### 1. PostgreSQL Installation

**Minimum Version**: PostgreSQL 15.0+  
**Recommended**: PostgreSQL 15.4 or 16.x

**macOS (Homebrew)**:
```bash
# Install PostgreSQL 15
brew install postgresql@15

# Start PostgreSQL service
brew services start postgresql@15

# Verify installation
psql --version
# Expected: psql (PostgreSQL) 15.x
```

**Ubuntu/Debian**:
```bash
# Add PostgreSQL repository
sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

# Install PostgreSQL
sudo apt-get update
sudo apt-get install postgresql-15

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql  # Start on boot

# Verify installation
psql --version
```

**Windows**:
```powershell
# Download installer from https://www.postgresql.org/download/windows/
# Run installer and follow prompts
# Add PostgreSQL bin to PATH

# Verify installation
psql --version
```

### 2. Required PostgreSQL Extensions

Agent Agency V2 requires these extensions:
- `uuid-ossp` - UUID generation
- `pg_trgm` - Trigram-based fuzzy text search
- `btree_gin` - GIN indexes for B-tree data types
- `hstore` (optional) - Key-value pairs

These are installed automatically by migration scripts.

---

## Database Setup

### Step 1: Create PostgreSQL Role

The application expects a `postgres` superuser role.

**Create role**:
```bash
# Check if role exists
psql -U $USER -d postgres -c "\du postgres"

# If role doesn't exist, create it
createuser -s postgres

# Verify
psql -U postgres -d postgres -c "SELECT current_user;"
# Expected: postgres
```

**Alternative: Use custom role**:
```bash
# Create custom role with permissions
createuser -s agent_agency_admin
createdb -O agent_agency_admin agent_agency_v2_test

# Update .env.test with custom role
PGUSER=agent_agency_admin
```

### Step 2: Create Test Database

```bash
# Create test database
createdb agent_agency_v2_test -O postgres

# Verify database exists
psql -U postgres -l | grep agent_agency_v2_test
# Expected: agent_agency_v2_test | postgres | UTF8 | ...

# Verify connection
psql -U postgres -d agent_agency_v2_test -c "SELECT 1 AS health;"
# Expected:
#  health 
# --------
#       1
```

**For Production**:
```bash
# Create production database
createdb agent_agency_v2_production -O postgres

# Set stricter permissions
psql -U postgres -d agent_agency_v2_production -c "
  REVOKE ALL ON SCHEMA public FROM public;
  GRANT ALL ON SCHEMA public TO postgres;
"
```

### Step 3: Configure Environment

Create `.env.test` configuration file:

```bash
# Copy example if it exists
cp .env.example .env.test

# Or create manually
cat > .env.test << 'EOF'
# Test Environment Configuration
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

**Security Note**: For production, use strong passwords and environment-specific credentials.

### Step 4: Run Migrations

Execute migrations in order:

```bash
# Navigate to project root
cd /path/to/agent-agency/iterations/v2

# Run migrations sequentially
psql -U postgres -d agent_agency_v2_test -f migrations/001_create_agent_registry_tables.sql
psql -U postgres -d agent_agency_v2_test -f migrations/002_create_task_queue_tables.sql
psql -U postgres -d agent_agency_v2_test -f migrations/003_create_knowledge_tables.sql
psql -U postgres -d agent_agency_v2_test -f migrations/005_task_research_provenance.sql
psql -U postgres -d agent_agency_v2_test -f migrations/006_create_knowledge_graph_schema.sql
psql -U postgres -d agent_agency_v2_test -f migrations/007_add_multi_tenant_isolation.sql
```

**Note**: Some migrations (004_, 006_) have multiple files. They can be run in any order within their number group.

**Automated Migration Script**:
```bash
# Create migration runner
cat > scripts/run-migrations.sh << 'EOF'
#!/bin/bash
set -e

DB_USER=${1:-postgres}
DB_NAME=${2:-agent_agency_v2_test}

echo "Running migrations for ${DB_NAME}..."

# Core migrations
for migration in migrations/00[1-3]*.sql migrations/005*.sql migrations/006_create_knowledge_graph_schema.sql migrations/007*.sql; do
  if [ -f "$migration" ]; then
    echo "Running: $migration"
    psql -U "$DB_USER" -d "$DB_NAME" -f "$migration" -v ON_ERROR_STOP=1
  fi
done

echo "Migrations complete!"
EOF

chmod +x scripts/run-migrations.sh

# Run migrations
./scripts/run-migrations.sh postgres agent_agency_v2_test
```

### Step 5: Verify Setup

```bash
# 1. Check tables were created
psql -U postgres -d agent_agency_v2_test -c "\dt"

# Expected output (6 core tables):
#  Schema |           Name           | Type  |  Owner   
# --------+--------------------------+-------+----------
#  public | agent_capabilities       | table | postgres
#  public | agent_profiles           | table | postgres
#  public | knowledge_queries        | table | postgres
#  public | task_assignments         | table | postgres
#  public | task_queue               | table | postgres
#  public | task_research_provenance | table | postgres

# 2. Check extensions
psql -U postgres -d agent_agency_v2_test -c "\dx"

# Expected extensions:
#  uuid-ossp | pg_trgm | btree_gin

# 3. Test connection from application
npm run test:integration -- --testPathPattern="health"

# 4. Verify connection pooling
node -e "
const { ConnectionPoolManager } = require('./src/database/ConnectionPoolManager');
const manager = new ConnectionPoolManager({
  host: 'localhost',
  port: 5432,
  database: 'agent_agency_v2_test',
  user: 'postgres',
  password: ''
});
manager.initialize().then(() => {
  manager.healthCheck().then(healthy => {
    console.log('Health check:', healthy ? 'PASS' : 'FAIL');
    process.exit(healthy ? 0 : 1);
  });
});
"
```

---

## Migration Details

### Migration Order and Dependencies

```
001_create_agent_registry_tables.sql
  └─ Creates: agent_profiles, agent_capabilities
  └─ Dependencies: None
  └─ Extensions: uuid-ossp

002_create_task_queue_tables.sql
  └─ Creates: task_queue, task_assignments
  └─ Dependencies: 001 (references agent_profiles)
  └─ Extensions: None

003_create_knowledge_tables.sql
  └─ Creates: knowledge_queries
  └─ Dependencies: 001 (references agent_profiles)
  └─ Extensions: pg_trgm

005_task_research_provenance.sql
  └─ Creates: task_research_provenance
  └─ Dependencies: 002 (references task_queue)
  └─ Extensions: None

006_create_knowledge_graph_schema.sql
  └─ Creates: Knowledge graph tables
  └─ Dependencies: 001, 003
  └─ Extensions: uuid-ossp, pg_trgm, btree_gin

007_add_multi_tenant_isolation.sql
  └─ Creates: Multi-tenant enums and tables
  └─ Dependencies: All previous
  └─ Extensions: None
```

### Migration Rollback

**Individual Migration Rollback**:
```bash
# Create rollback script
cat > migrations/rollback_001.sql << 'EOF'
-- Rollback 001_create_agent_registry_tables.sql

BEGIN;

DROP TABLE IF EXISTS agent_capabilities CASCADE;
DROP TABLE IF EXISTS agent_profiles CASCADE;
DROP EXTENSION IF EXISTS "uuid-ossp";

COMMIT;
EOF

# Execute rollback
psql -U postgres -d agent_agency_v2_test -f migrations/rollback_001.sql
```

**Full Database Reset**:
```bash
# Drop and recreate database
dropdb agent_agency_v2_test
createdb agent_agency_v2_test -O postgres

# Re-run migrations
./scripts/run-migrations.sh
```

---

## Development vs Production Setup

### Development Setup

**Characteristics**:
- Permissive permissions
- No SSL required
- Single database instance
- Verbose logging
- Connection pooling (min: 2, max: 10)

**Configuration** (`.env.development`):
```bash
DATABASE_URL=postgresql://postgres@localhost:5432/agent_agency_v2_dev
DB_POOL_MIN=2
DB_POOL_MAX=10
LOG_LEVEL=debug
```

### Test Setup

**Characteristics**:
- Isolated test database
- Fast connection pooling
- Transaction rollback after tests
- Minimal logging
- Connection pooling (min: 2, max: 10)

**Configuration** (`.env.test`):
```bash
DATABASE_URL=postgresql://postgres@localhost:5432/agent_agency_v2_test
DB_POOL_MIN=2
DB_POOL_MAX=10
LOG_LEVEL=error
NODE_ENV=test
```

**Test Database Best Practices**:
```typescript
// tests/setup.ts
beforeEach(async () => {
  // Start transaction
  await db.query('BEGIN');
});

afterEach(async () => {
  // Rollback transaction (clean slate for next test)
  await db.query('ROLLBACK');
});
```

### Production Setup

**Characteristics**:
- Strict permissions (least privilege)
- SSL/TLS required
- High-availability setup (primary + replicas)
- Connection pooling (min: 10, max: 100)
- Backup and recovery
- Monitoring and alerts

**Configuration** (`.env.production`):
```bash
DATABASE_URL=postgresql://agent_app:STRONG_PASSWORD@db-primary.example.com:5432/agent_agency_v2_production?ssl=true&sslmode=require
DB_POOL_MIN=10
DB_POOL_MAX=100
DB_POOL_IDLE_TIMEOUT=10000
DB_POOL_CONNECTION_TIMEOUT=3000
LOG_LEVEL=warn
NODE_ENV=production
```

**Production Security**:
```sql
-- Create application user with limited privileges
CREATE USER agent_app WITH PASSWORD 'STRONG_PASSWORD';

-- Grant only necessary permissions
GRANT CONNECT ON DATABASE agent_agency_v2_production TO agent_app;
GRANT USAGE ON SCHEMA public TO agent_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO agent_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO agent_app;

-- Revoke dangerous permissions
REVOKE CREATE ON SCHEMA public FROM agent_app;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
```

---

## Troubleshooting

### Issue 1: Role "postgres" does not exist

**Error**:
```
error: role "postgres" does not exist
```

**Solution**:
```bash
# Create postgres superuser
createuser -s postgres

# Or use your system user
createuser -s $USER
```

---

### Issue 2: Database does not exist

**Error**:
```
FATAL: database "agent_agency_v2_test" does not exist
```

**Solution**:
```bash
# Create database
createdb agent_agency_v2_test -O postgres

# Verify
psql -U postgres -l | grep agent_agency_v2_test
```

---

### Issue 3: Connection refused

**Error**:
```
Error: connect ECONNREFUSED 127.0.0.1:5432
```

**Solution**:
```bash
# Check if PostgreSQL is running
pg_isready

# If not running, start it
brew services start postgresql@15  # macOS
sudo systemctl start postgresql    # Linux

# Check port
lsof -i :5432

# If port conflict, update .env.test
PGPORT=5433
```

---

### Issue 4: Permission denied

**Error**:
```
ERROR: permission denied for table agent_profiles
```

**Solution**:
```sql
-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;
```

---

### Issue 5: Extension not found

**Error**:
```
ERROR: could not open extension control file
```

**Solution**:
```bash
# Install PostgreSQL contrib package
brew install postgresql@15  # Includes contrib
sudo apt-get install postgresql-contrib-15

# Verify extensions available
psql -U postgres -d agent_agency_v2_test -c "SELECT * FROM pg_available_extensions WHERE name IN ('uuid-ossp', 'pg_trgm', 'btree_gin');"
```

---

### Issue 6: Migration fails midway

**Error**:
```
ERROR: relation "agent_profiles" already exists
```

**Solution**:
```bash
# Check which tables exist
psql -U postgres -d agent_agency_v2_test -c "\dt"

# Option 1: Continue from next migration
# Skip migrations that created existing tables

# Option 2: Full reset
dropdb agent_agency_v2_test
createdb agent_agency_v2_test -O postgres
./scripts/run-migrations.sh
```

---

### Issue 7: Connection pool exhausted

**Error**:
```
Error: Connection pool exhausted
```

**Solution**:
```bash
# Increase pool size in .env
DB_POOL_MAX=20  # Increase from 10

# Check active connections
psql -U postgres -d agent_agency_v2_test -c "
  SELECT count(*) as active_connections 
  FROM pg_stat_activity 
  WHERE datname = 'agent_agency_v2_test';
"

# Kill idle connections
psql -U postgres -d agent_agency_v2_test -c "
  SELECT pg_terminate_backend(pid) 
  FROM pg_stat_activity 
  WHERE datname = 'agent_agency_v2_test' 
    AND state = 'idle' 
    AND state_change < current_timestamp - interval '5 minutes';
"
```

---

## Database Maintenance

### Backup

**Development Backup**:
```bash
# Backup database
pg_dump -U postgres agent_agency_v2_test > backups/agent_agency_v2_test_$(date +%Y%m%d).sql

# Restore from backup
psql -U postgres -d agent_agency_v2_test < backups/agent_agency_v2_test_20251013.sql
```

**Production Backup**:
```bash
# Automated daily backup
cat > scripts/backup-database.sh << 'EOF'
#!/bin/bash
set -e

BACKUP_DIR="/var/backups/postgresql"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DB_NAME="agent_agency_v2_production"

# Create backup
pg_dump -U postgres "$DB_NAME" | gzip > "$BACKUP_DIR/${DB_NAME}_${TIMESTAMP}.sql.gz"

# Keep last 30 days
find "$BACKUP_DIR" -name "${DB_NAME}_*.sql.gz" -mtime +30 -delete

echo "Backup complete: ${DB_NAME}_${TIMESTAMP}.sql.gz"
EOF

chmod +x scripts/backup-database.sh

# Add to crontab (daily at 2 AM)
crontab -e
# Add: 0 2 * * * /path/to/agent-agency/scripts/backup-database.sh
```

### Vacuum and Analyze

```bash
# Analyze database (update statistics)
psql -U postgres -d agent_agency_v2_test -c "ANALYZE;"

# Vacuum (reclaim space)
psql -U postgres -d agent_agency_v2_test -c "VACUUM;"

# Full vacuum (more thorough, locks tables)
psql -U postgres -d agent_agency_v2_test -c "VACUUM FULL;"
```

### Monitor Database Size

```sql
-- Database size
SELECT pg_size_pretty(pg_database_size('agent_agency_v2_test'));

-- Table sizes
SELECT 
  table_name,
  pg_size_pretty(pg_total_relation_size(quote_ident(table_name))) AS size
FROM information_schema.tables
WHERE table_schema = 'public'
ORDER BY pg_total_relation_size(quote_ident(table_name)) DESC;

-- Index sizes
SELECT
  indexname,
  pg_size_pretty(pg_relation_size(quote_ident(indexname))) AS size
FROM pg_indexes
WHERE schemaname = 'public'
ORDER BY pg_relation_size(quote_ident(indexname)) DESC;
```

---

## Connection Pooling

### Configuration

**Recommended Pool Sizes**:
- Development: 2-10 connections
- Test: 2-10 connections
- Production: 10-100 connections

**Pool Configuration**:
```typescript
// src/database/ConnectionPoolManager.ts
const poolConfig = {
  host: process.env.PGHOST,
  port: parseInt(process.env.PGPORT || '5432'),
  database: process.env.PGDATABASE,
  user: process.env.PGUSER,
  password: process.env.PGPASSWORD,
  
  // Pool settings
  min: parseInt(process.env.DB_POOL_MIN || '2'),
  max: parseInt(process.env.DB_POOL_MAX || '10'),
  idleTimeoutMillis: parseInt(process.env.DB_POOL_IDLE_TIMEOUT || '30000'),
  connectionTimeoutMillis: parseInt(process.env.DB_POOL_CONNECTION_TIMEOUT || '5000'),
};
```

### Health Check

```typescript
// Health check endpoint
async function healthCheck(): Promise<boolean> {
  try {
    const result = await pool.query('SELECT 1 AS health');
    return result.rows[0].health === 1;
  } catch (error) {
    console.error('[ConnectionPool] Health check failed:', error);
    return false;
  }
}
```

---

## Additional Resources

### Documentation

- **PostgreSQL Docs**: https://www.postgresql.org/docs/15/
- **pg_trgm Extension**: https://www.postgresql.org/docs/15/pgtrgm.html
- **Connection Pooling**: https://node-postgres.com/features/pooling

### Tools

- **pgAdmin**: GUI for PostgreSQL management
- **DBeaver**: Universal database tool
- **psql**: Command-line interface (included with PostgreSQL)

### Monitoring

- **pg_stat_activity**: View active connections
- **pg_stat_database**: Database-wide statistics
- **pg_stat_user_tables**: Table access statistics
- **pg_stat_user_indexes**: Index usage statistics

---

## Summary Checklist

Setup complete when all checks pass:

- [ ] PostgreSQL 15+ installed and running
- [ ] `postgres` role created
- [ ] `agent_agency_v2_test` database created
- [ ] `.env.test` configured
- [ ] All migrations executed successfully
- [ ] 6 core tables exist
- [ ] Extensions installed (uuid-ossp, pg_trgm, btree_gin)
- [ ] Connection health check passes
- [ ] Integration tests pass

---

**For questions or issues, see**: `docs/database/TROUBLESHOOTING.md` or file an issue in the project repository.

**Author**: @darianrosebrook  
**Last Updated**: October 13, 2025

