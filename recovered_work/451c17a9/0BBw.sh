#!/bin/bash

# Production Deployment Script for Multimodal RAG System
# This script handles the complete production deployment process

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENVIRONMENT="${ENVIRONMENT:-production}"
BACKUP_DIR="${BACKUP_DIR:-/backups}"
LOG_FILE="${LOG_FILE:-/var/log/multimodal-rag-deploy.log}"

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
    if ! command -v docker-compose &> /dev/null; then
        error_exit "Docker Compose is not installed"
    fi
    
    # Check if required environment variables are set
    local required_vars=("POSTGRES_PASSWORD" "REDIS_PASSWORD" "JWT_SECRET" "API_KEY")
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
    
    local backup_timestamp=$(date +'%Y%m%d_%H%M%S')
    local backup_path="$BACKUP_DIR/multimodal-rag-backup-$backup_timestamp"
    
    mkdir -p "$backup_path"
    
    # Backup database
    if docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" ps postgres | grep -q "Up"; then
        log "Backing up PostgreSQL database..."
        docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" exec -T postgres pg_dump -U multimodal_rag multimodal_rag > "$backup_path/database.sql"
        log_success "Database backup created"
    else
        log_warning "PostgreSQL container not running, skipping database backup"
    fi
    
    # Backup configuration files
    cp -r "$PROJECT_ROOT/config" "$backup_path/"
    cp -r "$PROJECT_ROOT/migrations" "$backup_path/"
    
    # Backup volumes
    if docker volume ls | grep -q "multimodal-rag"; then
        log "Backing up Docker volumes..."
        docker run --rm -v multimodal-rag_postgres_data:/data -v "$backup_path":/backup alpine tar czf /backup/postgres_data.tar.gz -C /data .
        docker run --rm -v multimodal-rag_redis_data:/data -v "$backup_path":/backup alpine tar czf /backup/redis_data.tar.gz -C /data .
        log_success "Volume backups created"
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
        if docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" exec -T postgres pg_isready -U multimodal_rag -d multimodal_rag &> /dev/null; then
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
    
    # Run migrations
    for migration_file in "$PROJECT_ROOT/migrations"/*.sql; do
        if [[ -f "$migration_file" ]]; then
            log "Running migration: $(basename "$migration_file")"
            docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" exec -T postgres psql -U multimodal_rag -d multimodal_rag -f - < "$migration_file"
        fi
    done
    
    log_success "Database migrations completed"
}

# Build and deploy services
deploy_services() {
    log "Building and deploying services..."
    
    cd "$PROJECT_ROOT"
    
    # Pull latest images
    log "Pulling latest base images..."
    docker-compose -f docker/docker-compose.production.yml pull postgres redis nginx prometheus grafana elasticsearch kibana
    
    # Build custom images
    log "Building multimodal RAG service image..."
    docker-compose -f docker/docker-compose.production.yml build multimodal-rag-service
    
    # Deploy services
    log "Deploying services..."
    docker-compose -f docker/docker-compose.production.yml up -d
    
    log_success "Services deployed"
}

# Health checks
run_health_checks() {
    log "Running health checks..."
    
    local services=("postgres" "redis" "multimodal-rag-service" "nginx" "prometheus" "grafana")
    local max_attempts=30
    local attempt=1
    
    for service in "${services[@]}"; do
        log "Checking health of $service..."
        
        while [ $attempt -le $max_attempts ]; do
            if docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" ps "$service" | grep -q "Up"; then
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
    
    # Test API endpoints
    log "Testing API endpoints..."
    
    local api_url="http://localhost:8080"
    local endpoints=("/health" "/metrics" "/api/v1/search")
    
    for endpoint in "${endpoints[@]}"; do
        local max_attempts=10
        local attempt=1
        
        while [ $attempt -le $max_attempts ]; do
            if curl -f -s "$api_url$endpoint" &> /dev/null; then
                log_success "API endpoint $endpoint is responding"
                break
            fi
            
            if [ $attempt -eq $max_attempts ]; then
                error_exit "API endpoint $endpoint failed health check"
            fi
            
            log "Attempt $attempt/$max_attempts: $endpoint not responding, waiting..."
            sleep 5
            ((attempt++))
        done
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
    log "Running basic load test..."
    cd "$PROJECT_ROOT/load-testing"
    
    if k6 run --duration 2m --vus 10 k6-multimodal-rag-test.js; then
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
    log "Starting Multimodal RAG production deployment..."
    log "Environment: $ENVIRONMENT"
    log "Project root: $PROJECT_ROOT"
    
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
    log "  - API: http://localhost:8080"
    log "  - Metrics: http://localhost:8081"
    log "  - Grafana: http://localhost:3000"
    log "  - Kibana: http://localhost:5601"
    log "  - Prometheus: http://localhost:9090"
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
        docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" ps
        ;;
    "logs")
        service="${2:-multimodal-rag-service}"
        docker-compose -f "$PROJECT_ROOT/docker/docker-compose.production.yml" logs -f "$service"
        ;;
    "backup")
        create_backup
        ;;
    *)
        echo "Usage: $0 {deploy|rollback|status|logs|backup}"
        echo "  deploy  - Deploy the multimodal RAG system (default)"
        echo "  rollback - Rollback to previous version (not implemented)"
        echo "  status  - Show deployment status"
        echo "  logs    - Show logs for a service"
        echo "  backup  - Create a backup"
        exit 1
        ;;
esac
