//! Worker Assignment Database Storage
//!
//! Provides database persistence for worker assignments and performance metrics.
//! Integrates with the worker assignment strategy to track assignments and performance.

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

/// Database storage for worker assignments
#[derive(Clone)]
pub struct AssignmentDatabaseStorage {
    pool: Arc<PgPool>,
}

impl AssignmentDatabaseStorage {
    /// Create a new assignment database storage instance
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .context("Failed to create assignment database connection pool")?;

        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .context("Failed to test assignment database connection")?;

        debug!("Assignment database storage initialized successfully");

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Create from existing pool
    pub fn from_pool(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Record a new assignment
    pub async fn record_assignment(
        &self,
        worker_id: Uuid,
        milestone_id: &str,
        plan_id: Option<Uuid>,
        priority: &str,
        resource_allocation: Option<&ResourceAllocation>,
    ) -> Result<Uuid> {
        let assignment_id = Uuid::new_v4();
        let assigned_at = Utc::now();

        let (cpu_cores, memory_mb, disk_mb, network_mbps, time_limit_ms) = match resource_allocation {
            Some(alloc) => (
                Some(alloc.cpu_cores as i32),
                Some(alloc.memory_mb as i32),
                Some(alloc.disk_mb as i32),
                alloc.network_mbps.map(|v| v as f64),
                alloc.time_limit_ms.map(|v| v as i64),
            ),
            None => (None, None, None, None, None),
        };

        sqlx::query(
            r#"
            INSERT INTO worker_assignments (
                id, worker_id, milestone_id, plan_id, assigned_at,
                status, priority, cpu_cores, memory_mb, disk_mb,
                network_mbps, time_limit_ms
            )
            VALUES ($1, $2, $3, $4, $5, 'Assigned', $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(assignment_id)
        .bind(worker_id)
        .bind(milestone_id)
        .bind(plan_id)
        .bind(assigned_at)
        .bind(priority)
        .bind(cpu_cores)
        .bind(memory_mb)
        .bind(disk_mb)
        .bind(network_mbps)
        .bind(time_limit_ms)
        .execute(&*self.pool)
        .await
        .context("Failed to record assignment")?;

        // Create history entry
        sqlx::query(
            r#"
            INSERT INTO assignment_history (
                assignment_id, worker_id, milestone_id, event_type,
                new_status, event_description
            )
            VALUES ($1, $2, $3, 'assigned', 'Assigned', 'Assignment created')
            "#
        )
        .bind(assignment_id)
        .bind(worker_id)
        .bind(milestone_id)
        .execute(&*self.pool)
        .await
        .context("Failed to create assignment history entry")?;

        debug!("Recorded assignment: {} for worker {} to milestone {}", assignment_id, worker_id, milestone_id);
        Ok(assignment_id)
    }

    /// Update assignment status
    pub async fn update_assignment_status(
        &self,
        assignment_id: Uuid,
        status: &str,
        description: Option<&str>,
    ) -> Result<()> {
        sqlx::query("SELECT update_assignment_status($1, $2, $3)")
            .bind(assignment_id)
            .bind(status)
            .bind(description)
        .execute(&*self.pool)
        .await
        .context("Failed to update assignment status")?;

        debug!("Updated assignment {} status to {}", assignment_id, status);
        Ok(())
    }

    /// Store worker performance metrics
    pub async fn store_performance_metrics(
        &self,
        worker_id: Uuid,
        tasks_completed: u64,
        tasks_failed: u64,
        avg_execution_time_ms: f64,
        success_rate: f64,
        performance_score: f64,
    ) -> Result<Uuid> {
        let metric_id = Uuid::new_v4();
        let measurement_time = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO worker_performance_metrics (
                id, worker_id, measurement_time, tasks_completed,
                tasks_failed, avg_execution_time_ms, success_rate, performance_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(metric_id)
        .bind(worker_id)
        .bind(measurement_time)
        .bind(tasks_completed as i64)
        .bind(tasks_failed as i64)
        .bind(avg_execution_time_ms)
        .bind(success_rate)
        .bind(performance_score)
        .execute(&*self.pool)
        .await
        .context("Failed to store performance metrics")?;

        debug!("Stored performance metrics for worker {}: score={}", worker_id, performance_score);
        Ok(metric_id)
    }

    /// Get latest performance metrics for a worker
    pub async fn get_latest_performance(&self, worker_id: Uuid) -> Result<Option<WorkerPerformance>> {
        let row = sqlx::query("SELECT * FROM get_latest_worker_performance($1)")
            .bind(worker_id)
        .fetch_optional(&*self.pool)
        .await
        .context("Failed to get latest performance metrics")?;

        Ok(row.map(|r| WorkerPerformance {
            tasks_completed: r.try_get::<i64, _>("tasks_completed").unwrap_or(0) as u64,
            tasks_failed: r.try_get::<i64, _>("tasks_failed").unwrap_or(0) as u64,
            avg_execution_time_ms: r.try_get("avg_execution_time_ms").unwrap_or(0.0),
            success_rate: r.try_get("success_rate").unwrap_or(0.0),
            performance_score: r.try_get("performance_score").unwrap_or(0.0),
            last_updated: r.try_get("measurement_time").unwrap_or(chrono::Utc::now()),
        }))
    }

    /// Get active assignments for a worker
    pub async fn get_active_assignments(&self, worker_id: Uuid) -> Result<Vec<AssignmentRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, worker_id, milestone_id, plan_id, assigned_at, status,
                   priority, started_at, completed_at, cpu_cores, memory_mb, disk_mb
            FROM worker_assignments
            WHERE worker_id = $1 AND status IN ('Assigned', 'Active')
            ORDER BY assigned_at DESC
            "#
        )
        .bind(worker_id)
        .fetch_all(&*self.pool)
        .await
        .context("Failed to get active assignments")?;

        Ok(rows.into_iter().map(|r| AssignmentRecord {
            id: r.try_get("id").unwrap_or(Uuid::new_v4()),
            worker_id: r.try_get("worker_id").unwrap_or(Uuid::new_v4()),
            milestone_id: r.try_get("milestone_id").unwrap_or_default(),
            plan_id: r.try_get("plan_id").ok(),
            assigned_at: r.try_get("assigned_at").unwrap_or(chrono::Utc::now()),
            status: r.try_get("status").unwrap_or_else(|_| "Unknown".to_string()),
            priority: r.try_get("priority").unwrap_or_else(|_| "Normal".to_string()),
            started_at: r.try_get("started_at").ok(),
            completed_at: r.try_get("completed_at").ok(),
        }).collect())
    }

    /// Get assignment statistics for a worker
    pub async fn get_worker_statistics(&self, worker_id: Uuid) -> Result<WorkerStatistics> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status IN ('Assigned', 'Active')) as active_count,
                COUNT(*) FILTER (WHERE status = 'Completed') as completed_count,
                COUNT(*) FILTER (WHERE status = 'Failed') as failed_count,
                COALESCE(AVG(performance_score), 0.0) as avg_performance_score
            FROM worker_assignments wa
            LEFT JOIN worker_performance_metrics wpm ON wa.worker_id = wpm.worker_id
            WHERE wa.worker_id = $1
            GROUP BY wa.worker_id
            "#
        )
        .bind(worker_id)
        .fetch_optional(&*self.pool)
        .await
        .context("Failed to get worker statistics")?;

        match row {
            Some(r) => Ok(WorkerStatistics {
                active_assignments: r.try_get::<Option<i64>, _>("active_count").unwrap_or(Some(0)).unwrap_or(0) as usize,
                completed_assignments: r.try_get::<Option<i64>, _>("completed_count").unwrap_or(Some(0)).unwrap_or(0) as u64,
                failed_assignments: r.try_get::<Option<i64>, _>("failed_count").unwrap_or(Some(0)).unwrap_or(0) as u64,
                avg_performance_score: r.try_get::<Option<f64>, _>("avg_performance_score").unwrap_or(Some(0.0)).unwrap_or(0.0),
            }),
            None => Ok(WorkerStatistics {
                active_assignments: 0,
                completed_assignments: 0,
                failed_assignments: 0,
                avg_performance_score: 0.0,
            }),
        }
    }

    /// Get database connection pool (for advanced usage)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Resource allocation for assignments
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub cpu_cores: usize,
    pub memory_mb: usize,
    pub disk_mb: usize,
    pub network_mbps: Option<f64>,
    pub time_limit_ms: Option<u64>,
}

/// Worker performance metrics
#[derive(Debug, Clone)]
pub struct WorkerPerformance {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub performance_score: f64,
    pub last_updated: DateTime<Utc>,
}

/// Assignment record
#[derive(Debug, Clone)]
pub struct AssignmentRecord {
    pub id: Uuid,
    pub worker_id: Uuid,
    pub milestone_id: String,
    pub plan_id: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub status: String,
    pub priority: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Worker statistics
#[derive(Debug, Clone)]
pub struct WorkerStatistics {
    pub active_assignments: usize,
    pub completed_assignments: u64,
    pub failed_assignments: u64,
    pub avg_performance_score: f64,
}

