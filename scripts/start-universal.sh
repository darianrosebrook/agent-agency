#!/bin/bash
set -euo pipefail

# Agent Agency V3 - Universal Start Script
# Starts MCP, Database, API, and all supporting services
#
# Usage: ./scripts/start-universal.sh [options]
#
# Options:
#   --minimal     Start only essential services (database, redis, api-server)
#   --docker      Use Docker Compose instead of native binaries
#   --no-monitoring  Skip monitoring stack (prometheus, grafana, etc.)
#   --help        Show this help message

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
V3_DIR="$PROJECT_ROOT/iterations/v3"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
MODE="full"
START_METHOD="native"
INCLUDE_MONITORING=true

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Wait for service to be ready
wait_for_service() {
    local name=$1
    local url=$2
    local max_attempts=${3:-30}
    local attempt=1

    log_info "Waiting for $name to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if curl -f --max-time 5 "$url" >/dev/null 2>&1; then
            log_success "$name is ready"
            return 0
        fi
        log_warn "$name not ready yet, retrying in 2s... ($attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done

    log_error "$name failed to start within $max_attempts attempts"
    return 1
}

# Start PostgreSQL database
start_postgres() {
    log_info "Starting PostgreSQL database..."

    if [ "$START_METHOD" = "docker" ]; then
        # Use Docker
        if ! docker ps | grep -q agent-agency-postgres; then
            docker run -d \
                --name agent-agency-postgres \
                -e POSTGRES_DB=agent_agency \
                -e POSTGRES_USER=agent_agency \
                -e POSTGRES_PASSWORD=password \
                -p 5432:5432 \
                postgres:15-alpine
        fi
        # Wait for Docker container to be ready
        sleep 5
        # Test PostgreSQL connectivity
        if docker exec agent-agency-postgres pg_isready -U agent_agency >/dev/null 2>&1; then
            log_success "PostgreSQL started successfully"
        else
            log_error "PostgreSQL health check failed"
            return 1
        fi
    else
        # Use system PostgreSQL
        if command_exists brew && [ "$(uname)" = "Darwin" ]; then
            # macOS with Homebrew
            if ! brew services list | grep postgresql | grep started >/dev/null 2>&1; then
                brew services start postgresql
            fi
        elif command_exists systemctl; then
            # Linux with systemd
            sudo systemctl start postgresql
        else
            log_error "PostgreSQL not found. Please install PostgreSQL or use --docker mode"
            return 1
        fi

        # Wait for system PostgreSQL to be ready
        sleep 3
        if command_exists pg_isready; then
            if pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
                log_success "PostgreSQL started successfully"
            else
                log_error "PostgreSQL health check failed"
                return 1
            fi
        else
            # Fallback - try to connect
            sleep 2
            log_success "PostgreSQL started (could not verify with pg_isready)"
        fi
    fi
}

# Start Redis
start_redis() {
    log_info "Starting Redis..."

    if [ "$START_METHOD" = "docker" ]; then
        if ! docker ps | grep -q agent-agency-redis; then
            docker run -d \
                --name agent-agency-redis \
                -p 6379:6379 \
                redis:7-alpine
        fi
    else
        if command_exists brew && [ "$(uname)" = "Darwin" ]; then
            if ! brew services list | grep redis | grep started >/dev/null 2>&1; then
                brew services start redis
            fi
        elif command_exists systemctl; then
            sudo systemctl start redis-server
        else
            log_error "Redis not found. Please install Redis or use --docker mode"
            return 1
        fi
    fi

    # Test Redis connectivity
    sleep 3
    if command_exists redis-cli; then
        if redis-cli ping | grep -q PONG; then
            log_success "Redis started successfully"
        else
            log_error "Redis ping failed"
            return 1
        fi
    else
        log_success "Redis started (could not verify with redis-cli)"
    fi
}

# Start Ollama (for AI models)
start_ollama() {
    log_info "Starting Ollama..."

    if [ "$START_METHOD" = "docker" ]; then
        if ! docker ps | grep -q agent-agency-ollama; then
            docker run -d \
                --name agent-agency-ollama \
                -p 11434:11434 \
                -v ollama_data:/root/.ollama \
                -e OLLAMA_HOST=0.0.0.0 \
                ollama/ollama:latest
        fi
    else
        if command_exists ollama; then
            # Start Ollama in background if not already running
            if ! pgrep -f "ollama serve" >/dev/null 2>&1; then
                nohup ollama serve >/dev/null 2>&1 &
                sleep 2
            fi
        else
            log_warn "Ollama not found. Skipping AI model service."
            return 0
        fi
    fi

    # Test Ollama
    if curl -f --max-time 5 http://localhost:11434/api/tags >/dev/null 2>&1; then
        log_success "Ollama started successfully"
    else
        log_warn "Ollama started but health check failed (this is normal for first startup)"
    fi
}

# Start MCP integration service
start_mcp() {
    log_info "Starting MCP integration service..."

    cd "$V3_DIR"

    if [ "$START_METHOD" = "docker" ]; then
        # Would need a Dockerfile for MCP service
        log_warn "MCP Docker container not configured yet, skipping"
        return 0
    else
        # Build and start MCP service
        if cargo build -p agent-agency-mcp; then
            cargo run -p agent-agency-mcp &
            MCP_PID=$!
            echo $MCP_PID > /tmp/agent-agency-mcp.pid
            log_success "MCP service started (PID: $MCP_PID)"
        else
            log_error "Failed to build MCP service"
            return 1
        fi
    fi
}

# Start API server
start_api_server() {
    log_info "Starting API server..."

    cd "$V3_DIR"

    if [ "$START_METHOD" = "docker" ]; then
        # Would need Dockerfile.api
        log_warn "API server Docker container not configured yet, falling back to native build"
        START_METHOD="native"
    fi

    if [ "$START_METHOD" = "native" ]; then
        # Build and start API server
        if cargo build -p agent-agency-api-server; then
            cargo run -p agent-agency-api-server &
            API_PID=$!
            echo $API_PID > /tmp/agent-agency-api.pid
            log_success "API server started on port 8080 (PID: $API_PID)"
        else
            log_error "Failed to build API server"
            return 1
        fi
    fi

    # Wait for API server to be ready
    wait_for_service "API server" "http://localhost:8080/health" || return 1
}

# Start monitoring stack
start_monitoring() {
    if [ "$INCLUDE_MONITORING" = false ]; then
        return 0
    fi

    log_info "Starting monitoring stack..."

    if [ "$START_METHOD" = "docker" ]; then
        # Start Prometheus
        if ! docker ps | grep -q agent-agency-prometheus; then
            docker run -d \
                --name agent-agency-prometheus \
                -p 9090:9090 \
                -v "$V3_DIR/monitoring/prometheus.yml:/etc/prometheus/prometheus.yml" \
                -v prometheus_data:/prometheus \
                prom/prometheus:latest \
                --config.file=/etc/prometheus/prometheus.yml \
                --storage.tsdb.path=/prometheus \
                --web.enable-lifecycle
        fi

        # Start Grafana
        if ! docker ps | grep -q agent-agency-grafana; then
            docker run -d \
                --name agent-agency-grafana \
                -p 3000:3000 \
                -e GF_SECURITY_ADMIN_PASSWORD=admin \
                -e GF_USERS_ALLOW_SIGN_UP=false \
                -v grafana_data:/var/lib/grafana \
                grafana/grafana:latest
        fi

        log_success "Monitoring stack started (Prometheus: http://localhost:9090, Grafana: http://localhost:3000)"
    else
        log_warn "Monitoring stack requires Docker. Use --docker flag or install services manually."
    fi
}

# Start all services in Docker Compose mode
start_docker_compose() {
    log_info "Starting services with Docker Compose..."

    cd "$PROJECT_ROOT/deploy/docker-compose"

    # Check which docker compose command to use
    if docker compose version >/dev/null 2>&1; then
        DOCKER_COMPOSE_CMD="docker compose"
    elif docker-compose --version >/dev/null 2>&1; then
        DOCKER_COMPOSE_CMD="docker-compose"
    else
        log_error "Neither 'docker compose' nor 'docker-compose' found"
        return 1
    fi

    if [ "$MODE" = "minimal" ]; then
        # Start only essential services
        $DOCKER_COMPOSE_CMD -f dev.yml up -d postgres redis
        log_success "Minimal Docker services started (PostgreSQL, Redis)"
    else
        # Start full stack
        $DOCKER_COMPOSE_CMD -f dev.yml up -d
        log_success "Full Docker stack started"
        log_info "Services available:"
        log_info "  - Orchestrator: http://localhost:8080"
        log_info "  - Council: http://localhost:8081"
        log_info "  - Ollama: http://localhost:11434"
        log_info "  - Kong Gateway: http://localhost:8000"
        log_info "  - Elasticsearch: http://localhost:9200"
        log_info "  - Prometheus: http://localhost:9090"
        log_info "  - Grafana: http://localhost:3000"
        log_info "  - Jaeger: http://localhost:16686"
    fi
}

# Stop all services
stop_services() {
    log_info "Stopping all services..."

    # Stop native processes
    if [ -f /tmp/agent-agency-api.pid ]; then
        kill "$(cat /tmp/agent-agency-api.pid)" 2>/dev/null || true
        rm /tmp/agent-agency-api.pid
    fi

    if [ -f /tmp/agent-agency-mcp.pid ]; then
        kill "$(cat /tmp/agent-agency-mcp.pid)" 2>/dev/null || true
        rm /tmp/agent-agency-mcp.pid
    fi

    # Stop Docker containers
    docker stop agent-agency-postgres agent-agency-redis agent-agency-ollama agent-agency-prometheus agent-agency-grafana 2>/dev/null || true
    docker rm agent-agency-postgres agent-agency-redis agent-agency-ollama agent-agency-prometheus agent-agency-grafana 2>/dev/null || true

    # Stop Docker Compose
    if [ -f "$PROJECT_ROOT/deploy/docker-compose/dev.yml" ]; then
        cd "$PROJECT_ROOT/deploy/docker-compose"
        # Check which docker compose command to use
        if docker compose version >/dev/null 2>&1; then
            docker compose -f dev.yml down 2>/dev/null || true
        elif docker-compose --version >/dev/null 2>&1; then
            docker-compose -f dev.yml down 2>/dev/null || true
        fi
    fi

    log_success "All services stopped"
}

# Show status of all services
show_status() {
    log_info "Service Status:"

    # Check native services
    if [ -f /tmp/agent-agency-api.pid ] && kill -0 "$(cat /tmp/agent-agency-api.pid)" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} API Server (PID: $(cat /tmp/agent-agency-api.pid))"
    else
        echo -e "  ${RED}✗${NC} API Server"
    fi

    if [ -f /tmp/agent-agency-mcp.pid ] && kill -0 "$(cat /tmp/agent-agency-mcp.pid)" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} MCP Service (PID: $(cat /tmp/agent-agency-mcp.pid))"
    else
        echo -e "  ${RED}✗${NC} MCP Service"
    fi

    # Check Docker services
    for service in postgres redis ollama prometheus grafana; do
        if docker ps | grep -q "agent-agency-$service"; then
            echo -e "  ${GREEN}✓${NC} $service (Docker)"
        else
            echo -e "  ${RED}✗${NC} $service (Docker)"
        fi
    done

    # Check ports
    for port in 5432 6379 8080 9090 3000; do
        if check_port $port; then
            echo -e "  ${GREEN}✓${NC} Port $port open"
        else
            echo -e "  ${RED}✗${NC} Port $port closed"
        fi
    done
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --minimal)
            MODE="minimal"
            shift
            ;;
        --docker)
            START_METHOD="docker"
            shift
            ;;
        --no-monitoring)
            INCLUDE_MONITORING=false
            shift
            ;;
        --stop)
            stop_services
            exit 0
            ;;
        --status)
            show_status
            exit 0
            ;;
        --help)
            echo "Agent Agency V3 - Universal Start Script"
            echo ""
            echo "Starts MCP, Database, API, and all supporting services."
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --minimal        Start only essential services (database, redis, api-server)"
            echo "  --docker         Use Docker Compose instead of native binaries"
            echo "  --no-monitoring  Skip monitoring stack (prometheus, grafana, etc.)"
            echo "  --stop           Stop all running services"
            echo "  --status         Show status of all services"
            echo "  --help           Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                          # Start full stack with native binaries"
            echo "  $0 --minimal               # Start only essential services"
            echo "  $0 --docker                # Use Docker Compose"
            echo "  $0 --stop                  # Stop all services"
            echo "  $0 --status                # Check service status"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Agent Agency V3 Universal Start Script"
    log_info "Mode: $MODE, Method: $START_METHOD"

    # Ensure we're in the right directory
    cd "$PROJECT_ROOT"

    if [ "$START_METHOD" = "docker" ]; then
        # Use Docker Compose for everything
        start_docker_compose
    else
        # Start services individually

        # Start infrastructure first
        start_postgres
        start_redis

        if [ "$MODE" = "full" ]; then
            start_ollama
            start_mcp
        fi

        # Start application services
        start_api_server

        if [ "$MODE" = "full" ] && [ "$INCLUDE_MONITORING" = true ]; then
            start_monitoring
        fi
    fi

    log_success "🎉 Agent Agency V3 startup complete!"
    log_info ""
    log_info "Services started:"
    if [ "$START_METHOD" = "docker" ]; then
        log_info "  Check Docker containers with: docker ps"
        if [ "$MODE" = "full" ]; then
            log_info "  Full stack available at various ports (see docker-compose logs)"
        fi
    else
        log_info "  - PostgreSQL: localhost:5432"
        log_info "  - Redis: localhost:6379"
        log_info "  - API Server: http://localhost:8080"
        if [ "$MODE" = "full" ]; then
            log_info "  - Ollama: http://localhost:11434"
            if [ "$INCLUDE_MONITORING" = true ]; then
                log_info "  - Prometheus: http://localhost:9090"
                log_info "  - Grafana: http://localhost:3000"
            fi
        fi
    fi
    log_info ""
    log_info "Use '$0 --status' to check service status"
    log_info "Use '$0 --stop' to stop all services"
}

# Run main function
main
