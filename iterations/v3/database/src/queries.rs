//! Database query definitions and utilities
//!
//! This module contains SQL query strings and query-related utilities
//! for the database operations.

/// SQL queries for judge operations
pub mod judge_queries {
    pub const CREATE_JUDGE: &str = r#"
        INSERT INTO judges (name, model_name, endpoint, weight, timeout_ms, optimization_target)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_JUDGE_BY_ID: &str = r#"
        SELECT * FROM judges WHERE id = $1
    "#;

    pub const GET_ALL_JUDGES: &str = r#"
        SELECT * FROM judges ORDER BY weight DESC
    "#;

    pub const UPDATE_JUDGE: &str = r#"
        UPDATE judges 
        SET name = $2, model_name = $3, endpoint = $4, weight = $5, timeout_ms = $6, optimization_target = $7
        WHERE id = $1
        RETURNING *
    "#;

    pub const DELETE_JUDGE: &str = r#"
        DELETE FROM judges WHERE id = $1
    "#;
}

/// SQL queries for worker operations
pub mod worker_queries {
    pub const CREATE_WORKER: &str = r#"
        INSERT INTO workers (name, worker_type, capabilities, status, endpoint)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
    "#;

    pub const GET_WORKER_BY_ID: &str = r#"
        SELECT * FROM workers WHERE id = $1
    "#;

    pub const GET_ALL_WORKERS: &str = r#"
        SELECT * FROM workers ORDER BY name
    "#;

    pub const UPDATE_WORKER: &str = r#"
        UPDATE workers 
        SET name = $2, worker_type = $3, capabilities = $4, status = $5, endpoint = $6
        WHERE id = $1
        RETURNING *
    "#;

    pub const DELETE_WORKER: &str = r#"
        DELETE FROM workers WHERE id = $1
    "#;
}

/// SQL queries for task operations
pub mod task_queries {
    pub const CREATE_TASK: &str = r#"
        INSERT INTO tasks (title, description, task_type, priority, status, assigned_worker_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_TASK_BY_ID: &str = r#"
        SELECT * FROM tasks WHERE id = $1
    "#;

    pub const GET_ALL_TASKS: &str = r#"
        SELECT * FROM tasks ORDER BY created_at DESC
    "#;

    pub const UPDATE_TASK: &str = r#"
        UPDATE tasks 
        SET title = $2, description = $3, task_type = $4, priority = $5, status = $6, assigned_worker_id = $7
        WHERE id = $1
        RETURNING *
    "#;

    pub const DELETE_TASK: &str = r#"
        DELETE FROM tasks WHERE id = $1
    "#;
}

/// SQL queries for task execution operations
pub mod task_execution_queries {
    pub const CREATE_TASK_EXECUTION: &str = r#"
        INSERT INTO task_executions (task_id, worker_id, status, started_at, completed_at, result_data)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_TASK_EXECUTION_BY_ID: &str = r#"
        SELECT * FROM task_executions WHERE id = $1
    "#;

    pub const GET_EXECUTIONS_BY_TASK: &str = r#"
        SELECT * FROM task_executions WHERE task_id = $1 ORDER BY started_at DESC
    "#;

    pub const UPDATE_TASK_EXECUTION: &str = r#"
        UPDATE task_executions 
        SET status = $2, completed_at = $3, result_data = $4
        WHERE id = $1
        RETURNING *
    "#;
}

/// SQL queries for council verdict operations
pub mod council_queries {
    pub const CREATE_VERDICT: &str = r#"
        INSERT INTO council_verdicts (task_id, consensus_result, confidence_score, reasoning, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
    "#;

    pub const GET_VERDICT_BY_ID: &str = r#"
        SELECT * FROM council_verdicts WHERE id = $1
    "#;

    pub const GET_VERDICTS_BY_TASK: &str = r#"
        SELECT * FROM council_verdicts WHERE task_id = $1 ORDER BY created_at DESC
    "#;

    pub const GET_ALL_VERDICTS: &str = r#"
        SELECT * FROM council_verdicts ORDER BY created_at DESC
    "#;
}

/// SQL queries for judge evaluation operations
pub mod judge_evaluation_queries {
    pub const CREATE_EVALUATION: &str = r#"
        INSERT INTO judge_evaluations (judge_id, task_id, verdict, confidence, reasoning, evaluation_time_ms)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_EVALUATIONS_BY_JUDGE: &str = r#"
        SELECT * FROM judge_evaluations WHERE judge_id = $1 ORDER BY created_at DESC
    "#;

    pub const GET_EVALUATIONS_BY_TASK: &str = r#"
        SELECT * FROM judge_evaluations WHERE task_id = $1 ORDER BY created_at DESC
    "#;
}

/// SQL queries for debate session operations
pub mod debate_queries {
    pub const CREATE_DEBATE_SESSION: &str = r#"
        INSERT INTO debate_sessions (task_id, status, started_at, ended_at, consensus_reached)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
    "#;

    pub const GET_DEBATE_SESSION_BY_ID: &str = r#"
        SELECT * FROM debate_sessions WHERE id = $1
    "#;

    pub const GET_DEBATE_SESSIONS_BY_TASK: &str = r#"
        SELECT * FROM debate_sessions WHERE task_id = $1 ORDER BY started_at DESC
    "#;
}

/// SQL queries for knowledge entry operations
pub mod knowledge_queries {
    pub const CREATE_KNOWLEDGE_ENTRY: &str = r#"
        INSERT INTO knowledge_entries (title, content, entry_type, source, confidence_score, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_KNOWLEDGE_ENTRY_BY_ID: &str = r#"
        SELECT * FROM knowledge_entries WHERE id = $1
    "#;

    pub const SEARCH_KNOWLEDGE_ENTRIES: &str = r#"
        SELECT * FROM knowledge_entries 
        WHERE title ILIKE $1 OR content ILIKE $1 
        ORDER BY confidence_score DESC, created_at DESC
    "#;
}

/// SQL queries for performance metrics operations
pub mod performance_queries {
    pub const CREATE_PERFORMANCE_METRIC: &str = r#"
        INSERT INTO performance_metrics (entity_type, entity_id, metric_name, metric_value, recorded_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
    "#;

    pub const GET_METRICS_BY_ENTITY: &str = r#"
        SELECT * FROM performance_metrics 
        WHERE entity_type = $1 AND entity_id = $2 
        ORDER BY recorded_at DESC
    "#;

    pub const GET_AVERAGE_METRICS: &str = r#"
        SELECT metric_name, AVG(metric_value) as avg_value, COUNT(*) as sample_count
        FROM performance_metrics 
        WHERE entity_type = $1 AND entity_id = $2
        GROUP BY metric_name
    "#;
}

/// SQL queries for CAWS compliance operations
pub mod caws_queries {
    pub const CREATE_COMPLIANCE_RECORD: &str = r#"
        INSERT INTO caws_compliance (task_id, compliance_status, quality_gates_passed, violations, recorded_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
    "#;

    pub const GET_COMPLIANCE_BY_TASK: &str = r#"
        SELECT * FROM caws_compliance WHERE task_id = $1 ORDER BY recorded_at DESC
    "#;

    pub const GET_COMPLIANCE_STATS: &str = r#"
        SELECT 
            compliance_status,
            COUNT(*) as count,
            AVG(quality_gates_passed) as avg_gates_passed
        FROM caws_compliance 
        WHERE recorded_at >= $1
        GROUP BY compliance_status
    "#;
}

/// SQL queries for audit trail operations
pub mod audit_queries {
    pub const CREATE_AUDIT_ENTRY: &str = r#"
        INSERT INTO audit_trail (entity_type, entity_id, action, details, user_id, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
    "#;

    pub const GET_AUDIT_TRAIL_BY_ENTITY: &str = r#"
        SELECT * FROM audit_trail 
        WHERE entity_type = $1 AND entity_id = $2 
        ORDER BY timestamp DESC
    "#;

    pub const GET_AUDIT_TRAIL_BY_USER: &str = r#"
        SELECT * FROM audit_trail 
        WHERE user_id = $1 
        ORDER BY timestamp DESC
    "#;
}

/// SQL queries for database health and statistics
pub mod health_queries {
    pub const CHECK_CONNECTIVITY: &str = "SELECT 1";

    pub const GET_TABLE_COUNT: &str = "SELECT COUNT(*) FROM $1";

    pub const GET_DATABASE_STATS: &str = r#"
        SELECT 
            schemaname,
            tablename,
            n_tup_ins as inserts,
            n_tup_upd as updates,
            n_tup_del as deletes
        FROM pg_stat_user_tables
        ORDER BY n_tup_ins + n_tup_upd + n_tup_del DESC
    "#;
}

/// SQL queries for migration operations
pub mod migration_queries {
    pub const CREATE_MIGRATIONS_TABLE: &str = r#"
        CREATE TABLE IF NOT EXISTS applied_migrations (
            id SERIAL PRIMARY KEY,
            migration_id VARCHAR(255) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            checksum VARCHAR(64),
            success BOOLEAN DEFAULT TRUE
        )
    "#;

    pub const GET_APPLIED_MIGRATIONS: &str = r#"
        SELECT migration_id, name, applied_at, checksum, success 
        FROM applied_migrations 
        ORDER BY applied_at
    "#;

    pub const INSERT_MIGRATION_RECORD: &str = r#"
        INSERT INTO applied_migrations (migration_id, name, checksum, success)
        VALUES ($1, $2, $3, $4)
    "#;

    pub const MARK_MIGRATION_FAILED: &str = r#"
        UPDATE applied_migrations 
        SET success = FALSE 
        WHERE migration_id = $1
    "#;
}
