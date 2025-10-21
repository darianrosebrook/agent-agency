#!/bin/bash
# Automated API Server Failover Script
# Handles API server failover with load balancer updates

set -euo pipefail

# Configuration
PRIMARY_API_HOST="${PRIMARY_API_HOST:-api-01.example.com}"
PRIMARY_API_PORT="${PRIMARY_API_PORT:-8080}"
BACKUP_API_HOST="${BACKUP_API_HOST:-api-02.example.com}"
BACKUP_API_PORT="${BACKUP_API_PORT:-8080}"
LOAD_BALANCER_HOST="${LOAD_BALANCER_HOST:-lb.example.com}"
HEALTH_CHECK_ENDPOINT="${HEALTH_CHECK_ENDPOINT:-/health}"
NOTIFICATION_WEBHOOK="${NOTIFICATION_WEBHOOK:-}"

# Logging
LOG_FILE="/var/log/agent-agency/api-failover-$(date +%Y%m%d-%H%M%S).log"
exec 1> >(tee -a "$LOG_FILE")
exec 2>&1

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >&2
}

error() {
    log "ERROR: $*" >&2
    exit 1
}

notify() {
    local message="$1"
    local level="${2:-info}"

    log "[$level] $message"

    if [[ -n "$NOTIFICATION_WEBHOOK" ]]; then
        curl -s -X POST "$NOTIFICATION_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{\"level\":\"$level\",\"message\":\"$message\",\"service\":\"api-server\",\"timestamp\":\"$(date -Iseconds)\"}" || true
    fi
}

check_api_health() {
    local host="$1"
    local port="$2"
    local endpoint="$3"
    local timeout="${4:-10}"

    if timeout "$timeout" curl -f -s "http://$host:$port$endpoint" >/dev/null; then
        return 0
    else
        return 1
    fi
}

wait_for_backup_ready() {
    local max_attempts="${1:-30}"
    local attempt=1

    log "Waiting for backup API server to be ready..."

    while [[ $attempt -le $max_attempts ]]; do
        if check_api_health "$BACKUP_API_HOST" "$BACKUP_API_PORT" "$HEALTH_CHECK_ENDPOINT"; then
            log "Backup API server is ready"
            return 0
        fi

        log "Attempt $attempt/$max_attempts: Backup API not ready yet"
        sleep 2
        ((attempt++))
    done

    error "Backup API server failed to become ready after $max_attempts attempts"
}

start_backup_service() {
    log "Starting backup API server..."

    # This would depend on your service management system (systemd, docker, k8s, etc.)
    case "${SERVICE_MANAGER:-systemd}" in
        systemd)
            ssh "$BACKUP_API_HOST" "sudo systemctl start agent-agency-api" || error "Failed to start backup service via systemd"
            ;;
        docker)
            ssh "$BACKUP_API_HOST" "docker start agent-agency-api" || error "Failed to start backup service via docker"
            ;;
        kubernetes)
            kubectl scale deployment agent-agency-api --replicas=2 || error "Failed to scale deployment"
            ;;
        *)
            error "Unsupported service manager: $SERVICE_MANAGER"
            ;;
    esac

    log "Backup service started successfully"
}

update_load_balancer() {
    log "Updating load balancer configuration..."

    # This depends on your load balancer (nginx, haproxy, aws elb, etc.)
    case "${LOAD_BALANCER_TYPE:-nginx}" in
        nginx)
            # Update nginx upstream
            ssh "$LOAD_BALANCER_HOST" "sudo sed -i 's/server $PRIMARY_API_HOST:$PRIMARY_API_PORT down/server $PRIMARY_API_HOST:$PRIMARY_API_PORT down/' /etc/nginx/conf.d/upstream.conf"
            ssh "$LOAD_BALANCER_HOST" "sudo sed -i 's/server $BACKUP_API_HOST:$BACKUP_API_PORT down/server $BACKUP_API_HOST:$BACKUP_API_PORT/' /etc/nginx/conf.d/upstream.conf"
            ssh "$LOAD_BALANCER_HOST" "sudo nginx -s reload"
            ;;
        haproxy)
            # Update HAProxy backend
            ssh "$LOAD_BALANCER_HOST" "sudo sed -i 's/server primary $PRIMARY_API_HOST:$PRIMARY_API_PORT check down/server primary $PRIMARY_API_HOST:$PRIMARY_API_PORT check/' /etc/haproxy/haproxy.cfg"
            ssh "$LOAD_BALANCER_HOST" "sudo sed -i 's/server backup $BACKUP_API_HOST:$BACKUP_API_PORT check down/server backup $BACKUP_API_HOST:$BACKUP_API_PORT check/' /etc/haproxy/haproxy.cfg"
            ssh "$LOAD_BALANCER_HOST" "sudo systemctl reload haproxy"
            ;;
        aws-elb)
            # Update AWS ELB target group
            aws elbv2 deregister-targets --target-group-arn "$TARGET_GROUP_ARN" --targets "Id=$PRIMARY_API_HOST,Port=$PRIMARY_API_PORT"
            aws elbv2 register-targets --target-group-arn "$TARGET_GROUP_ARN" --targets "Id=$BACKUP_API_HOST,Port=$BACKUP_API_PORT"
            ;;
        *)
            error "Unsupported load balancer type: $LOAD_BALANCER_TYPE"
            ;;
    esac

    log "Load balancer updated successfully"
}

verify_failover() {
    log "Verifying API failover success..."

    # Test load balancer endpoint
    if check_api_health "$LOAD_BALANCER_HOST" 80 "$HEALTH_CHECK_ENDPOINT" 30; then
        log "Load balancer routing working correctly"
    else
        error "Load balancer health check failed"
    fi

    # Test backup API directly
    if check_api_health "$BACKUP_API_HOST" "$BACKUP_API_PORT" "$HEALTH_CHECK_ENDPOINT"; then
        log "Backup API server health check passed"
    else
        error "Backup API server health check failed"
    fi

    # Test some application endpoints
    local test_endpoints=("/api/v1/tasks" "/api/v1/metrics")
    for endpoint in "${test_endpoints[@]}"; do
        if curl -f -s "http://$LOAD_BALANCER_HOST:80$endpoint" >/dev/null; then
            log "Endpoint $endpoint accessible through load balancer"
        else
            log "WARNING: Endpoint $endpoint not accessible"
        fi
    done

    log "API failover verification completed"
}

main() {
    local start_time
    start_time=$(date +%s)

    notify "Starting API server failover procedure" "warning"

    # Step 1: Check if primary is actually down
    log "Checking primary API server health..."
    if check_api_health "$PRIMARY_API_HOST" "$PRIMARY_API_PORT" "$HEALTH_CHECK_ENDPOINT" 15; then
        notify "Primary API server is still healthy - aborting failover" "warning"
        exit 0
    fi

    notify "Primary API server confirmed down - proceeding with failover" "error"

    # Step 2: Start backup service if not already running
    if ! check_api_health "$BACKUP_API_HOST" "$BACKUP_API_PORT" "$HEALTH_CHECK_ENDPOINT" 5; then
        start_backup_service
    fi

    # Step 3: Wait for backup to be ready
    wait_for_backup_ready 60

    # Step 4: Update load balancer
    update_load_balancer

    # Step 5: Verify failover
    verify_failover

    # Step 6: Calculate metrics
    local end_time
    local duration
    end_time=$(date +%s)
    duration=$((end_time - start_time))

    notify "API server failover completed successfully in ${duration}s" "info"

    # Step 7: Send metrics
    if [[ -n "${METRICS_ENDPOINT:-}" ]]; then
        curl -s -X POST "$METRICS_ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "{\"metric\":\"api_failover_duration\",\"value\":$duration,\"timestamp\":\"$(date -Iseconds)\"}" || true
    fi
}

# Run main function
main "$@"
