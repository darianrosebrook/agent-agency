#!/bin/bash
# E2E Autonomous Tests Runner
#
# This script sets up the environment and runs comprehensive end-to-end
# tests for Agent Agency V3's autonomous capabilities.
#
# Prerequisites:
# - Docker (for PostgreSQL)
# - Ollama (for local model inference)
# - CoreML Mistral model (for orchestrator)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCKER_COMPOSE_FILE="$SCRIPT_DIR/docker-compose.test.yml"
OLLAMA_MODEL="llama2:7b"  # Can be overridden via OLLAMA_MODEL env var
MISTRAL_MODEL_PATH="$PROJECT_ROOT/models/mistral"

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

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    # Check Ollama
    if ! command -v ollama &> /dev/null; then
        log_error "Ollama is not installed or not in PATH"
        exit 1
    fi

    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed or not in PATH"
        exit 1
    fi

    # Check Mistral model exists
    if [[ ! -d "$MISTRAL_MODEL_PATH" ]]; then
        log_error "Mistral model not found at $MISTRAL_MODEL_PATH"
        log_error "Please ensure the CoreML Mistral model is available"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# Setup test environment
setup_environment() {
    log_info "Setting up test environment..."

    # Start Docker services
    log_info "Starting Docker services..."
    if [[ -f "$DOCKER_COMPOSE_FILE" ]]; then
        docker-compose -f "$DOCKER_COMPOSE_FILE" up -d
        if [[ $? -ne 0 ]]; then
            log_error "Failed to start Docker services"
            exit 1
        fi
    else
        log_error "Docker Compose file not found: $DOCKER_COMPOSE_FILE"
        exit 1
    fi

    # Wait for services to be healthy
    log_info "Waiting for services to be ready..."
    wait_for_services

    # Ensure Ollama model is available
    log_info "Ensuring Ollama model $OLLAMA_MODEL is available..."
    if ! ollama list | grep -q "$OLLAMA_MODEL"; then
        log_info "Pulling Ollama model $OLLAMA_MODEL..."
        ollama pull "$OLLAMA_MODEL"
        if [[ $? -ne 0 ]]; then
            log_error "Failed to pull Ollama model $OLLAMA_MODEL"
            exit 1
        fi
    else
        log_info "Ollama model $OLLAMA_MODEL already available"
    fi

    log_success "Test environment setup complete"
}

# Wait for services to be healthy
wait_for_services() {
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        log_info "Checking service health (attempt $attempt/$max_attempts)..."

        # Check PostgreSQL
        if docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T postgres pg_isready -U test_user -d test_db &> /dev/null; then
            log_info "PostgreSQL is ready"
            postgresql_ready=true
        else
            log_info "PostgreSQL not ready yet"
            postgresql_ready=false
        fi

        # Check Ollama API
        if curl -s http://localhost:11434/api/tags &> /dev/null; then
            log_info "Ollama API is ready"
            ollama_ready=true
        else
            log_info "Ollama API not ready yet"
            ollama_ready=false
        fi

        # Check if all services are ready
        if [[ "$postgresql_ready" == "true" && "$ollama_ready" == "true" ]]; then
            log_success "All services are ready"
            return 0
        fi

        attempt=$((attempt + 1))
        sleep 2
    done

    log_error "Services failed to become ready within timeout"
    exit 1
}

# Run the E2E tests
run_tests() {
    log_info "Running E2E autonomous tests..."

    # Change to project root
    cd "$PROJECT_ROOT"

    # Set test environment variables
    export E2E_TEST_MODE=true
    export OLLAMA_MODEL=${OLLAMA_MODEL}
    export MISTRAL_MODEL_PATH=${MISTRAL_MODEL_PATH}
    export DATABASE_URL="postgresql://test_user:test_password@localhost:5433/test_db"

    # Run the tests with e2e feature flag
    log_info "Executing test scenarios..."
    if cargo test --package testing-validation --features e2e -- --nocapture; then
        log_success "E2E tests completed successfully"
        return 0
    else
        log_error "E2E tests failed"
        return 1
    fi
}

# Cleanup environment
cleanup_environment() {
    log_info "Cleaning up test environment..."

    # Stop Docker services
    if [[ -f "$DOCKER_COMPOSE_FILE" ]]; then
        docker-compose -f "$DOCKER_COMPOSE_FILE" down
    fi

    # Clean up any temporary files
    # (Add cleanup logic as needed)

    log_success "Test environment cleanup complete"
}

# Main execution
main() {
    log_info "Starting E2E Autonomous Tests for Agent Agency V3"
    log_info "================================================="

    # Trap to ensure cleanup on exit
    trap cleanup_environment EXIT

    # Run test phases
    check_prerequisites
    setup_environment

    if run_tests; then
        log_success "All E2E tests passed!"
        exit 0
    else
        log_error "E2E tests failed!"
        exit 1
    fi
}

# Run main function
main "$@"
