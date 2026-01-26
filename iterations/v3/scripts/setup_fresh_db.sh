#!/bin/bash
# Setup script for fresh PostgreSQL database
# This script creates the user, database, and enables pgvector
# @author @darianrosebrook
#
# Environment Variables (with defaults):
#   DB_HOST       - Database host (default: 127.0.0.1)
#   DB_PORT       - Database port (default: 5432)
#   DB_USER       - Your local PostgreSQL user for admin operations (default: current user)
#   DB_NAME       - Database name to create (default: agent_agency)
#   DB_PASSWORD   - Password for agent_agency user (default: agent_agency_dev)

set -e

# Add PostgreSQL binaries to PATH
export PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH"

# Admin user for creating database (your local PostgreSQL user)
DB_USER="${DB_USER:-$(whoami)}"
# Standard configuration for the application
DB_NAME="${DB_NAME:-agent_agency}"
DB_PASSWORD="${DB_PASSWORD:-agent_agency_dev}"
DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"

echo "Setting up fresh PostgreSQL database..."
echo "Using PostgreSQL from: $(which psql)"

# Step 1: Create user (connect to postgres database)
echo "1. Creating agent_agency user..."
psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d postgres <<EOF
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'agent_agency') THEN
        CREATE USER agent_agency WITH PASSWORD '${DB_PASSWORD}' SUPERUSER;
        RAISE NOTICE 'User agent_agency created';
    ELSE
        ALTER USER agent_agency WITH PASSWORD '${DB_PASSWORD}' SUPERUSER;
        RAISE NOTICE 'User agent_agency password updated';
    END IF;
END \$\$;
EOF

# Step 2: Create database (if it doesn't exist)
echo "2. Creating agent_agency database..."
createdb -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" "${DB_NAME}" 2>/dev/null || {
    echo "Database may already exist, continuing..."
}

# Step 3: Grant privileges
echo "3. Granting privileges..."
psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d postgres <<EOF
GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO agent_agency;
EOF

# Step 4: Enable pgvector extension (connect to agent_agency database)
echo "4. Enabling pgvector extension..."
psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" <<EOF
CREATE EXTENSION IF NOT EXISTS vector;
SELECT extname, extversion FROM pg_extension WHERE extname = 'vector';
EOF

echo ""
echo "Database setup complete!"
echo ""
echo "Connection string:"
echo "  DATABASE_URL=\"postgresql://agent_agency:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}\""
echo ""
echo "Or using environment variables:"
echo "  export DATABASE_USER=agent_agency"
echo "  export DATABASE_PASSWORD=${DB_PASSWORD}"
echo "  export DATABASE_HOST=${DB_HOST}"
echo "  export DATABASE_PORT=${DB_PORT}"
echo "  export DATABASE_NAME=${DB_NAME}"

