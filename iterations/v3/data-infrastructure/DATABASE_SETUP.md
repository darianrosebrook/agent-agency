# Database Setup for Schema Validation

## Current Database Status

You have PostgreSQL databases available:

1. **Docker Container: `agent-agency-v3-postgres`**
   - Port: `5433` (mapped from container port 5432)
   - Database: `agent_agency`
   - User: `postgres`
   - Password: `agent_agency_secure_password_123`
   - Connection: `postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency`

2. **Docker Container: `agent-agency-postgres`**
   - Check port mapping: `docker port agent-agency-postgres`

3. **Local PostgreSQL** (Homebrew)
   - Port: `5432` (default)
   - Check with: `psql --version`

## Quick Test

Test the schema validation against your V3 database:

```bash
cd iterations/v3/data-infrastructure

DATABASE_URL="postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency" \
  cargo run --features schema-validation --bin validate_schema
```

## Running Migrations

Before validation, ensure migrations are applied:

```bash
# Using the setup script
node scripts/v3/setup/setup-database-v3.js migrate

# Or manually using psql
psql "postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency" \
  -f iterations/v3/data-infrastructure/migrations/014_create_agent_management_tables.sql

psql "postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency" \
  -f iterations/v3/data-infrastructure/migrations/015_create_observation_tables.sql
```

## Expected Tables

After migrations 014 and 015, you should have:

**Migration 014:**
- `workers`
- `judges`
- `tasks`
- `task_executions`
- `council_verdicts`
- `judge_evaluations`
- `debate_sessions`

**Migration 015:**
- `saved_queries`
- `provenance_entries`
- `audit_trail_entries`
- `audit_logs`

## Troubleshooting

### Database Not Found

If validation can't connect:

1. **Check container is running:**
   ```bash
   docker ps | grep postgres
   ```

2. **Check port mapping:**
   ```bash
   docker port agent-agency-v3-postgres
   ```

3. **Test connection:**
   ```bash
   psql "postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency" -c "SELECT 1"
   ```

### Migrations Not Applied

Check which migrations have been applied:

```bash
psql "postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency" \
  -c "SELECT version, description, applied_at FROM migration_log ORDER BY version;"
```

If migrations 014 or 015 are missing, apply them using the setup script or manually.

### Validation Failures

If validation fails:

1. Check the error messages - they'll indicate which tables/fields are missing or mismatched
2. Verify migrations ran successfully
3. Check for manual schema changes that weren't migrated
4. Run with debug logging: `RUST_LOG=debug cargo run --bin validate_schema`

## Next Steps

1. **Apply migrations** if not already done
2. **Run validation** to verify schema matches models
3. **Fix any issues** found by validation
4. **Integrate into CI/CD** for continuous validation













