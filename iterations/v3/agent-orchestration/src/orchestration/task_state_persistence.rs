//! Task State Persistence
//!
//! Provides comprehensive task state persistence for resumable tasks and crash recovery.
//! Enables tasks to be paused, resumed, and recovered from interruptions.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use agent_agency_contracts::WorkingSpec;
use agent_agency_contracts::planning_io::ExecutionPlan;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;

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
        Ok(storage.values()
            .filter(|state| matches!(
                state.status,
                ExecutionStateStatus::Paused | ExecutionStateStatus::Crashed | ExecutionStateStatus::Running
            ))
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
        Ok(storage.get(&task_id)
            .map(|state| matches!(
                state.status,
                ExecutionStateStatus::Paused | ExecutionStateStatus::Crashed | ExecutionStateStatus::Running
            ))
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
/// This would integrate with a real database for persistent storage
#[cfg(feature = "database")]
pub struct DatabaseTaskStatePersistence {
    // TODO: Implement comprehensive database connection pool for task state persistence
    //       Currently a placeholder; should implement comprehensive database connection pool integration for persistent task state storage in production environments.
    //
    // COMPLETION CHECKLIST:
    // [ ] Primary functionality implemented
    // [ ] API/data structures defined & stable
    // [ ] Error handling + validation aligned with error taxonomy
    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
    // [ ] Integration tests for external systems/contracts
    // [ ] Documentation: public API + system behavior
    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
    // [ ] Security posture reviewed (inputs, authz, sandboxing)
    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
    // [ ] Configurability and feature flags defined if relevant
    // [ ] Failure-mode cards documented (degradation paths)
    //
    // ACCEPTANCE CRITERIA:
    // - Database connection pool is implemented
    // - Task state persistence uses real database
    // - Connection pooling is efficient and scalable
    // - Database operations are transactional and reliable
    //
    // DEPENDENCIES:
    // - Database connection pool library (Required)
    // - Database schema for task state (Required)
    // - Connection management utilities (Required)
    //
    // ESTIMATED EFFORT: 10-14 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (database persistence functionality)
    // - Change Budget: ~250 LOC
    // - Reviewer Requirements: Database connection pooling and persistence expertise
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl TaskStatePersistence for DatabaseTaskStatePersistence {
    async fn save_state(&self, _state: &TaskExecutionState) -> Result<()> {
        // TODO: Implement database persistence
        // - Serialize state to JSON
        // - Store in database with task_id as key
        // - Update last_updated timestamp
        todo!("Database persistence not yet implemented")
    }

    async fn load_state(&self, _task_id: Uuid) -> Result<Option<TaskExecutionState>> {
        // TODO: Implement database loading
        // - Query database for task_id
        // - Deserialize JSON to TaskExecutionState
        // - Return state if found
        todo!("Database loading not yet implemented")
    }

    async fn list_resumable_tasks(&self) -> Result<Vec<Uuid>> {
        // TODO: Query database for tasks with resumable status
        todo!("Database query not yet implemented")
    }

    async fn delete_state(&self, _task_id: Uuid) -> Result<()> {
        // TODO: Delete from database
        todo!("Database deletion not yet implemented")
    }

    async fn has_resumable_state(&self, _task_id: Uuid) -> Result<bool> {
        // TODO: Check database for resumable state
        todo!("Database check not yet implemented")
    }

    async fn create_checkpoint(&self, _task_id: Uuid, _state: &TaskExecutionState) -> Result<()> {
        // TODO: Create checkpoint in database
        todo!("Database checkpoint not yet implemented")
    }

    async fn list_checkpoints(&self, _task_id: Uuid) -> Result<Vec<DateTime<Utc>>> {
        // TODO: List checkpoints from database
        todo!("Database checkpoint listing not yet implemented")
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
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
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

