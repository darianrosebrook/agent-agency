#!/usr/bin/env bash
# Agent Agency V3 System Startup Script - Real Service Integrations
# Starts all real services: PostgreSQL (Docker), Ollama, CoreML models, and API server
# @darianrosebrook

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"
LOG_DIR="$V3_ROOT/logs"
PID_DIR="$V3_ROOT/pids"
MODELS_DIR="$PROJECT_ROOT/models/coreml"

# Service Configuration
POSTGRES_CONTAINER="agent-agency-v3-postgres"
POSTGRES_PORT="5433"
OLLAMA_PORT="11434"
API_PORT="8080"

declare -a PIDS=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_step() {
    echo -e "${BLUE}🔄 $1${NC}"
}

# Create necessary directories
setup_directories() {
    log_step "Setting up directories..."
    mkdir -p "$LOG_DIR" "$PID_DIR"
    log_success "Directories created"
}

# Check system dependencies
check_dependencies() {
    log_step "Checking system dependencies..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not installed"
        log_info "Install Docker from: https://docs.docker.com/get-docker/"
        exit 1
    fi

    # Check Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        log_info "Start Docker Desktop or run: sudo systemctl start docker"
        exit 1
    fi
    log_success "Docker is available"

    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found"
        log_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi
    log_success "Rust/Cargo is available"

    # Check CoreML models directory
    if [[ ! -d "$MODELS_DIR" ]]; then
        log_error "CoreML models directory not found: $MODELS_DIR"
        log_info "Run model download scripts from scripts/v3/models/"
        exit 1
    fi
    log_success "CoreML models directory exists"
}

# Start PostgreSQL via Docker
start_postgres() {
    log_step "Starting PostgreSQL via Docker..."

    # Stop any existing container
    docker stop "$POSTGRES_CONTAINER" &> /dev/null || true
    docker rm "$POSTGRES_CONTAINER" &> /dev/null || true

    # Start PostgreSQL container
    docker run -d \
        --name "$POSTGRES_CONTAINER" \
        --rm \
        -e POSTGRES_DB=agent_agency \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=agent_agency_secure_password_123 \
        -p "$POSTGRES_PORT:5432" \
        postgres:15-alpine

    if [[ $? -ne 0 ]]; then
        log_error "Failed to start PostgreSQL container"
        exit 1
    fi

    log_info "Waiting for PostgreSQL to be ready..."
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if docker exec "$POSTGRES_CONTAINER" pg_isready -U postgres -d agent_agency &> /dev/null; then
            log_success "PostgreSQL is ready (Port: $POSTGRES_PORT)"
            return 0
        fi

        log_info "PostgreSQL not ready yet (attempt $attempt/$max_attempts)..."
        sleep 2
        ((attempt++))
    done

    log_error "PostgreSQL failed to start within timeout"
    docker logs "$POSTGRES_CONTAINER"
    exit 1
}

# Start Ollama service
start_ollama() {
    log_step "Starting Ollama service..."

    # Check if Ollama is already running
    if curl -s "http://localhost:$OLLAMA_PORT/api/tags" &> /dev/null; then
        log_success "Ollama is already running (Port: $OLLAMA_PORT)"
        return 0
    fi

    # Start Ollama in background
    nohup ollama serve > "$LOG_DIR/ollama.log" 2>&1 &

    local pid=$!
    PIDS+=("$pid")
    echo $pid > "$PID_DIR/ollama.pid"

    log_info "Waiting for Ollama to be ready..."
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -s "http://localhost:$OLLAMA_PORT/api/tags" &> /dev/null; then
            log_success "Ollama is ready (Port: $OLLAMA_PORT)"
            return 0
        fi

        log_info "Ollama not ready yet (attempt $attempt/$max_attempts)..."
        sleep 2
        ((attempt++))
    done

    log_error "Ollama failed to start within timeout"
    cat "$LOG_DIR/ollama.log" || true
    exit 1
}

# Ensure Ollama model is available
ensure_ollama_model() {
    local model_name="${1:-gemma3n:e2b}"

    log_step "Ensuring Ollama model '$model_name' is available..."

    # Check if model is already available
    if ollama list | grep -q "$model_name"; then
        log_success "Model '$model_name' is already available"
        return 0
    fi

    log_info "Pulling model '$model_name'..."
    if ollama pull "$model_name"; then
        log_success "Model '$model_name' pulled successfully"
    else
        log_error "Failed to pull model '$model_name'"
        exit 1
    fi
}

# Start API server
start_api_server() {
    log_step "Starting API server with real integrations..."

    cd "$V3_ROOT"

    # Build if needed
    if [[ ! -f "target/debug/agent-agency-api-server" ]]; then
        log_info "Building API server..."
        cargo build --package data-interfaces-adapters --bin agent-agency-api-server
    fi

    # Set environment variables for real integrations
    export DATABASE_URL="postgresql://postgres:agent_agency_secure_password_123@localhost:$POSTGRES_PORT/agent_agency"
    export COREML_MODELS_PATH="$MODELS_DIR"
    export OLLAMA_BASE_URL="http://localhost:$OLLAMA_PORT"
    export RUST_LOG="${RUST_LOG:-info}"
    export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

    # Start API server
    nohup cargo run --package data-interfaces-adapters --bin agent-agency-api-server \
        -- --host 127.0.0.1 --port $API_PORT --enable-cors \
        > "$LOG_DIR/api-server.log" 2>&1 &

    local pid=$!
    PIDS+=("$pid")
    echo $pid > "$PID_DIR/api-server.pid"

    # Wait for service to be ready
    log_info "Waiting for API server to be ready..."
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -s "http://localhost:$API_PORT/health" &> /dev/null; then
            log_success "API server is ready (Port: $API_PORT)"
            return 0
        fi

        log_info "API server not ready yet (attempt $attempt/$max_attempts)..."
        sleep 2
        ((attempt++))
    done

    log_error "API server failed to start within timeout"
    cat "$LOG_DIR/api-server.log" || true
    exit 1
}

# Perform comprehensive health checks
health_check() {
    log_step "Performing comprehensive health checks..."

    local all_healthy=true

    # Check PostgreSQL
    if docker exec "$POSTGRES_CONTAINER" pg_isready -U postgres -d agent_agency &> /dev/null; then
        log_success "PostgreSQL: Healthy"
    else
        log_error "PostgreSQL: Unhealthy"
        all_healthy=false
    fi

    # Check Ollama
    if curl -s "http://localhost:$OLLAMA_PORT/api/tags" &> /dev/null; then
        log_success "Ollama: Healthy"
    else
        log_error "Ollama: Unhealthy"
        all_healthy=false
    fi

    # Check API server
    if curl -s "http://localhost:$API_PORT/health" &> /dev/null; then
        log_success "API Server: Healthy"
    else
        log_error "API Server: Unhealthy"
        all_healthy=false
    fi

    # Check CoreML models
    if [[ -d "$MODELS_DIR/mistral" ]] && [[ -d "$MODELS_DIR/fastvit" ]]; then
        log_success "CoreML Models: Available"
    else
        log_error "CoreML Models: Missing"
        all_healthy=false
    fi

    if [[ "$all_healthy" == "true" ]]; then
        log_success "All services are healthy! 🎉"
        return 0
    else
        log_error "Some services are unhealthy"
        return 1
    fi
}

# Stop all services
stop_services() {
    log_step "Stopping all services..."

    # Stop API server
    if [[ -f "$PID_DIR/api-server.pid" ]]; then
        local pid=$(cat "$PID_DIR/api-server.pid")
        if kill -0 $pid 2>/dev/null; then
            log_info "Stopping API server (PID: $pid)..."
            kill $pid || true
            sleep 2
            if kill -0 $pid 2>/dev/null; then
                kill -9 $pid || true
            fi
        fi
        rm -f "$PID_DIR/api-server.pid"
    fi

    # Stop Ollama
    if [[ -f "$PID_DIR/ollama.pid" ]]; then
        local pid=$(cat "$PID_DIR/ollama.pid")
        if kill -0 $pid 2>/dev/null; then
            log_info "Stopping Ollama (PID: $pid)..."
            kill $pid || true
            sleep 2
            if kill -0 $pid 2>/dev/null; then
                kill -9 $pid || true
            fi
        fi
        rm -f "$PID_DIR/ollama.pid"
    fi

    # Stop PostgreSQL container
    if docker ps | grep -q "$POSTGRES_CONTAINER"; then
        log_info "Stopping PostgreSQL container..."
        docker stop "$POSTGRES_CONTAINER" || true
    fi

    # Clean up any remaining processes
    pkill -f "data-infrastructure" 2>/dev/null || true
    pkill -f "ollama" 2>/dev/null || true

    log_success "All services stopped"
}

# Show service status
show_status() {
    log_step "Service Status:"

    # PostgreSQL
    if docker ps | grep -q "$POSTGRES_CONTAINER"; then
        log_success "PostgreSQL: Running (Container: $POSTGRES_CONTAINER, Port: $POSTGRES_PORT)"
    else
        log_warning "PostgreSQL: Not running"
    fi

    # Ollama
    if [[ -f "$PID_DIR/ollama.pid" ]]; then
        local pid=$(cat "$PID_DIR/ollama.pid")
        if kill -0 $pid 2>/dev/null; then
            log_success "Ollama: Running (PID: $pid, Port: $OLLAMA_PORT)"
        else
            log_warning "Ollama: PID file exists but process not running"
            rm -f "$PID_DIR/ollama.pid"
        fi
    else
        log_warning "Ollama: Not running"
    fi

    # API Server
    if [[ -f "$PID_DIR/api-server.pid" ]]; then
        local pid=$(cat "$PID_DIR/api-server.pid")
        if kill -0 $pid 2>/dev/null; then
            log_success "API Server: Running (PID: $pid, Port: $API_PORT)"
        else
            log_warning "API Server: PID file exists but process not running"
            rm -f "$PID_DIR/api-server.pid"
        fi
    else
        log_warning "API Server: Not running"
    fi

    # CoreML Models
    if [[ -d "$MODELS_DIR" ]]; then
        local model_count=$(find "$MODELS_DIR" -name "*.mlmodelc" | wc -l)
        log_success "CoreML Models: Available ($model_count compiled models)"
    else
        log_warning "CoreML Models: Directory not found"
    fi
}

# Show logs
show_logs() {
    local service="${1:-all}"

    if [[ "$service" == "all" ]]; then
        log_step "Recent logs from all services:"
        echo -e "\n${BLUE}=== API Server ===${NC}"
        tail -n 10 "$LOG_DIR/api-server.log" 2>/dev/null || echo "No API server logs"
        echo -e "\n${BLUE}=== Ollama ===${NC}"
        tail -n 10 "$LOG_DIR/ollama.log" 2>/dev/null || echo "No Ollama logs"
    elif [[ "$service" == "api" ]] || [[ "$service" == "api-server" ]]; then
        log_step "API Server logs:"
        tail -f "$LOG_DIR/api-server.log" 2>/dev/null || echo "No API server logs"
    elif [[ "$service" == "ollama" ]]; then
        log_step "Ollama logs:"
        tail -f "$LOG_DIR/ollama.log" 2>/dev/null || echo "No Ollama logs"
    else
        log_error "Unknown service: $service"
        echo "Available services: api-server, ollama"
    fi
}

# Main start function
start_all() {
    log_info "🚀 Starting Agent Agency V3 System with Real Service Integrations"
    log_info "=========================================================="

    setup_directories
    check_dependencies

    start_postgres
    start_ollama
    ensure_ollama_model "gemma3n:e2b"
    start_api_server

    sleep 2
    if health_check; then
        log_success "🎉 V3 System started successfully!"
        log_info ""
        log_info "Services available at:"
        log_info "  📊 API Server: http://localhost:$API_PORT"
        log_info "  🤖 Ollama: http://localhost:$OLLAMA_PORT"
        log_info "  🐘 PostgreSQL: localhost:$POSTGRES_PORT"
        log_info "  🧠 CoreML Models: $MODELS_DIR"
        log_info ""
        log_info "Management commands:"
        log_info "  📊 Status: $0 status"
        log_info "  📝 Logs: $0 logs [service]"
        log_info "  🛑 Stop: $0 stop"
        log_info "  🔄 Restart: $0 restart"
    else
        log_error "❌ System startup failed - some services are unhealthy"
        exit 1
    fi
}

# Signal handlers for graceful shutdown
cleanup() {
    log_info "Received shutdown signal, stopping services..."
    stop_services
    exit 0
}

trap cleanup SIGINT SIGTERM

# Main script logic
case "${1:-start}" in
    "start")
        start_all
        ;;
    "stop")
        stop_services
        ;;
    "restart")
        stop_services
        sleep 3
        start_all
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs "${2:-all}"
        ;;
    "health")
        health_check
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|health}"
        echo ""
        echo "Commands:"
        echo "  start   - Start all V3 services with real integrations"
        echo "  stop    - Stop all V3 services"
        echo "  restart - Restart all V3 services"
        echo "  status  - Show service status"
        echo "  logs    - Show logs (optionally specify service: api-server, ollama)"
        echo "  health  - Perform comprehensive health checks"
        echo ""
        echo "Services Started:"
        echo "  🐘 PostgreSQL - Docker container on port $POSTGRES_PORT"
        echo "  🤖 Ollama - Local AI models on port $OLLAMA_PORT"
        echo "  📊 API Server - Real integrations on port $API_PORT"
        echo "  🧠 CoreML Models - Hardware acceleration ready"
        exit 1
        ;;
esac
