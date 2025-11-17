#!/bin/bash

# Production Deployment Script for Agent Agency V3

set -euo pipefail

# Path configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_V3_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
ENVIRONMENT="${ENVIRONMENT:-production}"
COMPOSE_FILE="${COMPOSE_FILE:-$REPO_ROOT/deploy/docker/docker-compose.production.yml}"
MIGRATIONS_DIR="${MIGRATIONS_DIR:-$REPO_ROOT/iterations/v3/data-infrastructure/migrations}"
BACKUP_ROOT="${BACKUP_ROOT:-$REPO_ROOT/deploy/backups}"
LOG_FILE="${LOG_FILE:-$REPO_ROOT/deploy/logs/production-deploy.log}"

# Docker compose wrapper (supports docker compose v2 or docker-compose v1)
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE_BIN="docker-compose"
elif docker compose version &> /dev/null; then
    DOCKER_COMPOSE_BIN="docker compose"
else
    echo "docker compose is required but not installed"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] ✓${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] ⚠${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ✗${NC} $1" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log_error "Deployment failed: $1"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if Docker is installed and running
    if ! command -v docker &> /dev/null; then
        error_exit "Docker is not installed"
    fi
    
    if ! docker info &> /dev/null; then
        error_exit "Docker daemon is not running"
    fi
    
    # Check if Docker Compose is installed
    if [ ! -f "$COMPOSE_FILE" ]; then
        error_exit "Compose file not found at $COMPOSE_FILE"
    fi

    if [ ! -d "$MIGRATIONS_DIR" ]; then
        error_exit "Migrations directory not found at $MIGRATIONS_DIR"
    fi
    
    # Check if required environment variables are set
    local required_vars=("DATABASE_PASSWORD" "REDIS_PASSWORD" "JWT_SECRET" "API_KEY")
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            error_exit "Required environment variable $var is not set"
        fi
    done
    
    log_success "Prerequisites check passed"
}

# Create backup
create_backup() {
    log "Creating backup of current deployment..."
    
    local backup_timestamp
    backup_timestamp=$(date +'%Y%m%d_%H%M%S')
    local backup_path="$BACKUP_ROOT/agent-agency-$backup_timestamp"
    
    mkdir -p "$BACKUP_ROOT"
    mkdir -p "$backup_path"
    
    if $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" ps postgres | grep -q "Up"; then
        log "Backing up PostgreSQL database..."
        $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" exec -T postgres pg_dump -U agent_agency agent_agency > "$backup_path/database.sql"
        log_success "Database backup created"
    else
        log_warning "PostgreSQL container not running, skipping database backup"
    fi
    
    cp -r "$REPO_ROOT/deploy/docker/nginx" "$backup_path/nginx"
    cp -r "$REPO_ROOT/deploy/docker/monitoring" "$backup_path/monitoring"
    cp -r "$MIGRATIONS_DIR" "$backup_path/migrations"
    
    local postgres_volume
    postgres_volume=$(docker volume ls --format '{{.Name}}' | grep -m1 'postgres_data$' || true)
    if [[ -n "$postgres_volume" ]]; then
        log "Backing up Docker volume: $postgres_volume"
        docker run --rm -v "$postgres_volume":/data -v "$backup_path":/backup alpine tar czf /backup/postgres_data.tar.gz -C /data .
    else
        log_warning "PostgreSQL volume not found, skipping volume backup"
    fi

    local redis_volume
    redis_volume=$(docker volume ls --format '{{.Name}}' | grep -m1 'redis_data$' || true)
    if [[ -n "$redis_volume" ]]; then
        log "Backing up Docker volume: $redis_volume"
        docker run --rm -v "$redis_volume":/data -v "$backup_path":/backup alpine tar czf /backup/redis_data.tar.gz -C /data .
    else
        log_warning "Redis volume not found, skipping volume backup"
    fi
    
    log_success "Backup completed: $backup_path"
}

# Run database migrations
run_migrations() {
    log "Running database migrations..."
    
    # Wait for PostgreSQL to be ready
    log "Waiting for PostgreSQL to be ready..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" exec -T postgres pg_isready -U agent_agency -d agent_agency &> /dev/null; then
            log_success "PostgreSQL is ready"
            break
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            error_exit "PostgreSQL failed to become ready after $max_attempts attempts"
        fi
        
        log "Attempt $attempt/$max_attempts: PostgreSQL not ready, waiting..."
        sleep 10
        ((attempt++))
    done
    
    for migration_file in "$MIGRATIONS_DIR"/*.sql; do
        if [[ -f "$migration_file" ]]; then
            log "Running migration: $(basename "$migration_file")"
            $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" exec -T postgres psql -U agent_agency -d agent_agency -f - < "$migration_file"
        fi
    done
    
    log_success "Database migrations completed"
}

# Build and deploy services
deploy_services() {
    log "Building and deploying services..."
    
    # Pull upstream images for infrastructure components
    log "Pulling latest base images..."
    $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" pull postgres redis nginx prometheus grafana fluent-bit
    
    # Build first-party services
    log "Building Agent Agency services..."
    $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" build api dashboard worker
    
    log "Deploying services..."
    $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" up -d
    
    log_success "Services deployed"
}

# Health checks
run_health_checks() {
    log "Running health checks..."
    
    local services=("postgres" "redis" "api" "worker" "dashboard" "nginx" "prometheus" "grafana" "fluent-bit" "node-exporter" "cadvisor")
    local max_attempts=30
    local attempt=1
    
    for service in "${services[@]}"; do
        log "Checking health of $service..."
        
        while [ $attempt -le $max_attempts ]; do
            if $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" ps "$service" | grep -q "Up"; then
                log_success "$service is healthy"
                break
            fi
            
            if [ $attempt -eq $max_attempts ]; then
                error_exit "$service failed health check after $max_attempts attempts"
            fi
            
            log "Attempt $attempt/$max_attempts: $service not ready, waiting..."
            sleep 10
            ((attempt++))
        done
        
        attempt=1
    done
    
    log "Testing API endpoints..."
    local api_url="http://localhost:3000"
    local api_endpoints=("/health" "/metrics" "/api/v1/projects")
    
    for endpoint in "${api_endpoints[@]}"; do
        local api_attempt=1
        while [ $api_attempt -le 10 ]; do
            if curl -f -s "$api_url$endpoint" &> /dev/null; then
                log_success "API endpoint $endpoint is responding"
                break
            fi
            if [ $api_attempt -eq 10 ]; then
                error_exit "API endpoint $endpoint failed health check"
            fi
            log "Attempt $api_attempt/10: $endpoint not responding, waiting..."
            sleep 5
            ((api_attempt++))
        done
    done

    log "Testing worker health endpoint..."
    local worker_attempt=1
    while [ $worker_attempt -le 10 ]; do
        if curl -f -s "http://localhost:9090/health" &> /dev/null; then
            log_success "Worker health endpoint is responding"
            break
        fi
        if [ $worker_attempt -eq 10 ]; then
            error_exit "Worker health endpoint failed health check"
        fi
        log "Attempt $worker_attempt/10: worker not responding, waiting..."
        sleep 5
        ((worker_attempt++))
    done
    
    log_success "All health checks passed"
}

# Performance testing
run_performance_tests() {
    log "Running performance tests..."
    
    # Check if k6 is available
    if ! command -v k6 &> /dev/null; then
        log_warning "k6 not found, skipping performance tests"
        return
    fi
    
    # Run basic load test
    local load_test_dir="$REPO_ROOT/tests/performance"
    if [[ ! -d "$load_test_dir" ]]; then
        log_warning "Performance test directory not found ($load_test_dir). Documenting dependency and continuing."
        return
    fi

    local test_script="$load_test_dir/agent-agency-smoke.js"
    if [[ ! -f "$test_script" ]]; then
        log_warning "k6 test script missing at $test_script. Please provide a load test script to enable automated performance validation."
        return
    fi

    log "Running basic load test..."
    if k6 run --duration 2m --vus 10 "$test_script"; then
        log_success "Performance tests passed"
    else
        log_warning "Performance tests failed, but deployment continues"
    fi
}

# Cleanup old resources
cleanup() {
    log "Cleaning up old resources..."
    
    # Remove old Docker images
    docker image prune -f
    
    # Remove old containers
    docker container prune -f
    
    # Remove old volumes (be careful with this)
    # docker volume prune -f
    
    log_success "Cleanup completed"
}

# Main deployment function
main() {
    log "Starting Agent Agency production deployment..."
    log "Environment: $ENVIRONMENT"
    log "Repo root: $REPO_ROOT"
    
    # Create log directory if it doesn't exist
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Run deployment steps
    check_prerequisites
    create_backup
    deploy_services
    run_migrations
    run_health_checks
    run_performance_tests
    cleanup
    
    log_success "Production deployment completed successfully!"
    log "Services are available at:"
    log "  - API: http://localhost:3000"
    log "  - Dashboard: http://localhost:3001"
    log "  - Grafana: http://localhost:3002"
    log "  - Prometheus: http://localhost:9090"
    log "  - Nginx: http://localhost"
}

# Handle script arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "rollback")
        log "Rollback functionality not implemented yet"
        exit 1
        ;;
    "status")
        log "Checking deployment status..."
        $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" ps
        ;;
    "logs")
        service="${2:-api}"
        $DOCKER_COMPOSE_BIN -f "$COMPOSE_FILE" logs -f "$service"
        ;;
    "backup")
        create_backup
        ;;
    *)
        echo "Usage: $0 {deploy|rollback|status|logs|backup}"
        echo "  deploy  - Deploy the Agent Agency stack (default)"
        echo "  rollback - Rollback to previous version (not implemented)"
        echo "  status  - Show deployment status"
        echo "  logs    - Show logs for a service (default: api)"
        echo "  backup  - Create a backup"
        exit 1
        ;;
esac
