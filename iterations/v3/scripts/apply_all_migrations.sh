#!/bin/bash
# Apply all database migrations in order
# This script ensures all migrations are applied correctly from a fresh start
# @author @darianrosebrook

set -e

# Database connection details
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5433}"
DB_USER="${DB_USER:-test_user}"
DB_NAME="${DB_NAME:-agent_agency_test}"
PGPASSWORD="${PGPASSWORD:-test_password}"
export PGPASSWORD

MIGRATIONS_DIR="$(dirname "$0")/../data-infrastructure/migrations"

echo "=== Agent Agency V3 Migration Runner ==="
echo "Database: $DB_NAME on $DB_HOST:$DB_PORT"
echo "Migrations directory: $MIGRATIONS_DIR"
echo ""

# Function to run a migration file
run_migration() {
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
    
    echo "  [APPLY] $filename..."
    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$file" > /tmp/migration_output.txt 2>&1; then
        echo "    ✅ Success"
        return 0
    else
        echo "    ❌ Failed"
        cat /tmp/migration_output.txt | head -20
        return 1
    fi
}

# Ensure migration_log table exists
echo "Creating migration_log table if not exists..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
CREATE TABLE IF NOT EXISTS migration_log (
    version VARCHAR(255) PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);" 2>/dev/null

echo ""
echo "Applying migrations..."

# Apply migrations in order
# Note: We handle the duplicate 018 files by renaming one
for migration in $(ls "$MIGRATIONS_DIR"/*.sql | sort); do
    run_migration "$migration" || {
        echo ""
        echo "Migration failed. Stopping."
        exit 1
    }
done

echo ""
echo "=== Migration Summary ==="
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
SELECT version, description, applied_at 
FROM migration_log 
ORDER BY version;"

echo ""
echo "=== Table Count ==="
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
SELECT COUNT(*) as table_count 
FROM pg_tables 
WHERE schemaname = 'public';"

echo ""
echo "✅ All migrations complete!"







