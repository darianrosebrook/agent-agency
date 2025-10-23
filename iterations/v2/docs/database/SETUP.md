# Database Setup Guide for Agent Agency V2

**Version**: 2.0
**Last Updated**: October 14, 2025

---

## Quick Start

### Prerequisites

- **PostgreSQL 16+** with pgvector support
- **Node.js 18+** for running setup scripts
- **Database superuser access** for initial setup

### One-Command Setup

```bash
# Set required environment variables
export DB_PASSWORD="your_postgres_password"
export DB_APP_PASSWORD="secure_app_password"

# Run full setup
node scripts/setup-database.js init
```

That's it! The script handles everything: role creation, database setup, extensions, and migrations.

---

## Detailed Setup

### Step 1: Install PostgreSQL

#### macOS (with Homebrew)

```bash
brew install postgresql@16
brew services start postgresql@16

# Install pgvector
brew install pgvector/pgvector/pgvector
```

#### Ubuntu/Debian

```bash
# Add PostgreSQL repository
sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

sudo apt update
sudo apt install postgresql-16 postgresql-16-pgvector

sudo systemctl start postgresql
```

#### Docker

```bash
# Run PostgreSQL with pgvector
docker run -d \
  --name agent-agency-db \
  -e POSTGRES_PASSWORD=mypassword \
  -e POSTGRES_DB=agent_agency_v2 \
  -p 5432:5432 \
  pgvector/pgvector:pg16
```

### Step 2: Configure Environment

Create a `.env` file in the project root:

```bash
# Database Connection
DB_HOST=localhost
DB_PORT=5432
DB_NAME=agent_agency_v2
DB_USER=postgres
DB_PASSWORD=your_postgres_password

# Setup Credentials (superuser access needed for initial setup)
DB_SUPERUSER=postgres
DB_SUPERPASSWORD=your_postgres_password

# Application Database User (created during setup)
DB_APP_PASSWORD=secure_password_for_app_user

# SSL (set to 'true' for production)
DB_SSL=false
```

### Step 3: Run Setup Script

#### Full Automated Setup

```bash
node scripts/setup-database.js init
```

This command:

1. Creates PostgreSQL role `agent_agency_app`
2. Creates database `agent_agency_v2`
3. Installs pgvector and required extensions
4. Runs all migrations (8 migration files)
5. Verifies setup completeness

#### Manual Setup Steps (if needed)

If you prefer manual control or the automated script fails:

```bash
# 1. Create database role
psql -U postgres -c "CREATE ROLE agent_agency_app LOGIN PASSWORD 'secure_password';"

# 2. Create database
psql -U postgres -c "CREATE DATABASE agent_agency_v2 OWNER agent_agency_app;"

# 3. Install extensions
psql -U postgres -d agent_agency_v2 -c "CREATE EXTENSION IF NOT EXISTS vector;"
psql -U postgres -d agent_agency_v2 -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";"
psql -U postgres -d agent_agency_v2 -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

# 4. Run migrations
node scripts/setup-database.js migrate
```

### Step 4: Verify Setup

```bash
# Quick verification
node scripts/setup-database.js verify

# Detailed status
node scripts/setup-database.js status
```

Expected output:

```
Database connection
Migrations table
Core tables
RLS policies
All verification checks passed!
```

---

## Environment Variables Reference

### Required Variables

| Variable          | Description                   | Default                     | Required        |
| ----------------- | ----------------------------- | --------------------------- | --------------- |
| `DB_PASSWORD`     | PostgreSQL superuser password | `""`                        | Yes (for setup) |
| `DB_APP_PASSWORD` | App database user password    | `"change_me_in_production"` | Yes             |

### Connection Variables

| Variable  | Description            | Default             | Notes                         |
| --------- | ---------------------- | ------------------- | ----------------------------- |
| `DB_HOST` | PostgreSQL server host | `"localhost"`       | Use container name in Docker  |
| `DB_PORT` | PostgreSQL server port | `5432`              | Default PostgreSQL port       |
| `DB_NAME` | Database name          | `"agent_agency_v2"` | Application database          |
| `DB_USER` | App database username  | `"postgres"`        | For application connections   |
| `DB_SSL`  | Enable SSL connections | `"false"`           | Set to `"true"` in production |

### Setup Variables

| Variable           | Description          | Default       | Notes                               |
| ------------------ | -------------------- | ------------- | ----------------------------------- |
| `DB_SUPERUSER`     | PostgreSQL superuser | `"postgres"`  | Must have CREATE DATABASE privilege |
| `DB_SUPERPASSWORD` | Superuser password   | `DB_PASSWORD` | Usually same as DB_PASSWORD         |

---

## Troubleshooting

### "FATAL: role 'postgres' does not exist"

**Cause**: PostgreSQL superuser role not set up.

**Solution**:

```bash
# macOS/Homebrew
createuser -s postgres

# Ubuntu/Debian
sudo -u postgres createuser -s postgres
```

### "could not connect to server"

**Cause**: PostgreSQL not running or wrong connection details.

**Solution**:

```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# macOS/Homebrew
brew services list | grep postgresql
brew services start postgresql@16

# Ubuntu/Debian
sudo systemctl status postgresql
sudo systemctl start postgresql
```

### "extension 'vector' does not exist"

**Cause**: pgvector not installed.

**Solution**:

```bash
# macOS/Homebrew
brew install pgvector/pgvector/pgvector

# Ubuntu/Debian (from source)
git clone https://github.com/pgvector/pgvector.git
cd pgvector
make
sudo make install
```

### "permission denied to create extension"

**Cause**: Database user lacks superuser privileges for extension creation.

**Solution**:

```sql
-- Connect as superuser
psql -U postgres

-- Grant superuser temporarily for extension installation
ALTER USER agent_agency_app SUPERUSER;

-- Install extensions
\c agent_agency_v2
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Revoke superuser privileges
ALTER USER agent_agency_app NOSUPERUSER;
```

### Migration Errors

**Cause**: Migration files corrupted or database state inconsistent.

**Solution**:

```bash
# Check migration status
node scripts/setup-database.js status

# Reset and re-run (⚠️ DESTRUCTIVE)
node scripts/setup-database.js clean
node scripts/setup-database.js init
```

---

## Production Deployment

### Security Considerations

1. **Change default passwords**:

   ```bash
   export DB_APP_PASSWORD="strong_random_password"
   ```

2. **Enable SSL**:

   ```bash
   export DB_SSL=true
   ```

3. **Use dedicated database user**:

   - Don't use `postgres` superuser for app connections
   - Create dedicated user with minimal privileges

4. **Network security**:
   - Restrict database access to application servers only
   - Use VPC/security groups to limit network access

### Backup Strategy

```bash
# Daily backups
pg_dump agent_agency_v2 > backup_$(date +%Y%m%d).sql

# With compression
pg_dump agent_agency_v2 | gzip > backup_$(date +%Y%m%d).sql.gz

# Restore from backup
psql agent_agency_v2 < backup_20251014.sql
```

### Monitoring

Key metrics to monitor:

- Connection count (`SELECT count(*) FROM pg_stat_activity WHERE datname = 'agent_agency_v2'`)
- Long-running queries
- Database size growth
- Vector search performance

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Database Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: pgvector/pgvector:pg16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: "18"

      - name: Install dependencies
        run: npm ci

      - name: Setup test database
        run: node scripts/setup-database.js init
        env:
          DB_PASSWORD: postgres
          DB_APP_PASSWORD: test_password

      - name: Run tests
        run: npm test
        env:
          DATABASE_URL: postgresql://agent_agency_app:test_password@localhost:5432/agent_agency_v2
```

---

## Database Schema Overview

After setup, you'll have:

### Core Tables

- `tenants` - Multi-tenant organization data
- `agent_profiles` - Agent metadata and capabilities
- `performance_events` - Time-series performance metrics
- `agent_capabilities_graph` - Agent capability relationships
- `agent_relationships` - Inter-agent dependencies

### Extensions

- `pgvector` - Vector similarity search
- `uuid-ossp` - UUID generation
- `pgcrypto` - Cryptographic functions

### Security

- Row Level Security (RLS) policies
- Tenant-based data isolation
- Application-level connection pooling

---

## Next Steps

After database setup:

1. **Run integration tests**:

   ```bash
   npm run test:integration
   ```

2. **Start the application**:

   ```bash
   npm run dev
   ```

3. **Monitor database health**:
   ```bash
   # Check connection pool status
   curl http://localhost:3000/health/database
   ```

---

**Questions?** Check the [Database Documentation Index](./README.md) or create an issue.
