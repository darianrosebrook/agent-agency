#!/usr/bin/env bash
# Agent Agency V3 Startup Script
# Starts PostgreSQL, API server, and Dashboard with non-standard ports
# Ports chosen to avoid conflicts with other local development projects
# @darianrosebrook

set -euo pipefail

# Configuration - Non-standard ports to avoid conflicts
POSTGRES_PORT="5433"      # Standard is 5432
API_PORT="18080"          # Standard is 8080
DASHBOARD_PORT="13000"    # Standard is 3000

# Project paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"
DASHBOARD_ROOT="$PROJECT_ROOT/apps/v3-agent-agency-dashboard"
LOG_DIR="$V3_ROOT/logs"
PID_DIR="$V3_ROOT/pids"

# Service names
POSTGRES_CONTAINER="agent-agency-v3-postgres"
REDIS_CONTAINER="agent-agency-v3-redis"
REDIS_PORT="6379"
API_PID_FILE="$PID_DIR/api-server.pid"
DASHBOARD_PID_FILE="$PID_DIR/dashboard.pid"

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

# Check if port is in use
is_port_in_use() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Check if process is running by PID file
is_process_running() {
    local pid_file=$1
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file" 2>/dev/null || echo "")
        if [[ -n "$pid" ]] && ps -p "$pid" > /dev/null 2>&1; then
            return 0  # Process is running
        else
            # Clean up stale PID file
            rm -f "$pid_file"
            return 1  # Process is not running
        fi
    fi
    return 1  # PID file doesn't exist
}

# Start PostgreSQL (Docker container)
start_postgres() {
    log_step "Checking PostgreSQL..."

    # Check if container is already running
    if docker ps --format '{{.Names}}' | grep -q "^${POSTGRES_CONTAINER}$"; then
        log_success "PostgreSQL container already running (Port: $POSTGRES_PORT)"
        return 0
    fi

    # Check if port is in use by something else
    if is_port_in_use "$POSTGRES_PORT"; then
        log_warning "Port $POSTGRES_PORT is already in use"
        if docker ps -a --format '{{.Names}}' | grep -q "^${POSTGRES_CONTAINER}$"; then
            log_info "Starting existing PostgreSQL container..."
            docker start "$POSTGRES_CONTAINER" > /dev/null 2>&1
            sleep 2
            log_success "PostgreSQL container started (Port: $POSTGRES_PORT)"
            return 0
        else
            log_error "Port $POSTGRES_PORT is in use by another process"
            return 1
        fi
    fi

    log_step "Starting PostgreSQL container..."
    mkdir -p "$LOG_DIR" "$PID_DIR"

    docker run -d \
        --name "$POSTGRES_CONTAINER" \
        -e POSTGRES_DB=agent_agency \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=agent_agency_secure_password_123 \
        -p "$POSTGRES_PORT:5432" \
        -v "${V3_ROOT}/data/postgres:/var/lib/postgresql/data" \
        postgres:15-alpine \
        > "$LOG_DIR/postgres-start.log" 2>&1 || {
        log_error "Failed to start PostgreSQL container"
        return 1
    }

    # Wait for PostgreSQL to be ready
    log_info "Waiting for PostgreSQL to be ready..."
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if docker exec "$POSTGRES_CONTAINER" pg_isready -U postgres > /dev/null 2>&1; then
            log_success "PostgreSQL is ready (Port: $POSTGRES_PORT)"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    log_error "PostgreSQL failed to start within timeout"
    return 1
}

# Start Redis (Docker container)
start_redis() {
    log_step "Checking Redis..."

    # Check if container is already running
    if docker ps --format '{{.Names}}' | grep -q "^${REDIS_CONTAINER}$"; then
        log_success "Redis container already running (Port: $REDIS_PORT)"
        return 0
    fi

    # Check if port is in use by something else
    if is_port_in_use "$REDIS_PORT"; then
        log_warning "Port $REDIS_PORT is already in use"
        if docker ps -a --format '{{.Names}}' | grep -q "^${REDIS_CONTAINER}$"; then
            log_info "Starting existing Redis container..."
            docker start "$REDIS_CONTAINER" > /dev/null 2>&1
            sleep 2
            log_success "Redis container started (Port: $REDIS_PORT)"
            return 0
        else
            log_warning "Port $REDIS_PORT is in use by another process - Redis may already be running"
            log_info "Continuing without starting Redis container..."
            return 0
        fi
    fi

    log_step "Starting Redis container..."
    mkdir -p "$LOG_DIR" "$PID_DIR"

    docker run -d \
        --name "$REDIS_CONTAINER" \
        -p "$REDIS_PORT:6379" \
        redis:7-alpine \
        redis-server --appendonly yes \
        > "$LOG_DIR/redis-start.log" 2>&1 || {
        log_error "Failed to start Redis container"
        return 1
    }

    # Wait for Redis to be ready
    log_info "Waiting for Redis to be ready..."
    local max_attempts=15
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if docker exec "$REDIS_CONTAINER" redis-cli ping > /dev/null 2>&1; then
            log_success "Redis is ready (Port: $REDIS_PORT)"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    log_error "Redis failed to start within timeout"
    return 1
}

# Start API Server
start_api_server() {
    log_step "Checking API Server..."

    # Check if already running
    if is_process_running "$API_PID_FILE"; then
        local pid=$(cat "$API_PID_FILE")
        log_success "API Server already running (PID: $pid, Port: $API_PORT)"
        return 0
    fi

    # Check if port is in use
    if is_port_in_use "$API_PORT"; then
        log_warning "Port $API_PORT is already in use - API server may already be running"
        return 0
    fi

    log_step "Starting API Server..."

    cd "$V3_ROOT"

    # Build if needed
    if [[ ! -f "target/debug/data-infrastructure" ]] && [[ ! -f "target/release/data-infrastructure" ]]; then
        log_info "Building API server..."
        cargo build --package data-infrastructure --bin data-infrastructure 2>&1 | tee "$LOG_DIR/api-build.log" || {
            log_error "Failed to build API server"
            return 1
        }
    fi

    # Set environment variables
    export DATABASE_URL="postgresql://postgres:agent_agency_secure_password_123@localhost:$POSTGRES_PORT/agent_agency"
    export REDIS_URL="redis://localhost:$REDIS_PORT"
    export RUST_LOG="${RUST_LOG:-info}"
    export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

    # Start API server
    nohup cargo run --package data-infrastructure --bin data-infrastructure \
        -- --host 127.0.0.1 --port $API_PORT --enable-cors \
        > "$LOG_DIR/api-server.log" 2>&1 &

    local pid=$!
    echo $pid > "$API_PID_FILE"

    # Wait for API server to be ready
    log_info "Waiting for API Server to be ready..."
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -s "http://localhost:$API_PORT/health" > /dev/null 2>&1; then
            log_success "API Server is ready (PID: $pid, Port: $API_PORT)"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    log_error "API Server failed to start within timeout"
    cat "$LOG_DIR/api-server.log" | tail -20 || true
    return 1
}

# Start Dashboard
start_dashboard() {
    log_step "Checking Dashboard..."

    # Check if already running
    if is_process_running "$DASHBOARD_PID_FILE"; then
        local pid=$(cat "$DASHBOARD_PID_FILE")
        log_success "Dashboard already running (PID: $pid, Port: $DASHBOARD_PORT)"
        return 0
    fi

    # Check if port is in use
    if is_port_in_use "$DASHBOARD_PORT"; then
        log_warning "Port $DASHBOARD_PORT is already in use - Dashboard may already be running"
        return 0
    fi

    log_step "Starting Dashboard..."

    cd "$DASHBOARD_ROOT"

    # Check if node_modules exists
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing dashboard dependencies..."
        npm install > "$LOG_DIR/dashboard-install.log" 2>&1 || {
            log_error "Failed to install dashboard dependencies"
            return 1
        }
    fi

    # Create .env.local if it doesn't exist
    if [[ ! -f ".env.local" ]]; then
        log_info "Creating .env.local configuration..."
        cat > .env.local <<EOF
# V3 Agent Agency Dashboard Environment Configuration
NEXT_PUBLIC_API_URL=http://localhost:$API_PORT
API_ADMIN_USERNAME=admin
API_ADMIN_PASSWORD=
EOF
    else
        # Update API URL if needed
        if ! grep -q "NEXT_PUBLIC_API_URL=http://localhost:$API_PORT" .env.local 2>/dev/null; then
            log_info "Updating API URL in .env.local..."
            sed -i.bak "s|NEXT_PUBLIC_API_URL=.*|NEXT_PUBLIC_API_URL=http://localhost:$API_PORT|" .env.local
            rm -f .env.local.bak
        fi
    fi

    # Start dashboard
    PORT=$DASHBOARD_PORT nohup npm run dev \
        > "$LOG_DIR/dashboard.log" 2>&1 &

    local pid=$!
    echo $pid > "$DASHBOARD_PID_FILE"

    # Wait for dashboard to be ready
    log_info "Waiting for Dashboard to be ready..."
    local max_attempts=60
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -s "http://localhost:$DASHBOARD_PORT" > /dev/null 2>&1; then
            log_success "Dashboard is ready (PID: $pid, Port: $DASHBOARD_PORT)"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    log_error "Dashboard failed to start within timeout"
    cat "$LOG_DIR/dashboard.log" | tail -20 || true
    return 1
}

# Health check all services
health_check() {
    log_step "Performing health checks..."

    local all_healthy=true

    # Check PostgreSQL
    if docker exec "$POSTGRES_CONTAINER" pg_isready -U postgres > /dev/null 2>&1; then
        log_success "PostgreSQL: healthy"
    else
        log_error "PostgreSQL: unhealthy"
        all_healthy=false
    fi

    # Check Redis
    if docker exec "$REDIS_CONTAINER" redis-cli ping > /dev/null 2>&1; then
        log_success "Redis: healthy"
    else
        log_error "Redis: unhealthy"
        all_healthy=false
    fi

    # Check API Server
    if curl -s "http://localhost:$API_PORT/health" > /dev/null 2>&1; then
        log_success "API Server: healthy"
    else
        log_error "API Server: unhealthy"
        all_healthy=false
    fi

    # Check Dashboard
    if curl -s "http://localhost:$DASHBOARD_PORT" > /dev/null 2>&1; then
        log_success "Dashboard: healthy"
    else
        log_error "Dashboard: unhealthy"
        all_healthy=false
    fi

    return $([ "$all_healthy" = true ] && echo 0 || echo 1)
}

# Main function
main() {
    log_info "🚀 Starting Agent Agency V3 System"
    log_info "=================================="
    log_info ""
    log_info "Port Configuration:"
    log_info "  🐘 PostgreSQL: $POSTGRES_PORT"
    log_info "  🔴 Redis:      $REDIS_PORT"
    log_info "  🔌 API Server: $API_PORT"
    log_info "  📊 Dashboard:  $DASHBOARD_PORT"
    log_info ""

    # Create necessary directories
    mkdir -p "$LOG_DIR" "$PID_DIR"

    # Start services
    start_postgres || exit 1
    start_redis || exit 1
    start_api_server || exit 1
    start_dashboard || exit 1

    # Wait a moment for everything to settle
    sleep 2

    # Health check
    if health_check; then
        log_success ""
        log_success "🎉 V3 System started successfully!"
        log_info ""
        log_info "Services available at:"
        log_info "  🐘 PostgreSQL: localhost:$POSTGRES_PORT"
        log_info "  🔴 Redis:      localhost:$REDIS_PORT"
        log_info "  🔌 API Server: http://localhost:$API_PORT"
        log_info "  📊 Dashboard:  http://localhost:$DASHBOARD_PORT"
        log_info ""
        log_info "Management commands:"
        log_info "  📊 Status: $0 status"
        log_info "  📝 Logs:   $0 logs [service]"
        log_info "  🛑 Stop:   $0 stop"
    else
        log_error ""
        log_error "❌ System startup completed but some services are unhealthy"
        log_info "Check logs: $LOG_DIR/"
        exit 1
    fi
}

# Status command
status() {
    log_info "Agent Agency V3 System Status"
    log_info "============================"
    echo ""

    # PostgreSQL
    if docker ps --format '{{.Names}}' | grep -q "^${POSTGRES_CONTAINER}$"; then
        log_success "PostgreSQL: Running (Port: $POSTGRES_PORT)"
    else
        log_error "PostgreSQL: Not running"
    fi

    # Redis
    if docker ps --format '{{.Names}}' | grep -q "^${REDIS_CONTAINER}$"; then
        log_success "Redis: Running (Port: $REDIS_PORT)"
    else
        log_error "Redis: Not running"
    fi

    # API Server
    if is_process_running "$API_PID_FILE"; then
        local pid=$(cat "$API_PID_FILE")
        log_success "API Server: Running (PID: $pid, Port: $API_PORT)"
    else
        log_error "API Server: Not running"
    fi

    # Dashboard
    if is_process_running "$DASHBOARD_PID_FILE"; then
        local pid=$(cat "$DASHBOARD_PID_FILE")
        log_success "Dashboard: Running (PID: $pid, Port: $DASHBOARD_PORT)"
    else
        log_error "Dashboard: Not running"
    fi
}

# Stop command
stop() {
    log_info "Stopping Agent Agency V3 System..."
    echo ""

    # Stop Dashboard
    if is_process_running "$DASHBOARD_PID_FILE"; then
        local pid=$(cat "$DASHBOARD_PID_FILE")
        log_step "Stopping Dashboard (PID: $pid)..."
        kill "$pid" 2>/dev/null || true
        rm -f "$DASHBOARD_PID_FILE"
        log_success "Dashboard stopped"
    fi

    # Stop API Server
    if is_process_running "$API_PID_FILE"; then
        local pid=$(cat "$API_PID_FILE")
        log_step "Stopping API Server (PID: $pid)..."
        kill "$pid" 2>/dev/null || true
        rm -f "$API_PID_FILE"
        log_success "API Server stopped"
    fi

    # Stop Redis (optional - comment out if you want to keep it running)
    if docker ps --format '{{.Names}}' | grep -q "^${REDIS_CONTAINER}$"; then
        log_step "Stopping Redis container..."
        docker stop "$REDIS_CONTAINER" > /dev/null 2>&1 || true
        log_success "Redis stopped"
    fi

    # Stop PostgreSQL (optional - comment out if you want to keep it running)
    if docker ps --format '{{.Names}}' | grep -q "^${POSTGRES_CONTAINER}$"; then
        log_step "Stopping PostgreSQL container..."
        docker stop "$POSTGRES_CONTAINER" > /dev/null 2>&1 || true
        log_success "PostgreSQL stopped"
    fi

    log_success "All services stopped"
}

# Logs command
logs() {
    local service="${1:-all}"

    case "$service" in
        api|api-server)
            if [[ -f "$LOG_DIR/api-server.log" ]]; then
                tail -f "$LOG_DIR/api-server.log"
            else
                log_error "API Server log not found"
            fi
            ;;
        dashboard)
            if [[ -f "$LOG_DIR/dashboard.log" ]]; then
                tail -f "$LOG_DIR/dashboard.log"
            else
                log_error "Dashboard log not found"
            fi
            ;;
        postgres|postgresql)
            docker logs -f "$POSTGRES_CONTAINER" 2>/dev/null || log_error "PostgreSQL container not found"
            ;;
        redis)
            docker logs -f "$REDIS_CONTAINER" 2>/dev/null || log_error "Redis container not found"
            ;;
        all)
            log_info "Recent logs from all services:"
            echo ""
            echo "=== API Server ==="
            tail -n 20 "$LOG_DIR/api-server.log" 2>/dev/null || echo "No API server logs"
            echo ""
            echo "=== Dashboard ==="
            tail -n 20 "$LOG_DIR/dashboard.log" 2>/dev/null || echo "No dashboard logs"
            echo ""
            echo "=== PostgreSQL ==="
            docker logs --tail 20 "$POSTGRES_CONTAINER" 2>/dev/null || echo "No PostgreSQL logs"
            echo ""
            echo "=== Redis ==="
            docker logs --tail 20 "$REDIS_CONTAINER" 2>/dev/null || echo "No Redis logs"
            ;;
        *)
            log_error "Unknown service: $service"
            echo "Available services: api-server, dashboard, postgres, redis, all"
            exit 1
            ;;
    esac
}

# Parse command line arguments
case "${1:-start}" in
    start)
        main
        ;;
    status)
        status
        ;;
    stop)
        stop
        ;;
    logs)
        logs "${2:-all}"
        ;;
    *)
        echo "Usage: $0 {start|status|stop|logs [service]}"
        echo ""
        echo "Commands:"
        echo "  start          Start all services (default)"
        echo "  status         Show status of all services"
        echo "  stop           Stop all services"
        echo "  logs [service] Show logs (api-server|dashboard|postgres|all)"
        exit 1
        ;;
esac


