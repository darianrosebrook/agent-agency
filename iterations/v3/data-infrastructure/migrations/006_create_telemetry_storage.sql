-- Create telemetry storage schema
-- Migration: 006_create_telemetry_storage.sql

-- ===========================================
-- TELEMETRY DATA STORAGE
-- ===========================================

-- Store telemetry data points (metrics, logs, events, traces)
CREATE TABLE IF NOT EXISTS telemetry_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    source VARCHAR(255) NOT NULL,
    data_type VARCHAR(50) NOT NULL CHECK (data_type IN ('Metric', 'Log', 'Trace', 'Event', 'Custom')),
    payload JSONB NOT NULL,
    tags JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_telemetry_data_timestamp ON telemetry_data(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_data_source ON telemetry_data(source);
CREATE INDEX IF NOT EXISTS idx_telemetry_data_type ON telemetry_data(data_type);
CREATE INDEX IF NOT EXISTS idx_telemetry_data_tags ON telemetry_data USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_telemetry_data_payload ON telemetry_data USING GIN(payload);

-- Composite index for common queries (source + type + timestamp)
CREATE INDEX IF NOT EXISTS idx_telemetry_data_source_type_time ON telemetry_data(source, data_type, timestamp DESC);

-- Partitioning by month for better performance with large datasets
-- Note: PostgreSQL 10+ supports declarative partitioning
-- This creates a partitioned table structure (requires manual partition creation)

-- ===========================================
-- TELEMETRY BATCHES
-- ===========================================

-- Store telemetry batches for batch processing
CREATE TABLE IF NOT EXISTS telemetry_batches (
    id VARCHAR(255) PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    data_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    processing_status VARCHAR(50) DEFAULT 'pending' CHECK (processing_status IN ('pending', 'processing', 'completed', 'failed'))
);

-- Index for batch queries
CREATE INDEX IF NOT EXISTS idx_telemetry_batches_timestamp ON telemetry_batches(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_batches_status ON telemetry_batches(processing_status);

-- ===========================================
-- TELEMETRY RETENTION POLICY
-- ===========================================

-- Create a function to clean up old telemetry data
-- This can be called by a scheduled job
CREATE OR REPLACE FUNCTION cleanup_old_telemetry_data(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM telemetry_data
    WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- TELEMETRY AGGREGATION VIEWS
-- ===========================================

-- View for daily telemetry summaries
CREATE OR REPLACE VIEW telemetry_daily_summary AS
SELECT
    DATE_TRUNC('day', timestamp) as day,
    source,
    data_type,
    COUNT(*) as count,
    MIN(timestamp) as first_timestamp,
    MAX(timestamp) as last_timestamp
FROM telemetry_data
GROUP BY DATE_TRUNC('day', timestamp), source, data_type;

-- View for hourly telemetry summaries
CREATE OR REPLACE VIEW telemetry_hourly_summary AS
SELECT
    DATE_TRUNC('hour', timestamp) as hour,
    source,
    data_type,
    COUNT(*) as count,
    MIN(timestamp) as first_timestamp,
    MAX(timestamp) as last_timestamp
FROM telemetry_data
GROUP BY DATE_TRUNC('hour', timestamp), source, data_type;




