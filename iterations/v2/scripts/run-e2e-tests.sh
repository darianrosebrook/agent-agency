#!/bin/bash

###
# ARBITER-001 End-to-End Test Runner
#
# Starts test database and Redis, runs integration tests, then cleanup.
#
# Usage: ./scripts/run-e2e-tests.sh
#
# @author @darianrosebrook
###

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🧪 ARBITER-001 End-to-End Test Suite${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed or not in PATH${NC}"
    echo "E2E tests require Docker to run PostgreSQL and Redis"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker daemon is not running${NC}"
    echo "Please start Docker and try again"
    exit 1
fi

echo -e "${YELLOW}📦 Starting test containers...${NC}"

# Start PostgreSQL test container
echo "Starting PostgreSQL..."
docker run -d \
  --name agent-agency-test-db \
  -e POSTGRES_PASSWORD=test123 \
  -e POSTGRES_DB=agent_agency_test \
  -p 5432:5432 \
  postgres:16-alpine \
  > /dev/null 2>&1 || {
    echo -e "${YELLOW}Container already exists, restarting...${NC}"
    docker restart agent-agency-test-db > /dev/null
  }

# Start Redis test container
echo "Starting Redis..."
docker run -d \
  --name agent-agency-test-redis \
  -p 6379:6379 \
  redis:7-alpine \
  > /dev/null 2>&1 || {
    echo -e "${YELLOW}Container already exists, restarting...${NC}"
    docker restart agent-agency-test-redis > /dev/null
  }

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
  if docker exec agent-agency-test-db pg_isready -U postgres > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PostgreSQL is ready${NC}"
    break
  fi
  if [ $i -eq 30 ]; then
    echo -e "${RED}❌ PostgreSQL failed to start${NC}"
    docker logs agent-agency-test-db
    exit 1
  fi
  sleep 1
done

# Wait for Redis to be ready
echo "Waiting for Redis to be ready..."
for i in {1..10}; do
  if docker exec agent-agency-test-redis redis-cli ping > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Redis is ready${NC}"
    break
  fi
  if [ $i -eq 10 ]; then
    echo -e "${RED}❌ Redis failed to start${NC}"
    docker logs agent-agency-test-redis
    exit 1
  fi
  sleep 1
done

echo ""
echo -e "${GREEN}✅ Test infrastructure ready${NC}"
echo ""

# Set test environment variables
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=agent_agency_test
export TEST_DB_USER=postgres
export TEST_DB_PASSWORD=test123
export TEST_REDIS_URL=redis://localhost:6379

# Run tests
echo -e "${YELLOW}🧪 Running E2E tests...${NC}"
echo ""

# Run Jest with E2E test pattern
npm run test -- tests/integration/e2e --runInBand --detectOpenHandles --forceExit

TEST_EXIT_CODE=$?

echo ""
echo -e "${YELLOW}🧹 Cleaning up test containers...${NC}"

# Cleanup function
cleanup() {
  docker stop agent-agency-test-db agent-agency-test-redis > /dev/null 2>&1 || true
  docker rm agent-agency-test-db agent-agency-test-redis > /dev/null 2>&1 || true
  echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Register cleanup on script exit
trap cleanup EXIT

# Exit with test result code
if [ $TEST_EXIT_CODE -eq 0 ]; then
  echo ""
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}✅ All E2E tests passed!${NC}"
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  exit 0
else
  echo ""
  echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${RED}❌ E2E tests failed${NC}"
  echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  exit $TEST_EXIT_CODE
fi

