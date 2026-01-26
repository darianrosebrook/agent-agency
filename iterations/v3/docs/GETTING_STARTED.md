# Getting Started with Agent Agency V3

**Author:** @darianrosebrook

This guide provides the fastest path from cloning the repository to running the Agent Agency V3 system. For detailed workflows, see the [Developer Workflow Guide](./DEVELOPER_WORKFLOW_GUIDE.md).

---

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Rust | 1.75+ | Install via [rustup](https://rustup.rs/) |
| PostgreSQL | 15+ | With pgvector extension (Docker OR Homebrew) |
| Node.js | 18+ | For dashboard and CAWS tools |
| Docker | 20.10+ | Optional, for containerized database |

**Hardware Recommendations:**
- 16GB+ RAM (32GB recommended for local model execution)
- Apple Silicon (M1/M2/M3) recommended for CoreML acceleration
- 50GB+ free disk space

---

## Quick Start

Choose one of the following paths based on your environment:

### Option A: Docker (Recommended for New Developers)

This path uses Docker for PostgreSQL with pgvector, avoiding local installation complexity.

```bash
# 1. Navigate to v3 directory
cd iterations/v3

# 2. Start PostgreSQL with pgvector via Docker
docker-compose -f testing-validation/docker-compose.test.yml up -d

# 3. Wait for PostgreSQL to be ready (about 10 seconds)
sleep 10

# 4. Initialize the database with all migrations
DB_HOST=localhost DB_PORT=5433 DB_USER=test_user DB_NAME=agent_agency_test \
  PGPASSWORD=test_password ./scripts/init_fresh_database.sh

# 5. Build the workspace
SQLX_OFFLINE=true cargo build --workspace

# 6. Start the API server
export DATABASE_URL="postgresql://test_user:test_password@localhost:5433/agent_agency_test"
cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080
```

### Option B: Local PostgreSQL (macOS with Homebrew)

This path uses a local PostgreSQL installation, which is faster for development iteration.

```bash
# 1. Install PostgreSQL and pgvector
brew install postgresql@17 pgvector
brew services start postgresql@17

# 2. Navigate to v3 directory
cd iterations/v3

# 3. Create the database user and enable pgvector
./scripts/setup_fresh_db.sh

# 4. Initialize the database with all migrations
./scripts/init_fresh_database.sh

# 5. Build the workspace
SQLX_OFFLINE=true cargo build --workspace

# 6. Start the API server (from repository root)
cd ../..
./start-api-server.sh
```

**Note:** The `start-api-server.sh` script automatically starts PostgreSQL if it's not running and sets up the correct environment.

---

## SQLx Offline Mode

The V3 codebase uses SQLx for compile-time checked SQL queries. Normally, SQLx requires a running database to verify queries at compile time. However, the repository includes a pre-generated query cache in `.sqlx/` that enables **offline compilation**.

### When to Use Offline Mode

| Scenario | Use Offline Mode? |
|----------|-------------------|
| CI/CD pipelines | Yes |
| Quick code review/compilation | Yes |
| Running tests without database | Yes |
| Developing new database queries | No (need live database) |
| Running integration tests | No (need live database) |

### Using Offline Mode

```bash
# Build without a running database
SQLX_OFFLINE=true cargo build --workspace

# Run tests without a running database (unit tests only)
SQLX_OFFLINE=true cargo test --workspace

# Check compilation without a running database
SQLX_OFFLINE=true cargo check --workspace
```

### Regenerating the SQLx Cache

If you add new SQL queries or modify existing ones, regenerate the cache:

```bash
# Ensure database is running and migrations are applied
export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"

# Regenerate the cache
cargo sqlx prepare --workspace

# Commit the updated .sqlx/ directory
git add .sqlx/
git commit -m "Update SQLx query cache"
```

---

## Running the API Server

### Using the Wrapper Script (Recommended)

The repository includes a wrapper script that handles PostgreSQL startup and environment configuration:

```bash
# From the repository root
./start-api-server.sh
```

This script:
1. Checks if PostgreSQL is running (starts it via Homebrew if not)
2. Sets the correct `DATABASE_URL`
3. Configures Swift runtime library paths (for CoreML)
4. Starts the API server on port 8080

### Manual Startup

If you prefer manual control:

```bash
cd iterations/v3

# Set environment variables
export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"
export RUST_LOG=info

# Run the API server
cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080
```

### API Server Options

| Flag | Description | Default |
|------|-------------|---------|
| `--host` | Bind address | `127.0.0.1` |
| `--port` | Listen port | `8080` |
| `--enable-cors` | Enable CORS headers | Disabled |

---

## Verifying the Setup

After starting the API server, verify everything is working:

```bash
# Health check
curl http://localhost:8080/api/v1/system/health

# Expected response:
# {"status":"healthy","database":"connected","version":"0.1.0"}

# List tasks (should return empty array initially)
curl http://localhost:8080/api/v1/tasks

# List projects
curl http://localhost:8080/api/v1/projects

# List agents
curl http://localhost:8080/api/v1/agents
```

### Submitting a Test Task

```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Test task for verification",
    "execution_mode": "dry-run",
    "risk_tier": "3"
  }'
```

---

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency` | Full database connection string |
| `DB_HOST` | `127.0.0.1` | Database host (use IP to avoid DNS issues) |
| `DB_PORT` | `5432` | Database port |
| `DB_USER` | `agent_agency` | Database user |
| `DB_PASSWORD` | `agent_agency_dev` | Database password |
| `DB_NAME` | `agent_agency` | Database name |
| `SQLX_OFFLINE` | `false` | Enable offline SQLx compilation |
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |

---

## Common Issues and Troubleshooting

### PostgreSQL Connection Refused

```
Error: Connection refused (os error 61)
```

**Solution:** Ensure PostgreSQL is running:
```bash
# For Homebrew installation
brew services start postgresql@17

# For Docker
docker-compose -f testing-validation/docker-compose.test.yml up -d
```

### Authentication Failed

```
Error: password authentication failed for user "agent_agency"
```

**Solution:** Run the setup script to create the user:
```bash
./scripts/setup_fresh_db.sh
```

Or manually create the user:
```bash
psql -U postgres -c "CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;"
```

### pgvector Extension Not Found

```
Error: extension "vector" is not available
```

**Solution:** Install pgvector:
```bash
# Homebrew
brew install pgvector

# Then enable in database
psql -U agent_agency -d agent_agency -c "CREATE EXTENSION IF NOT EXISTS vector;"
```

### SQLx Compile Errors Without Database

```
Error: error communicating with database: Connection refused
```

**Solution:** Use offline mode:
```bash
SQLX_OFFLINE=true cargo build --workspace
```

### GSSAPI Authentication Errors

```
Error: GSSAPI authentication failed
```

**Solution:** Use IP address instead of hostname and disable GSSAPI:
```bash
export PGGSSENCMODE=disable
export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"
```

### Missing Tables After Migration

If the API server reports missing tables:

```bash
# Re-run migrations
./scripts/init_fresh_database.sh

# Verify critical tables exist
psql -U agent_agency -d agent_agency -c "\dt"
```

---

## Next Steps

Once your environment is set up:

1. **[Developer Workflow Guide](./DEVELOPER_WORKFLOW_GUIDE.md)** - Learn task execution workflows and CAWS compliance
2. **[System Architecture](./README.md)** - Understand the component organization
3. **[API Documentation](./interaction-contracts.md)** - REST API endpoints and contracts
4. **[Database Schema](./database/provenance.md)** - Database design and migrations

### Running the Web Dashboard

```bash
cd apps/agent_management_dashboard
pnpm install
pnpm dev
# Access at http://localhost:3000
```

### Running Tests

```bash
cd iterations/v3

# Unit tests (can run without database)
SQLX_OFFLINE=true cargo test --workspace

# Integration tests (requires running database)
cargo test --workspace
```

---

## Related Documentation

- [DATABASE_INITIALIZATION.md](../DATABASE_INITIALIZATION.md) - Detailed Docker setup and migration reference
- [LOCAL_POSTGRES_SETUP.md](./LOCAL_POSTGRES_SETUP.md) - Homebrew PostgreSQL troubleshooting
- [MANUAL_DB_SETUP.md](../scripts/MANUAL_DB_SETUP.md) - Manual database configuration for edge cases
