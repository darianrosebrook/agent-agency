//! Autonomous Executor for V3
//!
//! Coordinates worker execution with real-time progress tracking,
//! artifact collection, and provenance capture for autonomous task execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::manager::WorkerPoolManager;
use crate::types::{TaskSpec, WorkerAssignment, TaskExecutionResult};
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig};

use super::super::orchestration::planning::types::{WorkingSpec, ExecutionArtifacts, ExecutionEvent};
use super::super::orchestration::caws_runtime::{CawsRuntimeValidator, DiffStats, TaskDescriptor};

/// Configuration for autonomous execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutonomousExecutorConfig {
    /// Maximum execution time per task (seconds)
    pub max_execution_time_seconds: u64,
    /// Progress reporting interval (seconds)
    pub progress_report_interval_seconds: u64,
    /// Enable detailed artifact collection
    pub enable_detailed_artifacts: bool,
    /// Maximum artifacts size per task (MB)
    pub max_artifacts_size_mb: u64,
    /// Enable real-time event streaming
    pub enable_event_streaming: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u64,
    /// Circuit breaker reset timeout (seconds)
    pub circuit_breaker_reset_timeout_seconds: u64,
}

/// Autonomous executor that coordinates worker execution with tracking
pub struct AutonomousExecutor {
    worker_manager: Arc<WorkerPoolManager>,
    validator: Arc<dyn CawsRuntimeValidator>,
    config: AutonomousExecutorConfig,
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl AutonomousExecutor {
    pub fn new(
        worker_manager: Arc<WorkerPoolManager>,
        validator: Arc<dyn CawsRuntimeValidator>,
        config: AutonomousExecutorConfig,
    ) -> (Self, mpsc::UnboundedReceiver<ExecutionEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Create circuit breaker for task execution protection
        let circuit_breaker_config = CircuitBreakerConfig {
            name: Some("autonomous-executor".to_string()),
            failure_threshold: config.circuit_breaker_failure_threshold,
            success_threshold: 3,
            reset_timeout_ms: (config.circuit_breaker_reset_timeout_seconds * 1000) as u64,
            operation_timeout_ms: (config.max_execution_time_seconds * 1000) as u64,
            monitoring_window_ms: 60000, // 1 minute monitoring window
        };

        let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_breaker_config));

        (
            Self {
                worker_manager,
                validator,
                config,
                event_sender,
                circuit_breaker,
            },
            event_receiver,
        )
    }

    /// Execute a working spec autonomously with full tracking
    pub async fn execute_with_tracking(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<ExecutionResult, AutonomousExecutionError> {
        tracing::info!("Starting autonomous execution for task: {} (spec: {})",
            task_id, working_spec.id);

        // Convert working spec to task spec
        let task_spec = self.working_spec_to_task_spec(working_spec, task_id)?;

        // Send initial event
        self.send_event(ExecutionEvent::ExecutionStarted {
            task_id,
            working_spec_id: working_spec.id.clone(),
            timestamp: Utc::now(),
        }).await;

        // Execute the task
        let execution_start = std::time::Instant::now();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.max_execution_time_seconds),
            self.execute_task_with_progress(&task_spec, working_spec),
        ).await;

        let execution_duration = execution_start.elapsed();

        match result {
            Ok(execution_result) => {
                let final_result = ExecutionResult {
                    task_id,
                    working_spec_id: working_spec.id.clone(),
                    success: execution_result.success,
                    artifacts: execution_result.artifacts,
                    error_message: execution_result.error_message,
                    execution_time_ms: execution_duration.as_millis() as u64,
                    completed_at: Utc::now(),
                };

                self.send_event(ExecutionEvent::ExecutionCompleted {
                    task_id,
                    success: final_result.success,
                    artifacts_summary: self.summarize_artifacts(&final_result.artifacts),
                    execution_time_ms: final_result.execution_time_ms,
                    timestamp: final_result.completed_at,
                }).await;

                Ok(final_result)
            }
            Err(_) => {
                let error_msg = format!("Execution timed out after {} seconds",
                    self.config.max_execution_time_seconds);

                let final_result = ExecutionResult {
                    task_id,
                    working_spec_id: working_spec.id.clone(),
                    success: false,
                    artifacts: ExecutionArtifacts::default(),
                    error_message: Some(error_msg.clone()),
                    execution_time_ms: execution_duration.as_millis() as u64,
                    completed_at: Utc::now(),
                };

                self.send_event(ExecutionEvent::ExecutionFailed {
                    task_id,
                    error: error_msg,
                    timestamp: Utc::now(),
                }).await;

                Ok(final_result)
            }
        }
    }

    /// Execute task with progress tracking and artifact collection
    async fn execute_task_with_progress(
        &self,
        task_spec: &TaskSpec,
        working_spec: &WorkingSpec,
    ) -> Result<ExecutionResultInternal, AutonomousExecutionError> {
        let mut artifacts = ExecutionArtifacts {
            id: Uuid::new_v4(),
            task_id: task_spec.id,
            code_changes: Vec::new(),
            test_results: Default::default(),
            coverage: Default::default(),
            mutation: Default::default(),
            lint: Default::default(),
            types: Default::default(),
            provenance: Default::default(),
            generated_at: Utc::now(),
        };

        // Assign workers based on task requirements
        self.send_event(ExecutionEvent::WorkerAssignmentStarted {
            task_id: task_spec.id,
            timestamp: Utc::now(),
        }).await;

        let assignment = self.worker_manager.execute_task(
            task_spec.clone(),
            Some(&self.circuit_breaker),
        ).await?;

        self.send_event(ExecutionEvent::WorkerAssigned {
            task_id: task_spec.id,
            worker_id: assignment.worker_id,
            worker_type: assignment.worker_type.clone(),
            timestamp: Utc::now(),
        }).await;

        // Execute with progress reporting
        let progress_interval = std::time::Duration::from_secs(
            self.config.progress_report_interval_seconds
        );

        let mut last_progress = std::time::Instant::now();
        let mut success = true;
        let mut error_message = None;

        // TODO: Implement actual worker execution loop with progress tracking
        // This is a simplified implementation - real implementation would:
        // 1. Monitor worker execution in real-time
        // 2. Collect artifacts as they are produced
        // 3. Run CAWS validation checkpoints
        // 4. Handle partial failures gracefully

        // Simulate execution phases for now
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        self.send_event(ExecutionEvent::ExecutionPhaseStarted {
            task_id: task_spec.id,
            phase: "code_generation".to_string(),
            description: "Generating implementation code".to_string(),
            timestamp: Utc::now(),
        }).await;

        // Simulate code generation
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        self.send_event(ExecutionEvent::ArtifactProduced {
            task_id: task_spec.id,
            artifact_type: "code".to_string(),
            artifact_path: "src/generated.rs".to_string(),
            size_bytes: 1024,
            timestamp: Utc::now(),
        }).await;

        self.send_event(ExecutionEvent::ExecutionPhaseCompleted {
            task_id: task_spec.id,
            phase: "code_generation".to_string(),
            success: true,
            timestamp: Utc::now(),
        }).await;

        // Test generation phase
        self.send_event(ExecutionEvent::ExecutionPhaseStarted {
            task_id: task_spec.id,
            phase: "test_generation".to_string(),
            description: "Generating comprehensive tests".to_string(),
            timestamp: Utc::now(),
        }).await;

        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        self.send_event(ExecutionEvent::ArtifactProduced {
            task_id: task_spec.id,
            artifact_type: "test".to_string(),
            artifact_path: "tests/generated_test.rs".to_string(),
            size_bytes: 512,
            timestamp: Utc::now(),
        }).await;

        self.send_event(ExecutionEvent::ExecutionPhaseCompleted {
            task_id: task_spec.id,
            phase: "test_generation".to_string(),
            success: true,
            timestamp: Utc::now(),
        }).await;

        // Quality validation phase
        self.send_event(ExecutionEvent::ExecutionPhaseStarted {
            task_id: task_spec.id,
            phase: "quality_validation".to_string(),
            description: "Running quality gates and validation".to_string(),
            timestamp: Utc::now(),
        }).await;

        // Run CAWS validation
        let validation_result = self.validate_execution_checkpoint(
            working_spec,
            &artifacts,
            task_spec.id,
        ).await?;

        if !validation_result.violations.is_empty() {
            success = false;
            error_message = Some(format!("CAWS validation failed: {} violations",
                validation_result.violations.len()));
        }

        self.send_event(ExecutionEvent::QualityCheckCompleted {
            task_id: task_spec.id,
            check_type: "caws_validation".to_string(),
            passed: validation_result.violations.is_empty(),
            violations_count: validation_result.violations.len(),
            timestamp: Utc::now(),
        }).await;

        self.send_event(ExecutionEvent::ExecutionPhaseCompleted {
            task_id: task_spec.id,
            phase: "quality_validation".to_string(),
            success,
            timestamp: Utc::now(),
        }).await;

        Ok(ExecutionResultInternal {
            success,
            artifacts,
            error_message,
        })
    }

    /// Validate execution at a checkpoint
    async fn validate_execution_checkpoint(
        &self,
        working_spec: &WorkingSpec,
        artifacts: &ExecutionArtifacts,
        task_id: Uuid,
    ) -> Result<super::super::orchestration::caws_runtime::ValidationResult, AutonomousExecutionError> {
        // Create a mock task descriptor for validation
        let task_desc = TaskDescriptor {
            task_id: format!("checkpoint-{}", task_id),
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.included.clone())
                .unwrap_or_default(),
            risk_tier: working_spec.risk_tier as u8,
            acceptance: Some(working_spec.acceptance_criteria.iter()
                .map(|ac| format!("Given {}, When {}, Then {}", ac.given, ac.when, ac.then))
                .collect()),
            metadata: Some(serde_json::json!({
                "working_spec_id": working_spec.id,
                "artifacts_id": artifacts.id,
            })),
        };

        // Create diff stats based on artifacts
        let diff_stats = DiffStats {
            files_changed: artifacts.code_changes.len() as u32,
            lines_changed: artifacts.code_changes.iter()
                .map(|c| c.lines_added + c.lines_removed)
                .sum(),
            touched_paths: artifacts.code_changes.iter()
                .map(|c| c.file_path.clone())
                .collect(),
        };

        // Run validation
        let result = self.validator.validate(
            &super::super::orchestration::caws_runtime::WorkingSpec {
                risk_tier: working_spec.risk_tier as u8,
                scope_in: working_spec.scope.as_ref()
                    .and_then(|s| s.included.clone())
                    .unwrap_or_default(),
                change_budget_max_files: 50, // TODO: Make configurable
                change_budget_max_loc: 1000, // TODO: Make configurable
            },
            &task_desc,
            &diff_stats,
            &[], // no patches
            &[], // no language hints
            artifacts.test_results.total > 0, // tests added if we have results
            true, // assume deterministic for now
            vec![], // no waivers
        ).await?;

        Ok(result)
    }

    /// Convert working spec to task spec
    fn working_spec_to_task_spec(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<TaskSpec, AutonomousExecutionError> {
        Ok(TaskSpec {
            id: task_id,
            title: working_spec.title.clone(),
            description: format!(
                "Autonomous execution of working spec: {}\n\nDescription: {}\n\nAcceptance Criteria:\n{}",
                working_spec.id,
                working_spec.description,
                working_spec.acceptance_criteria.iter()
                    .enumerate()
                    .map(|(i, ac)| format!("{}. Given {}, When {}, Then {}",
                        i + 1, ac.given, ac.when, ac.then))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            risk_tier: match working_spec.risk_tier {
                1 => crate::models::RiskTier::Critical,
                2 => crate::models::RiskTier::High,
                3 => crate::models::RiskTier::Standard,
                _ => crate::models::RiskTier::Standard,
            },
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.included.clone())
                .unwrap_or_default(),
            acceptance_criteria: working_spec.acceptance_criteria.iter()
                .map(|ac| format!("Given {}, When {}, Then {}", ac.given, ac.when, ac.then))
                .collect(),
            constraints: working_spec.constraints.clone(),
            metadata: Some(serde_json::json!({
                "working_spec_id": working_spec.id,
                "generated_at": working_spec.generated_at,
                "context_hash": working_spec.context_hash,
            })),
        })
    }

    /// Send execution event
    async fn send_event(&self, event: ExecutionEvent) {
        if self.config.enable_event_streaming {
            let _ = self.event_sender.send(event);
        }
    }

    /// Create artifacts summary for events
    fn summarize_artifacts(&self, artifacts: &ExecutionArtifacts) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();
        summary.insert("code_files".to_string(), serde_json::json!(artifacts.code_changes.len()));
        summary.insert("test_passed".to_string(), serde_json::json!(artifacts.test_results.passed));
        summary.insert("test_failed".to_string(), serde_json::json!(artifacts.test_results.failed));
        summary.insert("coverage_percentage".to_string(), serde_json::json!(artifacts.coverage.coverage_percentage));
        summary.insert("mutation_score".to_string(), serde_json::json!(artifacts.mutation.mutation_score));
        summary.insert("lint_errors".to_string(), serde_json::json!(artifacts.lint.errors));
        summary
    }
}

/// Internal execution result
#[derive(Debug)]
struct ExecutionResultInternal {
    success: bool,
    artifacts: ExecutionArtifacts,
    error_message: Option<String>,
}

/// Public execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult {
    pub task_id: Uuid,
    pub working_spec_id: String,
    pub success: bool,
    pub artifacts: ExecutionArtifacts,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub completed_at: DateTime<Utc>,
}

pub type Result<T> = std::result::Result<T, AutonomousExecutionError>;

#[derive(Debug, thiserror::Error)]
pub enum AutonomousExecutionError {
    #[error("Worker execution failed: {0}")]
    WorkerError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Timeout exceeded")]
    TimeoutError,

    #[error("Artifact collection failed: {0}")]
    ArtifactError(String),

    #[error("Invalid working spec: {0}")]
    InvalidSpec(String),
}
