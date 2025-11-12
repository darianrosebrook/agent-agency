#!/bin/bash
# Quick test script for schema validation
# Tests against available PostgreSQL databases

set -e

echo "=== Testing Schema Validation ==="
echo ""

# Check for Docker containers
V3_CONTAINER=$(docker ps --filter "name=agent-agency-v3-postgres" --format "{{.Names}}" 2>/dev/null | head -1)
MAIN_CONTAINER=$(docker ps --filter "name=agent-agency-postgres" --format "{{.Names}}" 2>/dev/null | head -1)

if [ -n "$V3_CONTAINER" ]; then
    echo "Found V3 PostgreSQL container: $V3_CONTAINER"
    echo "Getting connection details..."
    
    # Extract port
    PORT=$(docker port "$V3_CONTAINER" 2>/dev/null | grep -oP ':\K[0-9]+' | head -1 || echo "5432")
    
    # Try to get credentials from container
    DB_NAME=$(docker inspect "$V3_CONTAINER" --format '{{index .Config.Env}}' 2>/dev/null | grep -oP 'POSTGRES_DB=\K[^ ]+' || echo "agent_agency")
    DB_USER=$(docker inspect "$V3_CONTAINER" --format '{{index .Config.Env}}' 2>/dev/null | grep -oP 'POSTGRES_USER=\K[^ ]+' || echo "postgres")
    DB_PASSWORD=$(docker inspect "$V3_CONTAINER" --format '{{index .Config.Env}}' 2>/dev/null | grep -oP 'POSTGRES_PASSWORD=\K[^ ]+' || echo "password")
    
    echo "Database: $DB_NAME"
    echo "User: $DB_USER"
    echo "Port: $PORT"
    echo ""
    
    # Construct DATABASE_URL
    DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${PORT}/${DB_NAME}"
    
    echo "Testing connection..."
    if psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
        echo "Connection successful!"
        echo ""
        echo "Running schema validation..."
        echo ""
        
        cd iterations/v3/data-infrastructure
        cargo run --features schema-validation --bin validate_schema -- --database-url "$DATABASE_URL"
    else
        echo "Could not connect to database. Please check:"
        echo "1. Container is running: docker ps | grep postgres"
        echo "2. Port is accessible: docker port $V3_CONTAINER"
        echo "3. Credentials are correct"
        echo ""
        echo "You can also manually set DATABASE_URL:"
        echo "  export DATABASE_URL='postgresql://user:password@localhost:port/dbname'"
        echo "  cargo run --bin validate_schema"
    fi
elif [ -n "$MAIN_CONTAINER" ]; then
    echo "Found main PostgreSQL container: $MAIN_CONTAINER"
    echo "Please run manually with:"
    echo "  docker port $MAIN_CONTAINER"
    echo "  export DATABASE_URL='postgresql://user:password@localhost:port/dbname'"
    echo "  cd iterations/v3/data-infrastructure"
    echo "  cargo run --features schema-validation --bin validate_schema"
else
    echo "No PostgreSQL containers found running."
    echo ""
    echo "To start a test database:"
    echo "  docker-compose -f deploy/docker-compose.yml up -d postgres"
    echo ""
    echo "Or use the test database:"
    echo "  cd iterations/v3/testing-validation"
    echo "  docker-compose -f docker-compose.test.yml up -d postgres"
fi













