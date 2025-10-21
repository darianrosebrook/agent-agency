# Agent Agency V3 Database Layer

## Overview

The Agent Agency V3 database layer provides comprehensive persistence for the constitutional AI system, supporting task execution, governance, provenance tracking, and monitoring capabilities. Built on PostgreSQL with ACID compliance, it ensures data integrity and auditability for all system operations.

## Architecture

### Core Components

- **PostgreSQL Database**: Primary data store with full ACID compliance
- **Connection Pooling**: Efficient connection management with deadpool
- **Migration System**: Version-controlled schema management
- **Audit Logging**: Complete operation tracking and provenance

### Key Tables

#### Task Management
- **`tasks`**: Core task execution records with status tracking
- **`task_artifacts`**: Execution artifacts and results storage
- **`task_progress`**: Real-time progress updates and metrics

#### Governance & Compliance
- **`council_verdicts`**: Constitutional council decisions and reasoning
- **`waivers`**: Quality gate exception management
- **`caws_violations`**: Compliance validation results

#### Provenance & Audit
- **`provenance_records`**: Cryptographically signed audit trails
- **`git_commits`**: Git integration for provenance linking
- **`audit_log`**: Comprehensive system operation logging

#### Monitoring & Metrics
- **`system_metrics`**: Performance and health metrics
- **`slo_measurements`**: Service level objective tracking
- **`alerts`**: Automated alerting and acknowledgment

## Database Schema

### Core Task Tables

```sql
-- Task execution and lifecycle management
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    execution_mode VARCHAR(20) DEFAULT 'auto',
    risk_tier INTEGER DEFAULT 2,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Task artifacts and execution results
CREATE TABLE task_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    artifact_type VARCHAR(50) NOT NULL,
    content JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Real-time progress tracking
CREATE TABLE task_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    phase VARCHAR(100) NOT NULL,
    progress_percentage DECIMAL(5,2),
    message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Governance Tables

```sql
-- Constitutional council verdicts
CREATE TABLE council_verdicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    judge_type VARCHAR(50) NOT NULL,
    verdict VARCHAR(20) NOT NULL,
    confidence_score DECIMAL(3,2),
    reasoning TEXT,
    evidence JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Quality gate waivers
CREATE TABLE waivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    reason VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    gates TEXT[] NOT NULL,
    approved_by VARCHAR(255) NOT NULL,
    impact_level VARCHAR(20) NOT NULL DEFAULT 'medium',
    mitigation_plan TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- CAWS compliance violations
CREATE TABLE caws_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id),
    rule VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    description TEXT,
    violation_data JSONB DEFAULT '{}'::jsonb,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Provenance Tables

```sql
-- Cryptographically signed provenance records
CREATE TABLE provenance_records (
    verdict_id UUID PRIMARY KEY,
    task_id UUID NOT NULL,
    decision_type VARCHAR(50) NOT NULL,
    decision_data JSONB NOT NULL,
    consensus_score DECIMAL(3,2) NOT NULL,
    judge_verdicts JSONB NOT NULL DEFAULT '{}',
    caws_compliance JSONB NOT NULL,
    claim_verification JSONB,
    git_commit_hash VARCHAR(40),
    git_trailer TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Git commit integration
CREATE TABLE git_commits (
    hash VARCHAR(40) PRIMARY KEY,
    author VARCHAR(255),
    message TEXT,
    timestamp TIMESTAMPTZ,
    provenance_record_id UUID REFERENCES provenance_records(verdict_id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Monitoring Tables

```sql
-- System metrics collection
CREATE TABLE system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(15,6) NOT NULL,
    labels JSONB DEFAULT '{}'::jsonb,
    collected_at TIMESTAMPTZ DEFAULT NOW()
);

-- SLO measurements and tracking
CREATE TABLE slo_measurements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slo_name VARCHAR(100) NOT NULL,
    target_value DECIMAL(10,4),
    actual_value DECIMAL(10,4),
    compliance_percentage DECIMAL(5,2),
    measurement_period INTERVAL,
    measured_at TIMESTAMPTZ DEFAULT NOW()
);

-- Alert management
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    acknowledged_by VARCHAR(255),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Migration System

### Migration Files

Database migrations are stored in `iterations/v3/database/migrations/` with numbered files:

```
001_initial_schema.sql
002_add_task_progress.sql
003_add_council_system.sql
004_add_provenance_tables.sql
005_add_waiver_system.sql
006_add_slo_monitoring.sql
007_add_alert_management.sql
008_performance_indexes.sql
009_data_retention_policies.sql
010_add_audit_triggers.sql
011_add_encryption_support.sql
012_add_multi_tenant_support.sql
013_add_advanced_monitoring.sql
014_core_persistence.sql
```

### Migration Commands

```bash
# Run all pending migrations
cd iterations/v3
cargo run --bin migrate

# Create new migration
cargo run --bin migrate -- create add_new_feature

# Rollback last migration
cargo run --bin migrate -- rollback

# Check migration status
cargo run --bin migrate -- status
```

## Performance Optimization

### Indexing Strategy

```sql
-- Task status and lifecycle indexes
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_tasks_task_id ON tasks(task_id);

-- Progress tracking indexes
CREATE INDEX idx_task_progress_task_id ON task_progress(task_id);
CREATE INDEX idx_task_progress_created_at ON task_progress(created_at);

-- Council verdict indexes
CREATE INDEX idx_council_verdicts_task_id ON council_verdicts(task_id);
CREATE INDEX idx_council_verdicts_judge_type ON council_verdicts(judge_type);

-- Waiver system indexes
CREATE INDEX idx_waivers_status ON waivers(status);
CREATE INDEX idx_waivers_expires_at ON waivers(expires_at);
CREATE INDEX idx_waivers_approved_by ON waivers(approved_by);

-- Provenance indexes
CREATE INDEX idx_provenance_records_task_id ON provenance_records(task_id);
CREATE INDEX idx_provenance_records_git_commit_hash ON provenance_records(git_commit_hash);

-- Monitoring indexes
CREATE INDEX idx_system_metrics_name_time ON system_metrics(metric_name, collected_at);
CREATE INDEX idx_slo_measurements_name_time ON slo_measurements(slo_name, measured_at);
CREATE INDEX idx_alerts_status_created ON alerts(status, created_at);
```

### Query Optimization

```sql
-- Efficient task status queries
SELECT * FROM tasks
WHERE status = 'running'
ORDER BY created_at DESC
LIMIT 50;

-- Progress tracking with window functions
SELECT task_id,
       phase,
       progress_percentage,
       LAG(progress_percentage) OVER (PARTITION BY task_id ORDER BY created_at) as prev_progress
FROM task_progress
WHERE task_id = $1
ORDER BY created_at DESC;

-- SLO compliance calculation
SELECT slo_name,
       AVG(compliance_percentage) as avg_compliance,
       MIN(compliance_percentage) as min_compliance
FROM slo_measurements
WHERE measured_at >= NOW() - INTERVAL '24 hours'
GROUP BY slo_name;
```

## Data Retention & Archiving

### Retention Policies

```sql
-- Task data retention (90 days)
CREATE OR REPLACE FUNCTION cleanup_old_tasks() RETURNS void AS $$
BEGIN
    DELETE FROM task_artifacts WHERE created_at < NOW() - INTERVAL '90 days';
    DELETE FROM task_progress WHERE created_at < NOW() - INTERVAL '90 days';
    UPDATE tasks SET status = 'archived' WHERE completed_at < NOW() - INTERVAL '90 days';
END;
$$ LANGUAGE plpgsql;

-- Metrics retention (30 days)
CREATE OR REPLACE FUNCTION cleanup_old_metrics() RETURNS void AS $$
BEGIN
    DELETE FROM system_metrics WHERE collected_at < NOW() - INTERVAL '30 days';
    DELETE FROM slo_measurements WHERE measured_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- Audit log retention (1 year)
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs() RETURNS void AS $$
BEGIN
    DELETE FROM audit_log WHERE created_at < NOW() - INTERVAL '1 year';
END;
$$ LANGUAGE plpgsql;
```

### Automated Cleanup

```sql
-- Schedule automated cleanup
SELECT cron.schedule('cleanup-old-tasks', '0 2 * * *', 'SELECT cleanup_old_tasks();');
SELECT cron.schedule('cleanup-old-metrics', '0 3 * * *', 'SELECT cleanup_old_metrics();');
SELECT cron.schedule('cleanup-audit-logs', '0 4 1 * *', 'SELECT cleanup_old_audit_logs();');
```

## Security & Access Control

### Row Level Security

```sql
-- Enable RLS on sensitive tables
ALTER TABLE provenance_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE waivers ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;

-- Create security policies
CREATE POLICY provenance_access ON provenance_records
    FOR ALL USING (current_user_has_access());

CREATE POLICY waiver_access ON waivers
    FOR ALL USING (approved_by = current_user() OR status = 'active');
```

### Audit Triggers

```sql
-- Automatic audit logging
CREATE OR REPLACE FUNCTION audit_trigger_function() RETURNS trigger AS $$
BEGIN
    INSERT INTO audit_log (table_name, operation, old_values, new_values, user_id, timestamp)
    VALUES (TG_TABLE_NAME, TG_OP, row_to_json(OLD), row_to_json(NEW), current_user, NOW());
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Apply audit triggers to critical tables
CREATE TRIGGER tasks_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON tasks
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER waivers_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON waivers
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();
```

## Backup & Recovery

### Backup Strategy

```bash
# Daily full backup
pg_dump -h localhost -U agent_agency agent_agency_v3 > backup_$(date +%Y%m%d).sql

# Continuous WAL archiving
archive_command = 'cp %p /var/lib/postgresql/archive/%f'

# Point-in-time recovery setup
recovery_target_time = '2025-01-15 10:00:00';
```

### Recovery Procedures

```sql
-- Restore from backup
psql -h localhost -U agent_agency agent_agency_v3 < backup_20250115.sql

-- Verify data integrity
SELECT COUNT(*) FROM tasks;
SELECT COUNT(*) FROM provenance_records;
SELECT COUNT(*) FROM waivers;
```

## Monitoring & Maintenance

### Health Checks

```sql
-- Database connectivity check
SELECT 1 as database_status;

-- Table size monitoring
SELECT schemaname, tablename,
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Index usage analysis
SELECT schemaname, tablename, indexname,
       idx_scan, idx_tup_read, idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### Performance Monitoring

```sql
-- Slow query analysis
SELECT query, calls, total_time, mean_time, rows
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 10;

-- Connection monitoring
SELECT count(*) as active_connections
FROM pg_stat_activity
WHERE state = 'active';

-- Lock monitoring
SELECT locktype, mode, granted,
       pg_class.relname as table_name,
       pg_locks.pid
FROM pg_locks
JOIN pg_class ON pg_locks.relation = pg_class.oid
WHERE pg_class.relname NOT LIKE 'pg_%';
```

## Configuration

### Environment Variables

```bash
# Database connection
DATABASE_URL=postgresql://username:password@localhost:5432/agent_agency_v3

# Connection pool settings
DB_MAX_CONNECTIONS=20
DB_MIN_CONNECTIONS=5
DB_CONNECT_TIMEOUT=30s
DB_IDLE_TIMEOUT=10m

# Migration settings
DB_MIGRATION_TABLE=__migrations__
DB_MIGRATION_PATH=./database/migrations
```

### Connection Pool Configuration

```rust
use deadpool_postgres::{Config, Manager, Pool, Runtime};

let mut cfg = Config::new();
cfg.dbname = Some("agent_agency_v3".to_string());
cfg.user = Some("username".to_string());
cfg.password = Some("password".to_string());
cfg.host = Some("localhost".to_string());
cfg.port = Some(5432);

cfg.manager = Some(Manager::from_config(cfg.clone(), NoTls, Default::default()));
cfg.max_size = 20;
cfg.min_idle = 5;

let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
```

## API Integration

### Database Client Usage

```rust
use agent_agency_database::DatabaseClient;

let db_client = DatabaseClient::new(&database_url).await?;

// Task operations
let task_id = db_client.create_task(task_data).await?;
let task = db_client.get_task(&task_id).await?;
db_client.update_task_status(&task_id, TaskStatus::Completed).await?;

// Waiver operations
let waivers = db_client.list_active_waivers().await?;
let waiver_id = db_client.create_waiver(waiver_data).await?;

// Provenance operations
let provenance = db_client.get_provenance_by_commit(&commit_hash).await?;
db_client.link_provenance_to_commit(&verdict_id, &commit_hash).await?;
```

This database layer provides the foundation for Agent Agency V3's constitutional AI operations, ensuring data integrity, auditability, and performance for all system components.
