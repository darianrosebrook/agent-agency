#!/bin/bash
# E2E Autonomous Tests Runner - Real Service Integrations
#
# This script uses the comprehensive start script to launch all real services
# and runs end-to-end tests for Agent Agency V3's autonomous capabilities.
#
# Prerequisites:
# - Docker (for PostgreSQL container)
# - Ollama (for local model inference)
# - CoreML models in /models/coreml directory

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
START_SCRIPT="$PROJECT_ROOT/scripts/v3/start-v3-system.sh"
OLLAMA_MODEL="${OLLAMA_MODEL:-gemma3n:e2b}"  # Can be overridden via OLLAMA_MODEL env var
MISTRAL_MODEL_PATH="${MISTRAL_MODEL_PATH:-$PROJECT_ROOT/models/coreml/mistral}"  # CoreML Mistral model path

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
    log_info "Setting up test environment with real service integrations..."

    # Use the comprehensive start script to launch all services
    log_info "Starting all V3 services using start script..."
    if [[ ! -f "$START_SCRIPT" ]]; then
        log_error "Start script not found: $START_SCRIPT"
        exit 1
    fi

    # Start all services
    if ! "$START_SCRIPT" start; then
        log_error "Failed to start V3 services"
        exit 1
    fi

    log_success "Test environment setup complete - all real services running"
}

# Services are now started and health-checked by the comprehensive start script
# No need for separate wait_for_services function

# Run the E2E tests
run_tests() {
    log_info "Running E2E autonomous tests..."

    # Change to workspace root (where Cargo.toml with [workspace] is located)
    WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
    cd "$WORKSPACE_ROOT"

    # Set test environment variables
    export E2E_TEST_MODE=true
    export OLLAMA_MODEL=${OLLAMA_MODEL}
    export COREML_MODELS_PATH="$PROJECT_ROOT/models/coreml"
    export DATABASE_URL="postgresql://postgres:agent_agency_secure_password_123@localhost:5433/agent_agency"

    # Run the tests without full feature first to validate basic infrastructure
    # The 'full' feature requires many optional dependencies that may not be fully implemented
    log_info "Executing test scenarios..."
    if cargo test --package testing-validation -- --nocapture; then
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

    # Stop all services using the comprehensive stop script
    if [[ -f "$START_SCRIPT" ]]; then
        "$START_SCRIPT" stop
    fi

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
