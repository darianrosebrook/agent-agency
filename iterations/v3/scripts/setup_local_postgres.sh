#!/bin/bash
# Setup script for local PostgreSQL database (non-Docker)
# This script enables pgvector and runs all migrations
# @author @darianrosebrook
#
# Environment Variables (with defaults):
#   DB_HOST     - Database host (default: 127.0.0.1)
#   DB_PORT     - Database port (default: 5432)
#   DB_USER     - Database user (default: agent_agency)
#   DB_PASSWORD - Database password (default: agent_agency_dev)
#   DB_NAME     - Database name (default: agent_agency)

set -e

echo "Setting up local PostgreSQL database..."

# Database connection details - standardized defaults
DB_USER="${DB_USER:-agent_agency}"
DB_PASSWORD="${DB_PASSWORD:-agent_agency_dev}"
DB_NAME="${DB_NAME:-agent_agency}"
DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"

# Construct DATABASE_URL
export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

echo "Using DATABASE_URL: postgresql://${DB_USER}:****@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# Change to v3 directory
cd "$(dirname "$0")/.." || exit 1

# Enable pgvector extension
echo "Enabling pgvector extension..."
psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -c "CREATE EXTENSION IF NOT EXISTS vector;" || {
    echo "Warning: Failed to enable pgvector extension. It may already be enabled or you may need to install it."
    echo "Install pgvector with: brew install pgvector"
}

# Run migrations using Rust migration runner
echo "Running database migrations..."
cargo run --bin run_migrations --manifest-path data-infrastructure/Cargo.toml || {
    echo "Error: Failed to run migrations"
    exit 1
}

echo "Database setup complete!"
echo ""
echo "To use this database, set:"
echo "  export DATABASE_URL=\"${DATABASE_URL}\""





