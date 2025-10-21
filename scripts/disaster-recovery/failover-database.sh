#!/bin/bash
# Automated Database Failover Script
# Handles database failover scenarios with minimal downtime

set -euo pipefail

# Configuration
PRIMARY_DB_HOST="${PRIMARY_DB_HOST:-localhost}"
PRIMARY_DB_PORT="${PRIMARY_DB_PORT:-5432}"
STANDBY_DB_HOST="${STANDBY_DB_HOST:-localhost}"
STANDBY_DB_PORT="${STANDBY_DB_PORT:-5433}"
DB_NAME="${DB_NAME:-agent_agency_v3}"
DB_USER="${DB_USER:-postgres}"
NOTIFICATION_WEBHOOK="${NOTIFICATION_WEBHOOK:-}"

# Logging
LOG_FILE="/var/log/agent-agency/failover-$(date +%Y%m%d-%H%M%S).log"
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
            -d "{\"level\":\"$level\",\"message\":\"$message\",\"timestamp\":\"$(date -Iseconds)\"}" || true
    fi
}

check_database_health() {
    local host="$1"
    local port="$2"
    local timeout="${3:-5}"

    if timeout "$timeout" psql -h "$host" -p "$port" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

wait_for_standby_ready() {
    local max_attempts="${1:-30}"
    local attempt=1

    log "Waiting for standby database to be ready..."

    while [[ $attempt -le $max_attempts ]]; do
        if check_database_health "$STANDBY_DB_HOST" "$STANDBY_DB_PORT"; then
            log "Standby database is ready"
            return 0
        fi

        log "Attempt $attempt/$max_attempts: Standby not ready yet"
        sleep 2
        ((attempt++))
    done

    error "Standby database failed to become ready after $max_attempts attempts"
}

promote_standby() {
    log "Promoting standby database to primary..."

    # In PostgreSQL, this would use pg_ctl promote or trigger file
    # For this example, we'll simulate the promotion
    if ssh "$STANDBY_DB_HOST" "sudo -u postgres pg_ctl -D /var/lib/postgresql/data promote" 2>/dev/null; then
        log "Standby promotion initiated"
    else
        error "Failed to promote standby database"
    fi
}

update_connection_strings() {
    log "Updating application connection strings..."

    # Update load balancer configuration
    # Update DNS records
    # Update configuration files
    # This would depend on your infrastructure setup

    # Example: Update a configuration file
    if [[ -f "/etc/agent-agency/database.conf" ]]; then
        sed -i "s/host=.*/host=$STANDBY_DB_HOST/" "/etc/agent-agency/database.conf"
        sed -i "s/port=.*/port=$STANDBY_DB_PORT/" "/etc/agent-agency/database.conf"
        log "Updated database configuration"
    fi
}

verify_failover() {
    log "Verifying failover success..."

    # Wait for new primary to accept connections
    wait_for_standby_ready 60

    # Verify data consistency
    local primary_count
    local standby_count

    primary_count=$(psql -h "$PRIMARY_DB_HOST" -p "$PRIMARY_DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM tasks" 2>/dev/null || echo "0")
    standby_count=$(psql -h "$STANDBY_DB_HOST" -p "$STANDBY_DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM tasks" 2>/dev/null || echo "0")

    if [[ "$primary_count" != "$standby_count" ]]; then
        error "Data inconsistency detected: Primary has $primary_count records, Standby has $standby_count records"
    fi

    # Test application connectivity
    if curl -f -s "http://localhost:8080/health" >/dev/null; then
        log "Application health check passed"
    else
        error "Application health check failed after failover"
    fi

    log "Failover verification completed successfully"
}

main() {
    local start_time
    start_time=$(date +%s)

    notify "Starting database failover procedure" "warning"

    # Step 1: Check if primary is actually down
    log "Checking primary database health..."
    if check_database_health "$PRIMARY_DB_HOST" "$PRIMARY_DB_PORT" 10; then
        notify "Primary database is still healthy - aborting failover" "warning"
        exit 0
    fi

    notify "Primary database confirmed down - proceeding with failover" "error"

    # Step 2: Promote standby to primary
    promote_standby

    # Step 3: Wait for promotion to complete
    wait_for_standby_ready 60

    # Step 4: Update connection strings
    update_connection_strings

    # Step 5: Verify failover success
    verify_failover

    # Step 6: Calculate and report metrics
    local end_time
    local duration
    end_time=$(date +%s)
    duration=$((end_time - start_time))

    notify "Database failover completed successfully in ${duration}s" "info"

    # Step 7: Send metrics to monitoring system
    if [[ -n "${METRICS_ENDPOINT:-}" ]]; then
        curl -s -X POST "$METRICS_ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "{\"metric\":\"database_failover_duration\",\"value\":$duration,\"timestamp\":\"$(date -Iseconds)\"}" || true
    fi
}

# Run main function
main "$@"
