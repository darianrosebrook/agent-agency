#!/bin/bash
# Integration Test Runner - Agent Agency V3
# Runs comprehensive integration tests to verify all modules work together

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCKER_COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.test.yml"
TEST_TIMEOUT=${TEST_TIMEOUT:-300}  # 5 minutes default
RUN_CHAOS=${RUN_CHAOS:-false}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    cd "$PROJECT_ROOT"
    docker-compose -f "$DOCKER_COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Main function
main() {
    log_info "🚀 Starting Agent Agency V3 Integration Tests"

    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi

    cd "$PROJECT_ROOT"

    # Stop any existing test containers
    log_info "Stopping any existing test containers..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true

    # Start test infrastructure
    log_info "Starting test infrastructure (PostgreSQL, Redis, etc.)..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d postgres redis smtp-mock

    # Wait for infrastructure to be ready
    log_info "Waiting for infrastructure to be ready..."
    local max_attempts=30
    local attempt=1

    while [ $attempt -le $max_attempts ]; do
        if docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T postgres pg_isready -U test_user -d agent_agency_test >/dev/null 2>&1 && \
           docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T redis redis-cli ping >/dev/null 2>&1; then
            log_success "Infrastructure is ready!"
            break
        fi

        log_info "Waiting for infrastructure... (attempt $attempt/$max_attempts)"
        sleep 2
        attempt=$((attempt + 1))
    done

    if [ $attempt -gt $max_attempts ]; then
        log_error "Infrastructure failed to start within timeout"
        exit 1
    fi

    # Start services
    log_info "Starting all services..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d

    # Wait for all services to be healthy
    log_info "Waiting for all services to be healthy..."
    local services=("orchestration" "health-monitor" "alerting" "learning" "tool-ecosystem" "claim-extraction" "apple-silicon" "worker-1" "worker-2")
    local service_timeout=60

    for service in "${services[@]}"; do
        log_info "Waiting for $service to be healthy..."
        local service_attempts=1
        local max_service_attempts=$((service_timeout / 2))

        while [ $service_attempts -le $max_service_attempts ]; do
            if curl -f -s "http://localhost:$(get_service_port "$service")/health" >/dev/null 2>&1; then
                log_success "$service is healthy"
                break
            fi

            sleep 2
            service_attempts=$((service_attempts + 1))
        done

        if [ $service_attempts -gt $max_service_attempts ]; then
            log_error "$service failed to become healthy within ${service_timeout}s"
            log_info "Checking service logs..."
            docker-compose -f "$DOCKER_COMPOSE_FILE" logs "$service" | tail -20
            exit 1
        fi
    done

    log_success "All services are healthy!"

    # Set environment variables for tests
    export DATABASE_URL="postgresql://test_user:test_password@localhost:5433/agent_agency_test"
    export REDIS_URL="redis://localhost:6380"
    export ORCHESTRATOR_URL="http://localhost:8080"
    export HEALTH_MONITOR_URL="http://localhost:8081"
    export ALERTING_URL="http://localhost:8082"
    export LEARNING_URL="http://localhost:8083"
    export TOOL_ECOSYSTEM_URL="http://localhost:8084"
    export CLAIM_EXTRACTION_URL="http://localhost:8085"
    export APPLE_SILICON_URL="http://localhost:8086"
    export WORKER_1_URL="http://localhost:8081"
    export WORKER_2_URL="http://localhost:8081"

    if [ "$RUN_CHAOS" = "true" ]; then
        export RUN_CHAOS_TESTS=1
        log_warning "Chaos testing enabled - this may cause service instability"
    fi

    # Run integration tests
    log_info "Running integration tests..."
    local test_start=$(date +%s)

    if timeout "$TEST_TIMEOUT" cargo test --test integration -- --nocapture; then
        local test_end=$(date +%s)
        local duration=$((test_end - test_start))
        log_success "🎉 All integration tests PASSED in ${duration}s!"

        # Run chaos tests if requested
        if [ "$RUN_CHAOS" = "true" ]; then
            log_info "Running chaos engineering tests..."
            if timeout "$TEST_TIMEOUT" cargo test --test integration -- --ignored --nocapture; then
                log_success "Chaos tests PASSED!"
            else
                log_error "Chaos tests FAILED!"
                exit 1
            fi
        fi

    else
        log_error "❌ Integration tests FAILED!"
        log_info "Checking service logs for debugging..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" logs | tail -50
        exit 1
    fi

    log_info "Integration tests completed successfully!"
}

# Get service port mapping
get_service_port() {
    local service="$1"
    case "$service" in
        "orchestration") echo "8080" ;;
        "health-monitor") echo "8081" ;;
        "alerting") echo "8082" ;;
        "learning") echo "8083" ;;
        "tool-ecosystem") echo "8084" ;;
        "claim-extraction") echo "8085" ;;
        "apple-silicon") echo "8086" ;;
        "worker-1") echo "8081" ;;
        "worker-2") echo "8081" ;;
        *) echo "8080" ;;
    esac
}

# Show usage
usage() {
    echo "Agent Agency V3 Integration Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --chaos          Enable chaos engineering tests"
    echo "  --timeout SECS   Test timeout in seconds (default: 300)"
    echo "  --help          Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  DATABASE_URL         PostgreSQL connection string"
    echo "  REDIS_URL           Redis connection string"
    echo "  *_URL               Service URLs (auto-configured)"
    echo "  RUN_CHAOS_TESTS     Enable chaos tests (alternative to --chaos)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Run standard integration tests"
    echo "  $0 --chaos                           # Run with chaos engineering"
    echo "  $0 --timeout 600                     # 10 minute timeout"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --chaos)
            RUN_CHAOS=true
            shift
            ;;
        --timeout)
            TEST_TIMEOUT="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Run main function
main "$@"
