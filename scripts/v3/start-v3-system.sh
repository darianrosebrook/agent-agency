#!/usr/bin/env bash
# Agent Agency V3 System Startup Script
# Unified startup and shutdown for all v3 services
# @darianrosebrook

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"
LOG_DIR="$V3_ROOT/logs"
PID_DIR="$V3_ROOT/pids"

# Service Configuration (using arrays instead of associative arrays for compatibility)
SERVICES=("api-server" "worker-system" "web-dashboard")
SERVICE_PACKAGES=("data-infrastructure" "agent-workers" "apps/web-dashboard")
SERVICE_PORTS=("8080" "8081" "3000")

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

# Helper function to get service index
get_service_index() {
    local service="$1"
    for i in "${!SERVICES[@]}"; do
        if [[ "${SERVICES[$i]}" == "$service" ]]; then
            echo $i
            return
        fi
    done
    echo -1
}

# Helper function to get service package
get_service_package() {
    local service="$1"
    local index=$(get_service_index "$service")
    if [[ $index -ge 0 ]]; then
        echo "${SERVICE_PACKAGES[$index]}"
    fi
}

# Helper function to get service port
get_service_port() {
    local service="$1"
    local index=$(get_service_index "$service")
    if [[ $index -ge 0 ]]; then
        echo "${SERVICE_PORTS[$index]}"
    fi
}

# Create necessary directories
setup_directories() {
    log_step "Setting up directories..."
    mkdir -p "$LOG_DIR" "$PID_DIR"
    log_success "Directories created"
}

# Check dependencies
check_dependencies() {
    log_step "Checking system dependencies..."
    
    # Check PostgreSQL
    if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
        log_error "PostgreSQL is not running on localhost:5432"
        log_info "Start PostgreSQL with: brew services start postgresql@14"
        exit 1
    fi
    log_success "PostgreSQL is running"
    
    # Check Redis
    if ! redis-cli ping >/dev/null 2>&1; then
        log_error "Redis is not running on localhost:6379"
        log_info "Start Redis with: brew services start redis"
        exit 1
    fi
    log_success "Redis is running"
    
    # Check Rust
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Rust/Cargo not found"
        log_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi
    log_success "Rust/Cargo is available"
    
    # Check Node.js
    if ! command -v node >/dev/null 2>&1; then
        log_error "Node.js not found"
        log_info "Install Node.js from: https://nodejs.org/"
        exit 1
    fi
    log_success "Node.js is available"
}

# Setup environment
setup_environment() {
    log_step "Setting up environment..."
    
    # Set required environment variables
    export DATABASE_PASSWORD="${DATABASE_PASSWORD:-agent_agency_secure_password_123}"
    export DATABASE_URL="postgresql://postgres:${DATABASE_PASSWORD}@localhost:5432/agent_agency"
    export REDIS_URL="redis://localhost:6379"
    export RUST_LOG="${RUST_LOG:-info}"
    export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
    
    # Set build environment
    export BUILD_NAMESPACE="v3-system"
    export AGENT_PLATFORM="aarch64-apple-darwin"
    export AGENT_TYPE="production"
    
    log_success "Environment configured"
}

# Start Rust API Server
start_api_server() {
    log_step "Starting API Server..."
    
    cd "$V3_ROOT"
    
    # Build if needed
    if [[ ! -f "target/debug/data-infrastructure" ]]; then
        log_info "Building API server..."
        cargo build --package data-infrastructure --bin data-infrastructure
    fi
    
    # Start API server
    nohup cargo run --package data-infrastructure --bin data-infrastructure \
        -- --host 127.0.0.1 --port 8080 --enable-cors \
        --db-host localhost --db-port 5432 --db-name agent_agency \
        --db-user postgres --db-password "$DATABASE_PASSWORD" \
        --enable-redis --redis-url "$REDIS_URL" \
        --v3-backend-host "http://localhost:3001" \
        > "$LOG_DIR/api-server.log" 2>&1 &
    
    local pid=$!
    PIDS+=("$pid")
    echo $pid > "$PID_DIR/api-server.pid"
    
    # Wait for service to be ready
    sleep 3
    if kill -0 $pid 2>/dev/null; then
        log_success "API Server started (PID: $pid, Port: 8080)"
    else
        log_error "API Server failed to start"
        return 1
    fi
}

# Start Worker System
start_worker_system() {
    log_step "Starting Worker System..."
    
    cd "$V3_ROOT"
    
    # Build if needed
    if [[ ! -f "target/debug/agent-workers" ]]; then
        log_info "Building worker system..."
        cargo build --package agent-workers --bin agent-workers
    fi
    
    # Start worker system
    nohup cargo run --package agent-workers --bin agent-workers \
        > "$LOG_DIR/worker-system.log" 2>&1 &
    
    local pid=$!
    PIDS+=("$pid")
    echo $pid > "$PID_DIR/worker-system.pid"
    
    # Wait for service to be ready
    sleep 3
    if kill -0 $pid 2>/dev/null; then
        log_success "Worker System started (PID: $pid, Port: 8081)"
    else
        log_error "Worker System failed to start"
        return 1
    fi
}

# Start Web Dashboard
start_web_dashboard() {
    log_step "Starting Web Dashboard..."
    
    cd "$V3_ROOT/apps/web-dashboard"
    
    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing dashboard dependencies..."
        npm install
    fi
    
    # Start Next.js development server
    nohup npm run dev \
        > "$LOG_DIR/web-dashboard.log" 2>&1 &
    
    local pid=$!
    PIDS+=("$pid")
    echo $pid > "$PID_DIR/web-dashboard.pid"
    
    # Wait for service to be ready
    sleep 5
    if kill -0 $pid 2>/dev/null; then
        log_success "Web Dashboard started (PID: $pid, Port: 3000)"
    else
        log_error "Web Dashboard failed to start"
        return 1
    fi
}

# Health check all services
health_check() {
    log_step "Performing health checks..."
    
    # Check API server
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        log_success "API Server health check passed"
    else
        log_warning "API Server health check failed"
    fi
    
    # Check web dashboard
    if curl -s http://localhost:3000 >/dev/null 2>&1; then
        log_success "Web Dashboard health check passed"
    else
        log_warning "Web Dashboard health check failed"
    fi
    
    # Check worker system (if it has health endpoint)
    if curl -s http://localhost:8081/health >/dev/null 2>&1; then
        log_success "Worker System health check passed"
    else
        log_warning "Worker System health check failed (may not have health endpoint)"
    fi
}

# Stop all services
stop_services() {
    log_step "Stopping all services..."
    
    # Stop services by PID file
    for service in "${SERVICES[@]}"; do
        local pid_file="$PID_DIR/$service.pid"
        if [[ -f "$pid_file" ]]; then
            local pid=$(cat "$pid_file")
            if kill -0 $pid 2>/dev/null; then
                log_info "Stopping $service (PID: $pid)..."
                kill $pid
                sleep 2
                if kill -0 $pid 2>/dev/null; then
                    log_warning "Force killing $service..."
                    kill -9 $pid
                fi
                log_success "$service stopped"
            fi
            rm -f "$pid_file"
        fi
    done
    
    # Clean up any remaining processes
    pkill -f "data-infrastructure" 2>/dev/null || true
    pkill -f "agent-workers" 2>/dev/null || true
    pkill -f "next-server" 2>/dev/null || true
    
    log_success "All services stopped"
}

# Show service status
show_status() {
    log_step "Service Status:"
    
    for i in "${!SERVICES[@]}"; do
        local service="${SERVICES[$i]}"
        local port="${SERVICE_PORTS[$i]}"
        local pid_file="$PID_DIR/$service.pid"
        
        if [[ -f "$pid_file" ]]; then
            local pid=$(cat "$pid_file")
            if kill -0 $pid 2>/dev/null; then
                log_success "$service: Running (PID: $pid, Port: $port)"
            else
                log_warning "$service: PID file exists but process not running"
                rm -f "$pid_file"
            fi
        else
            log_warning "$service: Not running"
        fi
    done
}

# Show logs
show_logs() {
    local service="${1:-all}"
    
    if [[ "$service" == "all" ]]; then
        log_step "Recent logs from all services:"
        for log_file in "$LOG_DIR"/*.log; do
            if [[ -f "$log_file" ]]; then
                echo -e "\n${BLUE}=== $(basename "$log_file") ===${NC}"
                tail -n 20 "$log_file"
            fi
        done
    else
        local log_file="$LOG_DIR/$service.log"
        if [[ -f "$log_file" ]]; then
            log_step "Logs for $service:"
            tail -f "$log_file"
        else
            log_error "No log file found for $service"
        fi
    fi
}

# Main functions
start_all() {
    log_info "Starting Agent Agency V3 System..."
    
    setup_directories
    check_dependencies
    setup_environment
    
    start_api_server
    start_worker_system
    start_web_dashboard
    
    sleep 2
    health_check
    
    log_success "V3 System started successfully!"
    log_info "Services available at:"
    log_info "  - API Server: http://localhost:8080"
    log_info "  - Web Dashboard: http://localhost:3000"
    log_info "  - Worker System: http://localhost:8081"
    log_info ""
    log_info "Use '$0 status' to check service status"
    log_info "Use '$0 logs [service]' to view logs"
    log_info "Use '$0 stop' to stop all services"
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
        sleep 2
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
        echo "  start   - Start all V3 services"
        echo "  stop    - Stop all V3 services"
        echo "  restart - Restart all V3 services"
        echo "  status  - Show service status"
        echo "  logs    - Show logs (optionally specify service: api-server, worker-system, web-dashboard)"
        echo "  health  - Perform health checks"
        echo ""
        echo "Services:"
        for i in "${!SERVICES[@]}"; do
            echo "  - ${SERVICES[$i]} (${SERVICE_PACKAGES[$i]}) - Port ${SERVICE_PORTS[$i]}"
        done
        exit 1
        ;;
esac