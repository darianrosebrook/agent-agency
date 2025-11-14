//! Task State Persistence
//!
//! Provides comprehensive task state persistence for resumable tasks and crash recovery.
//! Enables tasks to be paused, resumed, and recovered from interruptions.
//!
//! @author @darianrosebrook

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::planning_io::ExecutionPlan;
use agent_agency_contracts::WorkingSpec;

use data_infrastructure::simple_client::DatabaseClient;
use sqlx::Row;

/// Complete execution state for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionState {
    /// Task ID
    pub task_id: Uuid,
    /// Working spec
    pub working_spec: WorkingSpec,
    /// Execution plan
    pub execution_plan: Option<ExecutionPlan>,
    /// Completed artifacts
    pub artifacts: Vec<ExecutionArtifacts>,
    /// Current iteration number
    pub current_iteration: u32,
    /// Quality scores per iteration
    pub quality_scores: Vec<f64>,
    /// Current phase/step
    pub current_phase: String,
    /// Progress percentage (0.0-100.0)
    pub progress_percentage: f64,
    /// State status
    pub status: ExecutionStateStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// Checkpoint timestamp
    pub checkpoint_at: Option<DateTime<Utc>>,
    /// Error information (if any)
    pub error: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution state status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStateStatus {
    /// Task is pending
    Pending,
    /// Task is running
    Running,
    /// Task is paused (can be resumed)
    Paused,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task crashed (can be recovered)
    Crashed,
}

/// Trait for task state persistence
#[async_trait::async_trait]
pub trait TaskStatePersistence: Send + Sync {
    /// Save execution state
    async fn save_state(&self, state: &TaskExecutionState) -> Result<()>;

    /// Load execution state for a task
    async fn load_state(&self, task_id: Uuid) -> Result<Option<TaskExecutionState>>;

    /// List all resumable tasks (Paused, Crashed, or Running)
    async fn list_resumable_tasks(&self) -> Result<Vec<Uuid>>;

    /// Delete state for a task
    async fn delete_state(&self, task_id: Uuid) -> Result<()>;

    /// Check if a task has resumable state
    async fn has_resumable_state(&self, task_id: Uuid) -> Result<bool>;

    /// Create a checkpoint for a task
    async fn create_checkpoint(&self, task_id: Uuid, state: &TaskExecutionState) -> Result<()>;

    /// List all checkpoints for a task
    async fn list_checkpoints(&self, task_id: Uuid) -> Result<Vec<DateTime<Utc>>>;
}

/// In-memory task state persistence implementation
pub struct InMemoryTaskStatePersistence {
    /// State storage (task_id -> TaskExecutionState)
    state_storage: Arc<RwLock<HashMap<Uuid, TaskExecutionState>>>,
    /// Checkpoint storage (task_id -> Vec<checkpoint_timestamp>)
    checkpoint_storage: Arc<RwLock<HashMap<Uuid, Vec<DateTime<Utc>>>>>,
}

impl InMemoryTaskStatePersistence {
    pub fn new() -> Self {
        Self {
            state_storage: Arc::new(RwLock::new(HashMap::new())),
            checkpoint_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryTaskStatePersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TaskStatePersistence for InMemoryTaskStatePersistence {
    async fn save_state(&self, state: &TaskExecutionState) -> Result<()> {
        let mut storage = self.state_storage.write().await;
        let mut state_to_save = state.clone();
        state_to_save.last_updated = Utc::now();
        storage.insert(state.task_id, state_to_save);
        Ok(())
    }

    async fn load_state(&self, task_id: Uuid) -> Result<Option<TaskExecutionState>> {
        let storage = self.state_storage.read().await;
        Ok(storage.get(&task_id).cloned())
    }

    async fn list_resumable_tasks(&self) -> Result<Vec<Uuid>> {
        let storage = self.state_storage.read().await;
        Ok(storage
            .values()
            .filter(|state| {
                matches!(
                    state.status,
                    ExecutionStateStatus::Paused
                        | ExecutionStateStatus::Crashed
                        | ExecutionStateStatus::Running
                )
            })
            .map(|state| state.task_id)
            .collect())
    }

    async fn delete_state(&self, task_id: Uuid) -> Result<()> {
        let mut storage = self.state_storage.write().await;
        storage.remove(&task_id);

        let mut checkpoints = self.checkpoint_storage.write().await;
        checkpoints.remove(&task_id);

        Ok(())
    }

    async fn has_resumable_state(&self, task_id: Uuid) -> Result<bool> {
        let storage = self.state_storage.read().await;
        Ok(storage
            .get(&task_id)
            .map(|state| {
                matches!(
                    state.status,
                    ExecutionStateStatus::Paused
                        | ExecutionStateStatus::Crashed
                        | ExecutionStateStatus::Running
                )
            })
            .unwrap_or(false))
    }

    async fn create_checkpoint(&self, task_id: Uuid, state: &TaskExecutionState) -> Result<()> {
        // Save state
        self.save_state(state).await?;

        // Record checkpoint timestamp
        let mut checkpoints = self.checkpoint_storage.write().await;
        let checkpoint_list = checkpoints.entry(task_id).or_insert_with(Vec::new);
        checkpoint_list.push(Utc::now());

        // Update state checkpoint timestamp
        let mut storage = self.state_storage.write().await;
        if let Some(state) = storage.get_mut(&task_id) {
            state.checkpoint_at = Some(Utc::now());
        }

        Ok(())
    }

    async fn list_checkpoints(&self, task_id: Uuid) -> Result<Vec<DateTime<Utc>>> {
        let checkpoints = self.checkpoint_storage.read().await;
        Ok(checkpoints.get(&task_id).cloned().unwrap_or_default())
    }
}

/// Database-backed task state persistence (for production use)
/// Provides persistent storage for task execution state enabling resumption and recovery
pub struct DatabaseTaskStatePersistence {
    /// Database client for persistence operations
    db_client: Arc<DatabaseClient>,
}

impl DatabaseTaskStatePersistence {
    /// Create a new database-backed task state persistence instance
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    /// Convert ExecutionStateStatus to database string representation
    fn status_to_string(status: &ExecutionStateStatus) -> String {
        match status {
            ExecutionStateStatus::Pending => "pending".to_string(),
            ExecutionStateStatus::Running => "running".to_string(),
            ExecutionStateStatus::Paused => "paused".to_string(),
            ExecutionStateStatus::Completed => "completed".to_string(),
            ExecutionStateStatus::Failed => "failed".to_string(),
            ExecutionStateStatus::Cancelled => "cancelled".to_string(),
            ExecutionStateStatus::Crashed => "crashed".to_string(),
        }
    }

    /// Convert database string to ExecutionStateStatus
    #[allow(dead_code)]
    fn string_to_status(s: &str) -> Result<ExecutionStateStatus> {
        match s {
            "pending" => Ok(ExecutionStateStatus::Pending),
            "running" => Ok(ExecutionStateStatus::Running),
            "paused" => Ok(ExecutionStateStatus::Paused),
            "completed" => Ok(ExecutionStateStatus::Completed),
            "failed" => Ok(ExecutionStateStatus::Failed),
            "cancelled" => Ok(ExecutionStateStatus::Cancelled),
            "crashed" => Ok(ExecutionStateStatus::Crashed),
            _ => Err(anyhow::anyhow!("Invalid status: {}", s)),
        }
    }
}

#[async_trait::async_trait]
impl TaskStatePersistence for DatabaseTaskStatePersistence {
    async fn save_state(&self, state: &TaskExecutionState) -> Result<()> {
        debug!("Saving task execution state for task {}", state.task_id);

        // Serialize state to JSON
        let state_json = serde_json::to_value(state)
            .context("Failed to serialize TaskExecutionState to JSON")?;

        let status_str = Self::status_to_string(&state.status);

        // Upsert state (insert or update)
        // Use sqlx directly since DatabaseClient::execute doesn't support parameterized queries
        sqlx::query(
            r#"
            INSERT INTO task_execution_states (task_id, state_data, status, checkpoint_at, last_updated)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (task_id) DO UPDATE SET
                state_data = EXCLUDED.state_data,
                status = EXCLUDED.status,
                checkpoint_at = EXCLUDED.checkpoint_at,
                last_updated = EXCLUDED.last_updated
            "#,
        )
        .bind(state.task_id)
        .bind(&state_json)
        .bind(&status_str)
        .bind(state.checkpoint_at)
        .bind(Utc::now())
        .execute(self.db_client.pool())
        .await
        .context("Failed to save task execution state to database")?;

        debug!(
            "Successfully saved task execution state for task {}",
            state.task_id
        );
        Ok(())
    }

    async fn load_state(&self, task_id: Uuid) -> Result<Option<TaskExecutionState>> {
        debug!("Loading task execution state for task {}", task_id);

        // Use sqlx directly since DatabaseClient::query_one doesn't support parameterized queries properly
        let row = sqlx::query(
            r#"
            SELECT state_data, status, checkpoint_at, last_updated
            FROM task_execution_states
            WHERE task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(self.db_client.pool())
        .await
        .context("Failed to query task execution state from database")?;

        match row {
            Some(row) => {
                let state_json: serde_json::Value = row
                    .try_get("state_data")
                    .context("Failed to get state_data from database row")?;

                // Deserialize JSON to TaskExecutionState
                let state: TaskExecutionState = serde_json::from_value(state_json)
                    .context("Failed to deserialize TaskExecutionState from JSON")?;

                debug!(
                    "Successfully loaded task execution state for task {}",
                    task_id
                );
                Ok(Some(state))
            }
            None => {
                debug!("No task execution state found for task {}", task_id);
                Ok(None)
            }
        }
    }

    async fn list_resumable_tasks(&self) -> Result<Vec<Uuid>> {
        debug!("Listing resumable tasks");

        // Use sqlx directly since DatabaseClient::query doesn't support parameterized queries properly
        let rows = sqlx::query(
            r#"
            SELECT task_id
            FROM task_execution_states
            WHERE status IN ('paused', 'crashed', 'running')
            ORDER BY last_updated DESC
            "#,
        )
        .fetch_all(self.db_client.pool())
        .await
        .context("Failed to query resumable tasks from database")?;

        let mut task_ids = Vec::new();
        for row in rows {
            let task_id: Uuid = row
                .try_get("task_id")
                .context("Failed to get task_id from database row")?;
            task_ids.push(task_id);
        }

        debug!("Found {} resumable tasks", task_ids.len());
        Ok(task_ids)
    }

    async fn delete_state(&self, task_id: Uuid) -> Result<()> {
        debug!("Deleting task execution state for task {}", task_id);

        // Delete checkpoints first (foreign key constraint)
        // Use sqlx directly since DatabaseClient::execute doesn't support parameterized queries
        sqlx::query("DELETE FROM task_state_checkpoints WHERE task_id = $1")
            .bind(task_id)
            .execute(self.db_client.pool())
            .await
            .context("Failed to delete task state checkpoints")?;

        // Delete state
        sqlx::query("DELETE FROM task_execution_states WHERE task_id = $1")
            .bind(task_id)
            .execute(self.db_client.pool())
            .await
            .context("Failed to delete task execution state from database")?;

        debug!(
            "Successfully deleted task execution state for task {}",
            task_id
        );
        Ok(())
    }

    async fn has_resumable_state(&self, task_id: Uuid) -> Result<bool> {
        debug!("Checking if task {} has resumable state", task_id);

        // Use sqlx directly since DatabaseClient::query_one doesn't support parameterized queries properly
        let row = sqlx::query(
            r#"
            SELECT status
            FROM task_execution_states
            WHERE task_id = $1 AND status IN ('paused', 'crashed', 'running')
            "#,
        )
        .bind(task_id)
        .fetch_optional(self.db_client.pool())
        .await
        .context("Failed to check resumable state in database")?;

        let has_resumable = row.is_some();
        debug!("Task {} has resumable state: {}", task_id, has_resumable);
        Ok(has_resumable)
    }

    async fn create_checkpoint(&self, task_id: Uuid, state: &TaskExecutionState) -> Result<()> {
        debug!("Creating checkpoint for task {}", task_id);

        // Save state first (this will update checkpoint_at)
        let mut state_with_checkpoint = state.clone();
        state_with_checkpoint.checkpoint_at = Some(Utc::now());
        self.save_state(&state_with_checkpoint).await?;

        // Serialize state for checkpoint storage
        let state_json = serde_json::to_value(state)
            .context("Failed to serialize TaskExecutionState to JSON for checkpoint")?;

        // Create checkpoint record
        // Use sqlx directly since DatabaseClient::execute doesn't support parameterized queries
        sqlx::query(
            r#"
            INSERT INTO task_state_checkpoints (task_id, checkpoint_timestamp, state_data)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(task_id)
        .bind(Utc::now())
        .bind(&state_json)
        .execute(self.db_client.pool())
        .await
        .context("Failed to create checkpoint in database")?;

        debug!("Successfully created checkpoint for task {}", task_id);
        Ok(())
    }

    async fn list_checkpoints(&self, task_id: Uuid) -> Result<Vec<DateTime<Utc>>> {
        debug!("Listing checkpoints for task {}", task_id);

        // Use sqlx directly since DatabaseClient::query doesn't support parameterized queries properly
        let rows = sqlx::query(
            r#"
            SELECT checkpoint_timestamp
            FROM task_state_checkpoints
            WHERE task_id = $1
            ORDER BY checkpoint_timestamp DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(self.db_client.pool())
        .await
        .context("Failed to query checkpoints from database")?;

        let mut checkpoints = Vec::new();
        for row in rows {
            let timestamp: DateTime<Utc> = row
                .try_get("checkpoint_timestamp")
                .context("Failed to get checkpoint_timestamp from database row")?;
            checkpoints.push(timestamp);
        }

        debug!(
            "Found {} checkpoints for task {}",
            checkpoints.len(),
            task_id
        );
        Ok(checkpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_persistence_basic_operations() {
        let persistence = InMemoryTaskStatePersistence::new();
        let task_id = Uuid::new_v4();

        // Create a test state
        let state = TaskExecutionState {
            task_id,
            working_spec: agent_agency_contracts::WorkingSpec {
                version: "1.0".to_string(),
                id: "test-spec".to_string(),
                title: "Test Spec".to_string(),
                description: "Test description".to_string(),
                goals: vec![],
                risk_tier: 2,
                constraints: agent_agency_contracts::WorkingSpecConstraints {
                    max_duration_minutes: None,
                    max_iterations: None,
                    budget_limits: None,
                    scope_restrictions: None,
                },
                acceptance_criteria: vec![],
                test_plan: agent_agency_contracts::TestPlan {
                    unit_tests: vec![],
                    integration_tests: vec![],
                    e2e_scenarios: vec![],
                    coverage_targets: None,
                },
                rollback_plan: agent_agency_contracts::RollbackPlan::default(),
                context: agent_agency_contracts::WorkingSpecContext {
                    workspace_root: "/tmp".to_string(),
                    git_branch: "main".to_string(),
                    recent_changes: vec![],
                    dependencies: HashMap::new(),
                    environment: agent_agency_contracts::task_request::Environment::Development,
                },
                non_functional_requirements: None,
                validation_results: None,
                quality_gates: None,
                scope: vec![],
                metadata: None,
                milestones: vec![],
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode:
                        agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                file_changes: vec![],
                coverage_targets: None,
                overview: "Test overview".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            execution_plan: None,
            artifacts: Vec::new(),
            current_iteration: 1,
            quality_scores: vec![0.8],
            current_phase: "execution".to_string(),
            progress_percentage: 50.0,
            status: ExecutionStateStatus::Running,
            created_at: Utc::now(),
            last_updated: Utc::now(),
            checkpoint_at: None,
            error: None,
            metadata: HashMap::new(),
        };

        // Save state
        assert!(persistence.save_state(&state).await.is_ok());

        // Load state
        let loaded = persistence.load_state(task_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().task_id, task_id);

        // Check resumable
        assert!(persistence.has_resumable_state(task_id).await.unwrap());

        // List resumable tasks
        let resumable = persistence.list_resumable_tasks().await.unwrap();
        assert!(resumable.contains(&task_id));

        // Create checkpoint
        assert!(persistence.create_checkpoint(task_id, &state).await.is_ok());

        // List checkpoints
        let checkpoints = persistence.list_checkpoints(task_id).await.unwrap();
        assert_eq!(checkpoints.len(), 1);

        // Delete state
        assert!(persistence.delete_state(task_id).await.is_ok());
        assert!(!persistence.has_resumable_state(task_id).await.unwrap());
    }
}
