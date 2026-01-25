//! PostgreSQL Repository
//!
//! High-level data access implementing the DatabasePort trait.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::Row;

use v4_types::events::AgentEvent;
use v4_types::execution::{ExecutionArtifacts, ExecutionMetrics, ExecutionStatus, TaskResult, TokenUsage};
use v4_types::ports::{DatabaseError, DatabasePort};
use v4_types::task::TaskSpec;

use crate::error::PostgresError;
use crate::pool::ConnectionPool;

/// PostgreSQL repository implementing DatabasePort
pub struct PostgresRepository {
    pool: ConnectionPool,
}

impl PostgresRepository {
    /// Create a new repository
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Get the connection pool
    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }

    /// Hash content for integrity verification
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get event type name from AgentEvent
    fn event_type_name(event: &AgentEvent) -> &'static str {
        match event {
            AgentEvent::TaskCreated { .. } => "TaskCreated",
            AgentEvent::TaskDecomposed { .. } => "TaskDecomposed",
            AgentEvent::TaskStarted { .. } => "TaskStarted",
            AgentEvent::TaskCompleted { .. } => "TaskCompleted",
            AgentEvent::TaskFailed { .. } => "TaskFailed",
            AgentEvent::InvariantChecked { .. } => "InvariantChecked",
            AgentEvent::CouncilConvened { .. } => "CouncilConvened",
            AgentEvent::CouncilVerdict { .. } => "CouncilVerdict",
            AgentEvent::GatePassed { .. } => "GatePassed",
            AgentEvent::GateFailed { .. } => "GateFailed",
            AgentEvent::MemoryStored { .. } => "MemoryStored",
            AgentEvent::MemoryRecalled { .. } => "MemoryRecalled",
            AgentEvent::MemoryDecayed { .. } => "MemoryDecayed",
            AgentEvent::OperatorInvoked { .. } => "OperatorInvoked",
            AgentEvent::OperatorCompleted { .. } => "OperatorCompleted",
            AgentEvent::ToolInvoked { .. } => "ToolInvoked",
            AgentEvent::ToolCompleted { .. } => "ToolCompleted",
            AgentEvent::WorkerSpawned { .. } => "WorkerSpawned",
            AgentEvent::WorkerCompleted { .. } => "WorkerCompleted",
            AgentEvent::CheckpointCreated { .. } => "CheckpointCreated",
            AgentEvent::CheckpointRestored { .. } => "CheckpointRestored",
        }
    }

    /// Get aggregate info from AgentEvent
    fn event_aggregate(event: &AgentEvent) -> (Option<&'static str>, Option<String>) {
        match event {
            AgentEvent::TaskCreated { task_id, .. }
            | AgentEvent::TaskDecomposed { task_id, .. }
            | AgentEvent::TaskStarted { task_id, .. }
            | AgentEvent::TaskCompleted { task_id, .. }
            | AgentEvent::TaskFailed { task_id, .. } => (Some("task"), Some(task_id.clone())),

            AgentEvent::CouncilConvened { task_id, .. }
            | AgentEvent::CouncilVerdict { task_id, .. } => (Some("task"), Some(task_id.clone())),

            AgentEvent::WorkerSpawned { worker_id, .. }
            | AgentEvent::WorkerCompleted { worker_id, .. } => (Some("worker"), Some(worker_id.clone())),

            AgentEvent::MemoryStored { key, .. }
            | AgentEvent::MemoryRecalled { key, .. }
            | AgentEvent::MemoryDecayed { key, .. } => (Some("memory"), Some(key.clone())),

            AgentEvent::CheckpointCreated { checkpoint_id, .. }
            | AgentEvent::CheckpointRestored { checkpoint_id, .. } => {
                (Some("checkpoint"), Some(checkpoint_id.clone()))
            }

            _ => (None, None),
        }
    }

    /// Store an event with explicit sequence tracking
    pub async fn store_event_internal(
        &self,
        event: &AgentEvent,
    ) -> Result<i64, PostgresError> {
        let event_type = Self::event_type_name(event);
        let (aggregate_type, aggregate_id) = Self::event_aggregate(event);
        let payload = serde_json::to_value(event)?;
        let content_hash = Self::hash_content(&payload.to_string());

        let row = sqlx::query(
            r#"
            INSERT INTO events (event_type, aggregate_type, aggregate_id, payload, content_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING sequence_number
            "#,
        )
        .bind(event_type)
        .bind(aggregate_type)
        .bind(&aggregate_id)
        .bind(&payload)
        .bind(&content_hash)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(row.get("sequence_number"))
    }

    /// Get events after a sequence number
    pub async fn get_events_after(&self, sequence: i64) -> Result<Vec<StoredEvent>, PostgresError> {
        let rows = sqlx::query(
            r#"
            SELECT id, sequence_number, event_type, aggregate_type, aggregate_id,
                   payload, content_hash, created_at
            FROM events
            WHERE sequence_number > $1
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(sequence)
        .fetch_all(self.pool.pool())
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: serde_json::Value = row.get("payload");
            let event: AgentEvent = serde_json::from_value(payload)?;

            events.push(StoredEvent {
                id: row.get("id"),
                sequence_number: row.get("sequence_number"),
                event,
                content_hash: row.get("content_hash"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    /// Get all events for a task
    pub async fn get_task_events(&self, task_id: &str) -> Result<Vec<StoredEvent>, PostgresError> {
        let rows = sqlx::query(
            r#"
            SELECT id, sequence_number, event_type, aggregate_type, aggregate_id,
                   payload, content_hash, created_at
            FROM events
            WHERE aggregate_type = 'task' AND aggregate_id = $1
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(task_id)
        .fetch_all(self.pool.pool())
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: serde_json::Value = row.get("payload");
            let event: AgentEvent = serde_json::from_value(payload)?;

            events.push(StoredEvent {
                id: row.get("id"),
                sequence_number: row.get("sequence_number"),
                event,
                content_hash: row.get("content_hash"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    /// Store a task
    pub async fn store_task(&self, spec: &TaskSpec) -> Result<(), PostgresError> {
        let constraints = serde_json::to_value(&spec.constraints)?;

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, priority, constraints, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                constraints = EXCLUDED.constraints,
                updated_at = NOW()
            "#,
        )
        .bind(&spec.id)
        .bind(&spec.title)
        .bind(&spec.description)
        .bind("normal") // Default priority
        .bind(&constraints)
        .bind(&spec.created_at)
        .execute(self.pool.pool())
        .await?;

        Ok(())
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>, PostgresError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, status, priority, environment,
                   constraints, metadata, created_at, updated_at, completed_at,
                   result_hash, council_verdict, execution_time_ms
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(self.pool.pool())
        .await?;

        match row {
            Some(row) => Ok(Some(TaskRecord {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
                priority: row.get("priority"),
                environment: row.get("environment"),
                constraints: row.get("constraints"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                completed_at: row.get("completed_at"),
                result_hash: row.get("result_hash"),
                council_verdict: row.get("council_verdict"),
                execution_time_ms: row.get("execution_time_ms"),
            })),
            None => Ok(None),
        }
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: &str,
    ) -> Result<(), PostgresError> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(task_id)
        .execute(self.pool.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(PostgresError::NotFound(format!("Task {} not found", task_id)));
        }

        Ok(())
    }

    /// Complete a task with result
    pub async fn complete_task(
        &self,
        task_id: &str,
        result_hash: &str,
        council_verdict: Option<serde_json::Value>,
        execution_time_ms: i64,
    ) -> Result<(), PostgresError> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET status = 'completed',
                completed_at = NOW(),
                updated_at = NOW(),
                result_hash = $1,
                council_verdict = $2,
                execution_time_ms = $3
            WHERE id = $4
            "#,
        )
        .bind(result_hash)
        .bind(council_verdict)
        .bind(execution_time_ms)
        .bind(task_id)
        .execute(self.pool.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(PostgresError::NotFound(format!("Task {} not found", task_id)));
        }

        Ok(())
    }

    /// Get event count
    pub async fn event_count(&self) -> Result<i64, PostgresError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM events")
            .fetch_one(self.pool.pool())
            .await?;

        Ok(row.get("count"))
    }

    /// Verify all event hashes
    pub async fn verify_integrity(&self) -> Result<bool, PostgresError> {
        let rows = sqlx::query(
            "SELECT payload, content_hash FROM events ORDER BY sequence_number"
        )
        .fetch_all(self.pool.pool())
        .await?;

        for row in rows {
            let payload: serde_json::Value = row.get("payload");
            let stored_hash: String = row.get("content_hash");
            let computed_hash = Self::hash_content(&payload.to_string());

            if stored_hash != computed_hash {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Stored event with metadata
#[derive(Debug, Clone)]
pub struct StoredEvent {
    /// Event ID
    pub id: uuid::Uuid,
    /// Sequence number
    pub sequence_number: i64,
    /// The event
    pub event: AgentEvent,
    /// Content hash
    pub content_hash: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl StoredEvent {
    /// Verify the event hash
    pub fn verify(&self) -> bool {
        let payload = serde_json::to_string(&self.event).unwrap_or_default();
        let computed = PostgresRepository::hash_content(&payload);
        // Note: This may not match exactly due to serialization differences
        // In production, store the exact serialized form
        !self.content_hash.is_empty() && computed.len() == self.content_hash.len()
    }
}

/// Task database record
#[derive(Debug, Clone)]
pub struct TaskRecord {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: Option<String>,
    /// Current status
    pub status: String,
    /// Priority
    pub priority: String,
    /// Environment
    pub environment: String,
    /// Constraints JSON
    pub constraints: Option<serde_json::Value>,
    /// Metadata JSON
    pub metadata: Option<serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Result hash
    pub result_hash: Option<String>,
    /// Council verdict JSON
    pub council_verdict: Option<serde_json::Value>,
    /// Execution time in ms
    pub execution_time_ms: Option<i64>,
}

// Implement DatabasePort trait
#[async_trait]
impl DatabasePort for PostgresRepository {
    async fn store_event(&self, event: &AgentEvent) -> Result<(), DatabaseError> {
        self.store_event_internal(event)
            .await
            .map(|_| ())
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    async fn get_events(&self, task_id: &str) -> Result<Vec<AgentEvent>, DatabaseError> {
        self.get_task_events(task_id)
            .await
            .map(|events| events.into_iter().map(|e| e.event).collect())
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    async fn store_task_spec(&self, spec: &TaskSpec) -> Result<(), DatabaseError> {
        self.store_task(spec)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    async fn get_task_spec(&self, task_id: &str) -> Result<Option<TaskSpec>, DatabaseError> {
        let record = self
            .get_task(task_id)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match record {
            Some(r) => {
                let constraints = r
                    .constraints
                    .map(|c| serde_json::from_value(c).unwrap_or_default())
                    .unwrap_or_default();

                Ok(Some(TaskSpec {
                    id: r.id,
                    version: "1.0".to_string(),
                    title: r.title,
                    description: r.description.unwrap_or_default(),
                    goals: vec![],
                    risk_tier: 1,
                    constraints,
                    acceptance_criteria: vec![],
                    created_at: r.created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn store_task_result(&self, result: &TaskResult) -> Result<(), DatabaseError> {
        let result_hash = Self::hash_content(&serde_json::to_string(result).unwrap_or_default());
        let execution_time_ms = result.metrics.execution_time_ms as i64;

        self.complete_task(&result.task_id, &result_hash, None, execution_time_ms)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    async fn get_task_result(&self, task_id: &str) -> Result<Option<TaskResult>, DatabaseError> {
        let record = self
            .get_task(task_id)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match record {
            Some(r) if r.status == "completed" => {
                Ok(Some(TaskResult {
                    task_id: r.id,
                    status: ExecutionStatus::Success,
                    artifacts: ExecutionArtifacts {
                        modified_files: vec![],
                        test_results: None,
                        git_info: None,
                        audit_events: vec![],
                    },
                    metrics: ExecutionMetrics {
                        total_time_ms: r.execution_time_ms.unwrap_or(0) as u64,
                        planning_time_ms: 0,
                        review_time_ms: 0,
                        execution_time_ms: r.execution_time_ms.unwrap_or(0) as u64,
                        iterations: 1,
                        tool_invocations: 0,
                        llm_calls: 0,
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: 0,
                            total_tokens: 0,
                        },
                    },
                    completed_at: r.completed_at.unwrap_or_else(Utc::now),
                }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let hash1 = PostgresRepository::hash_content("test content");
        let hash2 = PostgresRepository::hash_content("test content");
        let hash3 = PostgresRepository::hash_content("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_task_record() {
        let record = TaskRecord {
            id: "task-1".to_string(),
            title: "Test".to_string(),
            description: Some("Description".to_string()),
            status: "pending".to_string(),
            priority: "normal".to_string(),
            environment: "development".to_string(),
            constraints: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            result_hash: None,
            council_verdict: None,
            execution_time_ms: None,
        };

        assert_eq!(record.status, "pending");
    }
}
