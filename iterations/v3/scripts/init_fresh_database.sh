#!/bin/bash
# Initialize a fresh Agent Agency V3 database with all migrations
# This script applies all migrations in order to create a complete schema
# @author @darianrosebrook
#
# Environment Variables (with defaults):
#   DB_HOST     - Database host (default: 127.0.0.1)
#   DB_PORT     - Database port (default: 5432)
#   DB_USER     - Database user (default: agent_agency)
#   DB_NAME     - Database name (default: agent_agency)
#   PGPASSWORD  - Database password (default: agent_agency_dev)
#
# For Docker test environment, override with:
#   DB_PORT=5433 DB_USER=test_user DB_NAME=agent_agency_test PGPASSWORD=test_password ./init_fresh_database.sh

set -e

# Configuration - defaults match standard local development setup
DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-agent_agency}"
DB_NAME="${DB_NAME:-agent_agency}"
PGPASSWORD="${PGPASSWORD:-agent_agency_dev}"
export PGPASSWORD

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIGRATIONS_DIR="$SCRIPT_DIR/../data-infrastructure/migrations"

echo "=== Agent Agency V3 Database Initialization ==="
echo "Database: $DB_NAME on $DB_HOST:$DB_PORT"
echo "Migrations: $MIGRATIONS_DIR"
echo ""

# Test connection
echo "Testing database connection..."
if ! psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" > /dev/null 2>&1; then
    echo "ERROR: Cannot connect to database. Please ensure PostgreSQL is running."
    exit 1
fi
echo "Connection successful!"
echo ""

# Create migration_log table
echo "Creating migration_log table..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
CREATE TABLE IF NOT EXISTS migration_log (
    version VARCHAR(255) PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);" > /dev/null 2>&1

# Apply migrations in order
echo "Applying migrations..."
echo ""

apply_migration() {
    local file="$1"
    local version=$(basename "$file" | cut -d'_' -f1)
    local filename=$(basename "$file")
    
    # Check if already applied
    local applied=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc \
        "SELECT COUNT(*) FROM migration_log WHERE version = '$version';" 2>/dev/null || echo "0")
    
    if [ "$applied" = "1" ]; then
        echo "  [SKIP] $filename (already applied)"
        return 0
    fi
    
    echo -n "  [APPLY] $filename... "
    
    # Apply migration
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
        -v ON_ERROR_STOP=0 \
        -f "$file" > /tmp/migration_output.txt 2>&1; then
        
        # Record in migration_log
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c \
            "INSERT INTO migration_log (version, description) VALUES ('$version', '$filename') ON CONFLICT DO NOTHING;" > /dev/null 2>&1
        
        echo "OK"
        return 0
    else
        echo "PARTIAL (some objects may already exist)"
        # Still record it to prevent re-running
        psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c \
            "INSERT INTO migration_log (version, description) VALUES ('$version', '$filename') ON CONFLICT DO NOTHING;" > /dev/null 2>&1
        return 0
    fi
}

# Get all migration files sorted
for migration in $(ls "$MIGRATIONS_DIR"/*.sql 2>/dev/null | sort); do
    apply_migration "$migration"
done

echo ""
echo "=== Summary ==="

# Show table count
TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc \
    "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public';" 2>/dev/null)
echo "Tables created: $TABLE_COUNT"

# Show migration count
MIGRATION_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc \
    "SELECT COUNT(*) FROM migration_log;" 2>/dev/null)
echo "Migrations applied: $MIGRATION_COUNT"

# Verify critical tables
echo ""
echo "Verifying critical tables..."
CRITICAL_TABLES="tasks workers waivers workspace_registry execution_plans planning_audit_events council_sessions"
MISSING=0
for table in $CRITICAL_TABLES; do
    EXISTS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc \
        "SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public' AND tablename = '$table';" 2>/dev/null)
    if [ "$EXISTS" = "1" ]; then
        echo "  ✅ $table"
    else
        echo "  ❌ $table (MISSING)"
        MISSING=$((MISSING + 1))
    fi
done

if [ $MISSING -gt 0 ]; then
    echo ""
    echo "WARNING: $MISSING critical tables are missing!"
    exit 1
fi

echo ""
echo "✅ Database initialization complete!"
echo ""
echo "To start the API server:"
echo "  export DATABASE_URL=\"postgresql://$DB_USER:$PGPASSWORD@$DB_HOST:$DB_PORT/$DB_NAME\""
echo "  cargo run --bin agent-agency-api-server --features orchestration,testing -- --port 8889"







