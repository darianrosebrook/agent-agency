-- Worker resilience and capability tracking
-- Migration 011: Worker Capabilities and Task Snapshots
-- Author: @darianrosebrook

-- Worker capabilities (real-time registration)
CREATE TABLE IF NOT EXISTS worker_capabilities (
    worker_id TEXT PRIMARY KEY,
    capabilities JSONB NOT NULL DEFAULT '{}',
    health_status TEXT NOT NULL DEFAULT 'unknown',
    saturation_ratio REAL NOT NULL DEFAULT 0.0,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    registered_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Task snapshots (resumable state)
CREATE TABLE IF NOT EXISTS task_snapshots (
    task_id TEXT PRIMARY KEY,
    snapshot_data JSONB NOT NULL DEFAULT '{}',
    snapshot_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_worker_capabilities_health ON worker_capabilities (health_status);

CREATE INDEX IF NOT EXISTS idx_worker_capabilities_heartbeat ON worker_capabilities (last_heartbeat);

CREATE INDEX IF NOT EXISTS idx_task_snapshots_expires ON task_snapshots (expires_at);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_worker_capabilities_updated_at
  BEFORE UPDATE ON worker_capabilities
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_task_snapshots_updated_at
  BEFORE UPDATE ON task_snapshots
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
