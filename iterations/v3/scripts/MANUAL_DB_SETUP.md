# Manual Database Setup Guide

> **Note:** For a streamlined setup experience, see the **[Getting Started Guide](../docs/GETTING_STARTED.md)** which covers the standard setup path.
>
> This document provides manual steps for edge cases where automated scripts fail.

Due to PostgreSQL authentication configuration issues, follow these steps manually:

## Step 1: Set Password for Your User

PostgreSQL requires a password even with trust authentication configured. Set one:

```bash
export PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH"

# Option A: If you can connect somehow, run:
psql -h 127.0.0.1 -U darianrosebrook -d postgres
# Then: ALTER USER darianrosebrook WITH PASSWORD 'your_password';

# Option B: Try connecting as postgres superuser (may have empty password):
psql -h 127.0.0.1 -U postgres -d postgres
# If that works, create your user:
CREATE USER darianrosebrook WITH PASSWORD 'your_password' SUPERUSER;
```

## Step 2: Create agent_agency User and Database

Once you can connect:

```sql
-- Create user
CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;

-- Create database (if not exists)
CREATE DATABASE agent_agency OWNER agent_agency;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE agent_agency TO agent_agency;
```

## Step 3: Enable pgvector

```sql
-- Connect to agent_agency database
\c agent_agency

-- Enable extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Verify
SELECT extname, extversion FROM pg_extension WHERE extname = 'vector';
```

## Step 4: Run Migrations

```bash
cd iterations/v3
export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"
cargo run --bin run_migrations --manifest-path data-infrastructure/Cargo.toml
```

## Alternative: Use Environment Variables

Instead of DATABASE_URL, you can use:

```bash
export DATABASE_USER=agent_agency
export DATABASE_PASSWORD=agent_agency_dev
export DATABASE_HOST=127.0.0.1
export DATABASE_PORT=5432
export DATABASE_NAME=agent_agency
```

## Troubleshooting

If you still can't connect:

1. Check PostgreSQL is running: `brew services list | grep postgresql`
2. Verify port: `pg_isready -h 127.0.0.1 -p 5432`
3. Check pg_hba.conf: `/opt/homebrew/var/postgresql@17/pg_hba.conf`
4. Try restarting: `brew services restart postgresql@17`

## Quick Setup Script (After Password is Set)

Once you have a working connection, you can run:

```bash
export PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH"
export PGPASSWORD="your_password"  # Or agent_agency_dev if using that user
./iterations/v3/scripts/setup_fresh_db.sh
```




