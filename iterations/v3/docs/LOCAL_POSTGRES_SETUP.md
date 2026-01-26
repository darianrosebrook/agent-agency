# Local PostgreSQL Setup (Non-Docker)

> **Note:** For a streamlined setup experience, see the **[Getting Started Guide](./GETTING_STARTED.md)** which covers the standard setup path.
>
> This document provides detailed troubleshooting for local PostgreSQL authentication issues.

This document describes how to set up a local PostgreSQL database for development, replacing the Docker-based setup.

## Prerequisites

- PostgreSQL 17 installed via Homebrew: `brew install postgresql@17`
- pgvector extension: `brew install pgvector`
- PostgreSQL service running: `brew services start postgresql@17`

## Current Status

The database `agent_agency` has been created, but authentication configuration needs to be resolved.

## Authentication Issue

PostgreSQL is configured with `trust` authentication in `/opt/homebrew/var/postgresql@17/pg_hba.conf`, but connections are still requiring passwords. This may be due to:

1. PostgreSQL not reloading configuration
2. Client-side GSSAPI authentication interfering
3. Need to set explicit password for user

## Recommended Solution

### Option 1: Set Password for Current User (Recommended)

```bash
# Connect to PostgreSQL and set password for your user
psql -h 127.0.0.1 -U darianrosebrook -d postgres
# Then run: ALTER USER darianrosebrook WITH PASSWORD 'your_password';
```

Then use:
```bash
export DATABASE_URL="postgresql://darianrosebrook:your_password@127.0.0.1:5432/agent_agency"
```

### Option 2: Create agent_agency User Manually

Once you can connect (using Option 1 or another method):

```sql
CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
GRANT ALL PRIVILEGES ON DATABASE agent_agency TO agent_agency;
```

Then connect to agent_agency database:
```sql
\c agent_agency
CREATE EXTENSION IF NOT EXISTS vector;
```

### Option 3: Use PostgreSQL Superuser

If a `postgres` superuser exists:
```bash
psql -U postgres -d postgres
```

## Database Setup Steps

1. **Create database** (already done):
   ```bash
   createdb --host=localhost --username=darianrosebrook agent_agency
   ```

2. **Create user and enable pgvector**:
   - Connect using one of the methods above
   - Run the SQL commands from Option 2

3. **Run migrations**:
   ```bash
   cd iterations/v3
   export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"
   cargo run --bin run_migrations --manifest-path data-infrastructure/Cargo.toml
   ```

## Configuration

Update your environment or `.env` file:

```bash
# Option A: Using agent_agency user
export DATABASE_URL="postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency"

# Option B: Using individual variables
export DATABASE_USER=agent_agency
export DATABASE_PASSWORD=agent_agency_dev
export DATABASE_HOST=127.0.0.1
export DATABASE_PORT=5432
export DATABASE_NAME=agent_agency
```

## Troubleshooting

### Connection Refused
- Ensure PostgreSQL is running: `brew services list | grep postgresql`
- Check port: `pg_isready -h 127.0.0.1 -p 5432`

### Authentication Failed
- Check `pg_hba.conf`: `/opt/homebrew/var/postgresql@17/pg_hba.conf`
- Ensure `trust` is set for `127.0.0.1/32`
- Restart PostgreSQL: `brew services restart postgresql@17`

### GSSAPI Errors
- Set `PGGSSENCMODE=disable` in environment
- Or use IPv4 explicitly: `127.0.0.1` instead of `localhost`

## Next Steps

1. Resolve authentication to enable user creation
2. Enable pgvector extension
3. Run migrations: `cargo run --bin run_migrations`
4. Update application configuration to use local database
5. Test database connectivity from application

## Migration from Docker

Once local PostgreSQL is working:

1. Stop Docker PostgreSQL: `docker-compose down` (if running)
2. Update `docker-compose.yml` to comment out postgres service or remove it
3. Update documentation to reflect local setup as default
4. Update CI/CD if needed to use local PostgreSQL for tests





