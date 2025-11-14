-- WAL (Write-Ahead Log) storage for point-in-time recovery
-- This stores logical WAL records that can be replayed for PITR

CREATE TABLE IF NOT EXISTS wal_log_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    transaction_id UUID NOT NULL,
    sequence_number BIGSERIAL NOT NULL,
    operation_type VARCHAR(20) NOT NULL CHECK (operation_type IN ('INSERT', 'UPDATE', 'DELETE', 'DDL', 'TRUNCATE')),
    schema_name VARCHAR(255) NOT NULL DEFAULT 'public',
    table_name VARCHAR(255) NOT NULL,
    record_id UUID, -- Row ID for tracking specific records
    old_data JSONB, -- Previous state for UPDATE/DELETE
    new_data JSONB, -- New state for INSERT/UPDATE
    sql_statement TEXT, -- For DDL operations
    checksum VARCHAR(64), -- Integrity check
    applied BOOLEAN DEFAULT FALSE,
    replayed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wal_recorded_at ON wal_log_records(recorded_at);
CREATE INDEX IF NOT EXISTS idx_wal_transaction ON wal_log_records(transaction_id);
CREATE INDEX IF NOT EXISTS idx_wal_sequence ON wal_log_records(sequence_number);
CREATE INDEX IF NOT EXISTS idx_wal_table ON wal_log_records(schema_name, table_name);
CREATE INDEX IF NOT EXISTS idx_wal_applied ON wal_log_records(applied);
CREATE INDEX IF NOT EXISTS idx_wal_operation ON wal_log_records(operation_type);

-- WAL replay checkpoint tracking
CREATE TABLE IF NOT EXISTS wal_replay_checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replay_id UUID NOT NULL,
    checkpoint_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_sequence_number BIGINT NOT NULL,
    last_transaction_id UUID,
    records_applied BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL CHECK (status IN ('IN_PROGRESS', 'COMPLETED', 'FAILED', 'PAUSED')),
    error_message TEXT,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_wal_replay_id ON wal_replay_checkpoints(replay_id);
CREATE INDEX IF NOT EXISTS idx_wal_replay_status ON wal_replay_checkpoints(status);

-- Function to get WAL records for a time range
CREATE OR REPLACE FUNCTION get_wal_records_for_replay(
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    table_filter VARCHAR DEFAULT NULL
)
RETURNS TABLE (
    id UUID,
    recorded_at TIMESTAMPTZ,
    transaction_id UUID,
    sequence_number BIGINT,
    operation_type VARCHAR,
    schema_name VARCHAR,
    table_name VARCHAR,
    record_id UUID,
    old_data JSONB,
    new_data JSONB,
    sql_statement TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        w.id,
        w.recorded_at,
        w.transaction_id,
        w.sequence_number,
        w.operation_type,
        w.schema_name,
        w.table_name,
        w.record_id,
        w.old_data,
        w.new_data,
        w.sql_statement
    FROM wal_log_records w
    WHERE w.recorded_at >= start_time
      AND w.recorded_at <= end_time
      AND (table_filter IS NULL OR w.table_name = table_filter)
      AND w.applied = FALSE
    ORDER BY w.sequence_number ASC, w.recorded_at ASC;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup old WAL records (retention policy)
CREATE OR REPLACE FUNCTION cleanup_old_wal_records(retention_days INTEGER DEFAULT 30)
RETURNS BIGINT AS $$
DECLARE
    deleted_count BIGINT;
BEGIN
    DELETE FROM wal_log_records
    WHERE recorded_at < NOW() - (retention_days || ' days')::INTERVAL
      AND applied = TRUE;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- View for WAL statistics
CREATE OR REPLACE VIEW wal_statistics AS
SELECT 
    COUNT(*) as total_records,
    COUNT(*) FILTER (WHERE w.applied = FALSE) as pending_records,
    COUNT(*) FILTER (WHERE w.applied = TRUE) as applied_records,
    MIN(w.recorded_at) as oldest_record,
    MAX(w.recorded_at) as newest_record,
    COUNT(DISTINCT w.transaction_id) as unique_transactions,
    COUNT(DISTINCT w.table_name) as affected_tables,
    w.operation_type,
    COUNT(*) as operation_count
FROM wal_log_records w
GROUP BY w.operation_type;




