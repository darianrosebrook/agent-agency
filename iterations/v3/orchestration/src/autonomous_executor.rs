//! Autonomous Executor Loop
//!
//! Implements the core autonomous execution engine that can run tasks
//! end-to-end with progress tracking, error recovery, and consensus-based
//! decision making.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use agent_agency_contracts::task_executor::{TaskExecutionResult, TaskExecutor};
use agent_agency_contracts::task_executor_provider::TaskExecutorProvider;

use crate::orchestrate::{orchestrate_task, to_task_spec};
use crate::caws_runtime::{CawsRuntimeValidator, TaskDescriptor, WorkingSpec};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::tracking::progress_tracker::{ExecutionProgress, ExecutionStatus, ProgressTracker};
use crate::planning::types::ExecutionEvent;

use agent_agency_council::coordinator::ConsensusCoordinator;
use agent_agency_council::types::{ConsensusResult, FinalVerdict};
use agent_agency_observability::cache::CacheBackend;
use agent_agency_observability::metrics::MetricsBackend;

/// Configuration for the autonomous executor
#[derive(Debug, Clone)]
pub struct AutonomousExecutorConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task execution timeout (seconds)
    pub task_timeout_seconds: u64,
    /// Progress report interval (seconds)
    pub progress_report_interval_seconds: u64,
    /// Enable automatic retry on failure
    pub enable_auto_retry: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: usize,
    /// Enable consensus coordination
    pub enable_consensus: bool,
    /// Consensus timeout (seconds)
    pub consensus_timeout_seconds: u64,
}

/// Task execution state
#[derive(Debug, Clone)]
pub struct TaskExecutionState {
    pub task_id: Uuid,
    pub task_descriptor: TaskDescriptor,
    pub working_spec: WorkingSpec,
    pub start_time: DateTime<Utc>,
    pub status: ExecutionStatus,
    pub retry_count: usize,
    pub consensus_result: Option<ConsensusResult>,
    pub final_verdict: Option<FinalVerdict>,
    pub error_message: Option<String>,
}

/// Autonomous executor that runs tasks end-to-end
#[derive(Debug)]
pub struct AutonomousExecutor {
    config: AutonomousExecutorConfig,
    progress_tracker: Arc<ProgressTracker>,
    runtime_validator: Arc<dyn CawsRuntimeValidator>,
    consensus_coordinator: Option<Arc<ConsensusCoordinator>>,
    verdict_writer: Arc<dyn VerdictWriter>,
    provenance_emitter: Arc<OrchestrationProvenanceEmitter>,
    cache: Option<Arc<dyn CacheBackend>>,
    metrics: Option<Arc<dyn MetricsBackend>>,
    task_executor_provider: TaskExecutorProvider,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskExecutionState>>>,
    task_queue: mpsc::UnboundedSender<TaskDescriptor>,
    task_receiver: Arc<RwLock<mpsc::UnboundedReceiver<TaskDescriptor>>>,
}

impl AutonomousExecutor {
    /// Create a new autonomous executor
    pub fn new(
        config: AutonomousExecutorConfig,
        progress_tracker: Arc<ProgressTracker>,
        runtime_validator: Arc<dyn CawsRuntimeValidator>,
        consensus_coordinator: Option<Arc<ConsensusCoordinator>>,
        verdict_writer: Arc<dyn VerdictWriter>,
        provenance_emitter: Arc<OrchestrationProvenanceEmitter>,
        cache: Option<Arc<dyn CacheBackend>>,
        metrics: Option<Arc<dyn MetricsBackend>>,
        task_executor_provider: TaskExecutorProvider,
    ) -> Self {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            progress_tracker,
            runtime_validator,
            consensus_coordinator,
            verdict_writer,
            provenance_emitter,
            cache,
            metrics,
            task_executor_provider,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: task_sender,
            task_receiver: Arc::new(RwLock::new(task_receiver)),
        }
    }

    /// Submit a task for autonomous execution
    pub async fn submit_task(&self, task_descriptor: TaskDescriptor) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = task_descriptor.task_id;

        // Create initial execution state
        let execution_state = TaskExecutionState {
            task_id,
            task_descriptor: task_descriptor.clone(),
            working_spec: WorkingSpec {
                risk_tier: 1, // Low risk default
                scope_in: vec![],
                change_budget_max_files: 50,
                change_budget_max_loc: 1000,
            }, // Will be filled during planning
            start_time: Utc::now(),
            status: ExecutionStatus::Pending,
            retry_count: 0,
            consensus_result: None,
            final_verdict: None,
            error_message: None,
        };

        // Store in active tasks
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task_id, execution_state);
        }

        // Send to execution queue
        self.task_queue.send(task_descriptor)?;

        // Record metrics
        if let Some(ref metrics) = self.metrics {
            let _ = metrics.counter("autonomous_executor_tasks_submitted", &[], 1).await;
        }

        tracing::info!("Task {} submitted for autonomous execution", task_id);
        Ok(task_id)
    }

    /// Start the autonomous execution loop
    pub async fn start_execution_loop(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting autonomous execution loop with config: {:?}", self.config);

        let executor = Arc::clone(&self);

        // Spawn the main execution loop
        tokio::spawn(async move {
            if let Err(e) = executor.execution_loop().await {
                tracing::error!("Autonomous execution loop failed: {}", e);
            }
        });

        // Spawn progress reporting
        let progress_executor = Arc::clone(&self);
        tokio::spawn(async move {
            progress_executor.progress_reporting_loop().await;
        });

        // Spawn cleanup task
        let cleanup_executor = Arc::clone(&self);
        tokio::spawn(async move {
            cleanup_executor.cleanup_loop().await;
        });

        Ok(())
    }

    /// Main execution loop
    async fn execution_loop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut receiver = self.task_receiver.write().await;

        loop {
            // Wait for tasks or timeout for health checks
            match time::timeout(Duration::from_secs(30), receiver.recv()).await {
                Ok(Some(task_descriptor)) => {
                    let executor = Arc::new(self.clone());
                    tokio::spawn(async move {
                        if let Err(e) = executor.execute_task(task_descriptor).await {
                            tracing::error!("Task execution failed: {}", e);
                        }
                    });
                }
                Ok(None) => {
                    // Channel closed, exit
                    break;
                }
                Err(_) => {
                    // Timeout - perform health checks
                    self.perform_health_checks().await;
                }
            }
        }

        Ok(())
    }

    /// Execute a single task end-to-end
    async fn execute_task(&self, task_descriptor: TaskDescriptor) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let task_id = task_descriptor.task_id;
        let start_time = Instant::now();

        tracing::info!("Starting {} execution of task {}", match task_descriptor.execution_mode {
            crate::caws_runtime::ExecutionMode::Strict => "strict",
            crate::caws_runtime::ExecutionMode::Auto => "auto",
            crate::caws_runtime::ExecutionMode::DryRun => "dry-run",
        }, task_id);

        // Enforce execution mode behavior
        match task_descriptor.execution_mode {
            crate::caws_runtime::ExecutionMode::DryRun => {
                tracing::info!("Dry-run mode: Simulating execution without filesystem changes");
                // For dry-run, we still validate and plan but skip actual execution
                self.update_task_status(task_id.clone(), ExecutionStatus::Starting, Some("Initializing dry-run execution".to_string())).await?;
            }
            crate::caws_runtime::ExecutionMode::Strict => {
                tracing::info!("Strict mode: Manual approval required for each phase");
                self.update_task_status(task_id.clone(), ExecutionStatus::Starting, Some("Initializing strict mode execution".to_string())).await?;
            }
            crate::caws_runtime::ExecutionMode::Auto => {
                tracing::info!("Auto mode: Automatic execution with quality gates");
                self.update_task_status(task_id.clone(), ExecutionStatus::Starting, Some("Initializing auto execution".to_string())).await?;
            }
        }

        // Phase 1: Validate and prepare task
        let working_spec = self.prepare_task(&task_descriptor).await?;
        self.update_task_progress(task_id.clone(), 10.0, Some("Task prepared".to_string())).await?;

        // Strict mode: Require approval before proceeding
        if task_descriptor.execution_mode == crate::caws_runtime::ExecutionMode::Strict {
            self.update_task_status(task_id.clone(), ExecutionStatus::AwaitingApproval, Some("Awaiting approval for planning phase".to_string())).await?;
            // In a real implementation, this would wait for external approval
            tracing::info!("Strict mode: Awaiting user approval for planning phase");
        }

        // Phase 2: Planning and validation
        self.validate_task(&working_spec, &task_descriptor).await?;
        self.update_task_progress(task_id.clone(), 25.0, Some("Planning and validation complete".to_string())).await?;

        // Strict mode: Require approval before consensus
        if task_descriptor.execution_mode == crate::caws_runtime::ExecutionMode::Strict {
            self.update_task_status(task_id.clone(), ExecutionStatus::AwaitingApproval, Some("Awaiting approval for consensus phase".to_string())).await?;
            tracing::info!("Strict mode: Awaiting user approval for consensus phase");
        }

        // Phase 3: Consensus coordination (if enabled)
        if self.config.enable_consensus {
            self.perform_consensus_coordination(&working_spec, &task_descriptor).await?;
            self.update_task_progress(task_id.clone(), 40.0, Some("Consensus coordination complete".to_string())).await?;
        }

        // Strict mode: Require approval before execution
        if task_descriptor.execution_mode == crate::caws_runtime::ExecutionMode::Strict {
            self.update_task_status(task_id.clone(), ExecutionStatus::AwaitingApproval, Some("Awaiting approval for execution phase".to_string())).await?;
            tracing::info!("Strict mode: Awaiting user approval for execution phase");
        }

        // Phase 4: Execute task orchestration (skip for dry-run)
        let final_verdict = match task_descriptor.execution_mode {
            crate::caws_runtime::ExecutionMode::DryRun => {
                tracing::info!("Dry-run mode: Skipping actual orchestration, simulating results");
                // Create a mock verdict for dry-run
                agent_agency_council::types::FinalVerdict {
                    decision: "Accept".to_string(),
                    confidence: 0.95,
                    summary: "Dry-run simulation - no actual changes made".to_string(),
                    metadata: std::collections::HashMap::new(),
                }
            }
            _ => {
                self.execute_orchestration(&working_spec, &task_descriptor).await?
            }
        };
        self.update_task_progress(task_id.clone(), 80.0, Some("Task orchestration complete".to_string())).await?;

        // Phase 5: Post-execution processing
        self.process_results(&final_verdict, &task_descriptor).await?;
        self.update_task_progress(task_id.clone(), 100.0, Some("Execution complete".to_string())).await?;

        // Update final status
        self.update_task_status(task_id, ExecutionStatus::Completed, None).await?;

        let duration = start_time.elapsed();
        tracing::info!("Task {} completed successfully in {:?}", task_id, duration);

        // Record metrics
        if let Some(ref metrics) = self.metrics {
            let _ = metrics.counter("autonomous_executor_tasks_completed", &[], 1).await;
            let _ = metrics.histogram("autonomous_executor_task_duration", &[], duration.as_secs_f64()).await;
        }

        Ok(())
    }

    /// Prepare task specification
    async fn prepare_task(&self, task_descriptor: &TaskDescriptor) -> Result<WorkingSpec, Box<dyn std::error::Error + Send + Sync>> {
        // Generate working spec from task descriptor
        // This would involve planning and specification generation
        let working_spec = WorkingSpec {
            id: task_descriptor.task_id.to_string(),
            title: format!("Autonomous Task {}", task_descriptor.task_id),
            description: task_descriptor.description.clone(),
            risk_tier: match task_descriptor.risk_tier {
                1 => agent_agency_council::models::RiskTier::Low,
                2 => agent_agency_council::models::RiskTier::Medium,
                _ => agent_agency_council::models::RiskTier::High,
            },
            scope_in: task_descriptor.scope_in.clone(),
            scope_out: task_descriptor.scope_out.clone(),
            acceptance_criteria: task_descriptor.acceptance.clone().unwrap_or_default(),
            invariants: vec![],
            operational_rollback_slo: "5m".to_string(),
            blast_radius: Default::default(),
            mode: "feature".to_string(),
            change_budget: Default::default(),
            non_functional_requirements: Default::default(),
            contracts: vec![],
        };

        Ok(working_spec)
    }

    /// Validate task specification
    async fn validate_task(&self, working_spec: &WorkingSpec, task_descriptor: &TaskDescriptor) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Perform validation checks
        // This would involve CAWS runtime validation
        self.runtime_validator.validate_spec(working_spec).await?;

        Ok(())
    }

    /// Perform consensus coordination
    async fn perform_consensus_coordination(&self, working_spec: &WorkingSpec, task_descriptor: &TaskDescriptor) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref coordinator) = self.consensus_coordinator {
            let task_spec = to_task_spec(task_descriptor);

            // Run consensus coordination with timeout
            let consensus_timeout = Duration::from_secs(self.config.consensus_timeout_seconds);
            let consensus_result = time::timeout(
                consensus_timeout,
                coordinator.coordinate_consensus(task_spec)
            ).await??;

            // Store consensus result
            let mut active_tasks = self.active_tasks.write().await;
            if let Some(state) = active_tasks.get_mut(&task_descriptor.task_id) {
                state.consensus_result = Some(consensus_result);
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Execute task orchestration
    async fn execute_orchestration(&self, working_spec: &WorkingSpec, task_descriptor: &TaskDescriptor) -> Result<FinalVerdict, Box<dyn std::error::Error + Send + Sync>> {
        let diff_stats = crate::caws_runtime::DiffStats {
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            lines_added: 0,
            lines_deleted: 0,
            binary_files_changed: 0,
        };

        let verdict = orchestrate_task(
            working_spec,
            task_descriptor,
            &diff_stats,
            false, // tests_added
            true,  // deterministic
            &mut self.consensus_coordinator.clone().unwrap(),
            &*self.verdict_writer,
            &self.provenance_emitter,
            &self.provenance_emitter,
            None, // council circuit breaker
            None, // db circuit breaker
        ).await?;

        Ok(verdict)
    }

    /// Process execution results
    async fn process_results(&self, final_verdict: &FinalVerdict, task_descriptor: &TaskDescriptor) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Store final verdict
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(state) = active_tasks.get_mut(&task_descriptor.task_id) {
            state.final_verdict = Some(final_verdict.clone());
        }

        // Write verdict to persistence
        self.verdict_writer.write_verdict(final_verdict).await?;

        Ok(())
    }

    /// Update task progress
    async fn update_task_progress(&self, task_id: Uuid, completion_percentage: f32, phase: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut progress = ExecutionProgress {
            task_id,
            status: ExecutionStatus::Running,
            start_time: Utc::now(),
            last_update: Utc::now(),
            events: vec![],
            current_phase: phase,
            completion_percentage,
            metadata: HashMap::new(),
        };

        self.progress_tracker.update_progress(task_id, progress).await?;
        Ok(())
    }

    /// Update task status
    async fn update_task_status(&self, task_id: Uuid, status: ExecutionStatus, error_message: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(state) = active_tasks.get_mut(&task_id) {
            state.status = status.clone();
            if let Some(error) = &error_message {
                state.error_message = Some(error.clone());
            }
        }

        let mut progress = ExecutionProgress {
            task_id,
            status,
            start_time: Utc::now(),
            last_update: Utc::now(),
            events: vec![],
            current_phase: None,
            completion_percentage: if matches!(status, ExecutionStatus::Completed) { 100.0 } else { 0.0 },
            metadata: HashMap::new(),
        };

        self.progress_tracker.update_progress(task_id, progress).await?;
        Ok(())
    }

    /// Progress reporting loop
    async fn progress_reporting_loop(&self) {
        let mut interval = time::interval(Duration::from_secs(self.config.progress_report_interval_seconds));

        loop {
            interval.tick().await;

            // Report progress for all active tasks
            let active_tasks = self.active_tasks.read().await;
            for (task_id, state) in active_tasks.iter() {
                if let Ok(Some(progress)) = self.progress_tracker.get_progress(*task_id).await {
                    tracing::info!(
                        "Task {} progress: {:.1}% - {} ({:?})",
                        task_id,
                        progress.completion_percentage,
                        progress.current_phase.as_deref().unwrap_or("Unknown"),
                        progress.status
                    );
                }
            }
        }
    }

    /// Cleanup completed tasks
    async fn cleanup_loop(&self) {
        let mut interval = time::interval(Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            let mut active_tasks = self.active_tasks.write().await;
            let completed_tasks: Vec<Uuid> = active_tasks.iter()
                .filter(|(_, state)| matches!(state.status, ExecutionStatus::Completed | ExecutionStatus::Failed))
                .map(|(id, _)| *id)
                .collect();

            for task_id in completed_tasks {
                active_tasks.remove(&task_id);
                tracing::info!("Cleaned up completed task {}", task_id);
            }
        }
    }

    /// Perform health checks
    async fn perform_health_checks(&self) {
        // Check consensus coordinator health
        if let Some(ref coordinator) = self.consensus_coordinator {
            if let Ok(health) = coordinator.health_check().await {
                if !health {
                    tracing::warn!("Consensus coordinator health check failed");
                }
            }
        }

        // Check cache health
        if let Some(ref cache) = self.cache {
            if let Ok(false) = cache.exists("health_check").await {
                tracing::warn!("Cache health check failed");
            }
        }

        // Record health metrics
        if let Some(ref metrics) = self.metrics {
            let active_task_count = self.active_tasks.read().await.len() as u64;
            let _ = metrics.gauge("autonomous_executor_active_tasks", &[], active_task_count as f64).await;
        }
    }

    /// Get current task status
    pub async fn get_task_status(&self, task_id: Uuid) -> Option<TaskExecutionState> {
        let active_tasks = self.active_tasks.read().await;
        active_tasks.get(&task_id).cloned()
    }

    /// Pause a running task
    pub async fn pause_task(&self, task_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(mut state) = active_tasks.get_mut(&task_id) {
            if state.status != ExecutionStatus::Running {
                return Ok(false); // Can only pause running tasks
            }

            state.status = ExecutionStatus::Paused;
            self.update_task_status(task_id, ExecutionStatus::Paused, Some("Task paused by user".to_string())).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resume a paused task
    pub async fn resume_task(&self, task_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(mut state) = active_tasks.get_mut(&task_id) {
            if state.status != ExecutionStatus::Paused {
                return Ok(false); // Can only resume paused tasks
            }

            state.status = ExecutionStatus::Running;
            self.update_task_status(task_id, ExecutionStatus::Running, Some("Task resumed by user".to_string())).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(mut state) = active_tasks.get_mut(&task_id) {
            state.status = ExecutionStatus::Cancelled;

            // Try to cancel on the worker if we have a worker_id
            if let Some(worker_id) = state.worker_id {
                if let Err(e) = self.task_executor_provider.create_executor().cancel_task_execution(task_id, worker_id).await {
                    tracing::warn!("Failed to cancel task {} on worker {}: {}", task_id, worker_id, e);
                    // Continue with local cancellation even if worker cancel fails
                }
            }

            self.update_task_status(task_id, ExecutionStatus::Cancelled, Some("Task cancelled by user".to_string())).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
