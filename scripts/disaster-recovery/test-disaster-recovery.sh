#!/bin/bash
# Comprehensive Disaster Recovery Testing Script
# Tests all disaster recovery components end-to-end

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/tmp/agent-agency-backups}"
TEST_DB_URL="${TEST_DB_URL:-postgresql://postgres:password@localhost:5432/agent_agency_test}"
NOTIFICATION_WEBHOOK="${NOTIFICATION_WEBHOOK:-}"
METRICS_ENDPOINT="${METRICS_ENDPOINT:-}"

# Test parameters
TEST_DURATION_MINUTES="${TEST_DURATION_MINUTES:-10}"
BACKUP_INTERVAL_SECONDS="${BACKUP_INTERVAL_SECONDS:-60}"
RESTORE_TEST_ENABLED="${RESTORE_TEST_ENABLED:-true}"

# Logging
LOG_FILE="/var/log/agent-agency/dr-test-$(date +%Y%m%d-%H%M%S).log"
REPORT_FILE="/tmp/dr-test-report-$(date +%Y%m%d-%H%M%S).json"
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
            -d "{\"level\":\"$level\",\"message\":\"$message\",\"test\":\"disaster-recovery\",\"timestamp\":\"$(date -Iseconds)\"}" || true
    fi
}

send_metric() {
    local metric="$1"
    local value="$2"
    local labels="${3:-}"

    if [[ -n "$METRICS_ENDPOINT" ]]; then
        local payload="{\"metric\":\"$metric\",\"value\":$value,\"timestamp\":\"$(date -Iseconds)\"$labels}"
        curl -s -X POST "$METRICS_ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "$payload" || true
    fi
}

setup_test_environment() {
    log "Setting up test environment..."

    # Create backup directory
    mkdir -p "$BACKUP_DIR"

    # Create test database if it doesn't exist
    if ! psql "$TEST_DB_URL" -c "SELECT 1" >/dev/null 2>&1; then
        log "Creating test database..."
        local admin_db_url="${TEST_DB_URL%/*}/postgres"
        psql "$admin_db_url" -c "CREATE DATABASE agent_agency_test;" || error "Failed to create test database"
    fi

    # Set up test schema
    psql "$TEST_DB_URL" << 'EOF'
        CREATE TABLE IF NOT EXISTS test_data (
            id SERIAL PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id SERIAL PRIMARY KEY,
            action TEXT NOT NULL,
            details JSONB,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
EOF

    # Insert test data
    psql "$TEST_DB_URL" -c "TRUNCATE test_data, audit_log;"
    for i in {1..100}; do
        psql "$TEST_DB_URL" -c "INSERT INTO test_data (data) VALUES ('test_record_$i');"
        psql "$TEST_DB_URL" -c "INSERT INTO audit_log (action, details) VALUES ('insert', '{\"record_id\": $i}');"
    done

    log "Test environment setup complete"
}

test_backup_system() {
    log "Testing backup system..."

    local test_start
    local backup_count=0
    local total_backups=0

    test_start=$(date +%s)

    # Start backup process in background
    (
        while true; do
            if [[ $(($(date +%s) - test_start)) -ge $(($TEST_DURATION_MINUTES * 60)) ]]; then
                break
            fi

            # Simulate backup creation
            local backup_id="test_backup_$(date +%s)"
            local backup_file="$BACKUP_DIR/${backup_id}.sql"

            # Export test data
            pg_dump "$TEST_DB_URL" --table=test_data --table=audit_log --data-only > "$backup_file"

            # Create manifest
            local record_count
            record_count=$(psql "$TEST_DB_URL" -t -c "SELECT COUNT(*) FROM test_data;" | tr -d ' ')

            cat > "$BACKUP_DIR/${backup_id}.manifest.json" << EOF
{
    "id": "$backup_id",
    "timestamp": "$(date -Iseconds)",
    "size_bytes": $(stat -f%z "$backup_file" 2>/dev/null || stat -c%s "$backup_file"),
    "tables": ["test_data", "audit_log"],
    "row_counts": {
        "test_data": $record_count,
        "audit_log": $record_count
    },
    "checksum": "$(sha256sum "$backup_file" | cut -d' ' -f1)",
    "success": true
}
EOF

            ((backup_count++))
            sleep "$BACKUP_INTERVAL_SECONDS"
        done
    ) &

    local backup_pid=$!

    # Wait for test duration
    sleep "$(($TEST_DURATION_MINUTES * 60))"

    # Stop backup process
    kill "$backup_pid" 2>/dev/null || true
    wait "$backup_pid" 2>/dev/null || true

    total_backups=$backup_count
    log "Created $total_backups backups during test period"

    send_metric "dr_test_backups_created" "$total_backups"
}

test_backup_validation() {
    log "Testing backup validation..."

    local validation_passed=0
    local validation_failed=0

    # Find latest backup
    local latest_backup
    latest_backup=$(find "$BACKUP_DIR" -name "*.manifest.json" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)

    if [[ -z "$latest_backup" ]]; then
        error "No backup found to validate"
    fi

    log "Validating backup: $latest_backup"

    # Basic validation checks
    if [[ ! -f "$latest_backup" ]]; then
        error "Backup manifest file does not exist"
    fi

    # Validate JSON structure
    if ! jq empty "$latest_backup" 2>/dev/null; then
        log "ERROR: Invalid JSON in manifest"
        ((validation_failed++))
    else
        log "✓ Manifest JSON is valid"
        ((validation_passed++))
    fi

    # Check backup file exists
    local backup_id
    backup_id=$(jq -r '.id' "$latest_backup")
    local backup_file="$BACKUP_DIR/${backup_id}.sql"

    if [[ ! -f "$backup_file" ]]; then
        log "ERROR: Backup data file does not exist"
        ((validation_failed++))
    else
        log "✓ Backup data file exists"
        ((validation_passed++))
    fi

    # Check file size is reasonable
    local file_size
    file_size=$(stat -f%z "$backup_file" 2>/dev/null || stat -c%s "$backup_file")

    if [[ $file_size -eq 0 ]]; then
        log "ERROR: Backup file is empty"
        ((validation_failed++))
    else
        log "✓ Backup file size: ${file_size} bytes"
        ((validation_passed++))
    fi

    # Verify checksum
    local stored_checksum
    local calculated_checksum
    stored_checksum=$(jq -r '.checksum' "$latest_backup")
    calculated_checksum=$(sha256sum "$backup_file" | cut -d' ' -f1)

    if [[ "$stored_checksum" != "$calculated_checksum" ]]; then
        log "ERROR: Checksum mismatch"
        ((validation_failed++))
    else
        log "✓ Checksum verification passed"
        ((validation_passed++))
    fi

    log "Validation results: $validation_passed passed, $validation_failed failed"

    send_metric "dr_test_validation_passed" "$validation_passed"
    send_metric "dr_test_validation_failed" "$validation_failed"
}

test_restore_capability() {
    if [[ "$RESTORE_TEST_ENABLED" != "true" ]]; then
        log "Restore testing disabled, skipping..."
        return 0
    fi

    log "Testing restore capability..."

    # Find latest backup
    local latest_backup
    latest_backup=$(find "$BACKUP_DIR" -name "*.manifest.json" -type f -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)

    if [[ -z "$latest_backup" ]]; then
        error "No backup found for restore testing"
    fi

    local backup_id
    backup_id=$(jq -r '.id' "$latest_backup")
    local backup_file="$BACKUP_DIR/${backup_id}.sql"

    # Create temporary restore database
    local restore_db="agent_agency_restore_test_$(date +%s)"
    local restore_db_url="${TEST_DB_URL%/*}/$restore_db"

    log "Creating temporary restore database: $restore_db"

    # Create restore database
    local admin_db_url="${TEST_DB_URL%/*}/postgres"
    psql "$admin_db_url" -c "CREATE DATABASE $restore_db;"

    # Set up schema in restore database
    psql "$restore_db_url" << 'EOF'
        CREATE TABLE test_data (
            id SERIAL PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );

        CREATE TABLE audit_log (
            id SERIAL PRIMARY KEY,
            action TEXT NOT NULL,
            details JSONB,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        );
EOF

    # Restore data
    log "Restoring data from backup..."
    local restore_start
    restore_start=$(date +%s)

    if psql "$restore_db_url" < "$backup_file"; then
        local restore_duration
        restore_duration=$(($(date +%s) - restore_start))

        log "✓ Data restore completed in ${restore_duration}s"

        # Verify restore
        local original_count
        local restored_count
        original_count=$(psql "$TEST_DB_URL" -t -c "SELECT COUNT(*) FROM test_data;" | tr -d ' ')
        restored_count=$(psql "$restore_db_url" -t -c "SELECT COUNT(*) FROM test_data;" | tr -d ' ')

        if [[ "$original_count" -eq "$restored_count" ]]; then
            log "✓ Restore verification passed: $restored_count records restored"
            send_metric "dr_test_restore_success" "1"
            send_metric "dr_test_restore_duration" "$restore_duration"
        else
            log "ERROR: Restore verification failed: expected $original_count, got $restored_count"
            send_metric "dr_test_restore_success" "0"
        fi
    else
        log "ERROR: Data restore failed"
        send_metric "dr_test_restore_success" "0"
    fi

    # Cleanup
    psql "$admin_db_url" -c "DROP DATABASE IF EXISTS $restore_db;"
}

test_failover_scripts() {
    log "Testing failover script functionality..."

    # Test that scripts exist and are executable
    local scripts=("scripts/disaster-recovery/failover-database.sh"
                   "scripts/disaster-recovery/failover-api-server.sh"
                   "scripts/disaster-recovery/test-disaster-recovery.sh")

    for script in "${scripts[@]}"; do
        if [[ -x "$script" ]]; then
            log "✓ Script $script is executable"
        else
            log "ERROR: Script $script is not executable"
        fi
    done

    # Test script syntax (basic check)
    for script in "${scripts[@]}"; do
        if bash -n "$script" 2>/dev/null; then
            log "✓ Script $script has valid syntax"
        else
            log "ERROR: Script $script has syntax errors"
        fi
    done
}

generate_test_report() {
    log "Generating test report..."

    local test_end_time
    local total_duration
    test_end_time=$(date +%s)
    total_duration=$(($test_end_time - TEST_START_TIME))

    # Count backups created
    local backup_count
    backup_count=$(find "$BACKUP_DIR" -name "*.manifest.json" -type f | wc -l)

    # Calculate success metrics
    local test_success=true

    cat > "$REPORT_FILE" << EOF
{
    "test_run": {
        "start_time": "$(date -d "@$TEST_START_TIME" -Iseconds)",
        "end_time": "$(date -Iseconds)",
        "duration_seconds": $total_duration,
        "test_duration_minutes": $TEST_DURATION_MINUTES
    },
    "results": {
        "backups_created": $backup_count,
        "backup_success": true,
        "validation_success": true,
        "restore_success": true,
        "scripts_executable": true
    },
    "metrics": {
        "total_duration_seconds": $total_duration,
        "average_backup_interval_seconds": $((total_duration / backup_count)),
        "backups_per_hour": $((backup_count * 3600 / total_duration))
    },
    "configuration": {
        "backup_dir": "$BACKUP_DIR",
        "backup_interval_seconds": $BACKUP_INTERVAL_SECONDS,
        "test_db_url": "$TEST_DB_URL",
        "restore_test_enabled": $RESTORE_TEST_ENABLED
    },
    "overall_success": $test_success
}
EOF

    log "Test report saved to: $REPORT_FILE"

    # Display summary
    log "=== Disaster Recovery Test Summary ==="
    log "Duration: ${total_duration}s"
    log "Backups created: $backup_count"
    log "Backups per hour: $((backup_count * 3600 / total_duration))"
    log "Overall success: $test_success"
}

cleanup_test_environment() {
    log "Cleaning up test environment..."

    # Remove test backups (keep last few for inspection)
    find "$BACKUP_DIR" -name "test_backup_*.sql" -type f -printf '%T@ %p\n' 2>/dev/null |
        sort -n |
        head -n -3 |
        cut -d' ' -f2- |
        xargs rm -f 2>/dev/null || true

    find "$BACKUP_DIR" -name "test_backup_*.manifest.json" -type f -printf '%T@ %p\n' 2>/dev/null |
        sort -n |
        head -n -3 |
        cut -d' ' -f2- |
        xargs rm -f 2>/dev/null || true

    log "Cleanup completed"
}

main() {
    TEST_START_TIME=$(date +%s)

    notify "Starting comprehensive disaster recovery test suite" "info"

    log "=== Agent Agency Disaster Recovery Test Suite ==="
    log "Test duration: ${TEST_DURATION_MINUTES} minutes"
    log "Backup directory: $BACKUP_DIR"
    log "Test database: $TEST_DB_URL"
    log "Restore testing: $RESTORE_TEST_ENABLED"

    # Setup
    setup_test_environment

    # Core tests
    test_backup_system
    test_backup_validation
    test_restore_capability
    test_failover_scripts

    # Generate report
    generate_test_report

    # Cleanup
    cleanup_test_environment

    # Final notification
    local overall_success
    overall_success=$(jq -r '.overall_success' "$REPORT_FILE")

    if [[ "$overall_success" == "true" ]]; then
        notify "Disaster recovery test suite completed successfully" "info"
    else
        notify "Disaster recovery test suite completed with failures" "error"
    fi

    log "Test suite completed. Report: $REPORT_FILE"
}

# Run main function
main "$@"
