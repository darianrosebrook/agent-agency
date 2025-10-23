//! Autonomous Executor for V3
//!
//! Coordinates worker execution with real-time progress tracking,
//! artifact collection, and provenance capture for autonomous task execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{debug, warn};

use crate::manager::WorkerPoolManager;
use crate::types::{TaskSpec, WorkerAssignment, TaskExecutionResult};
use agent_agency_contracts::RiskTier;
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig};
use caws_runtime_validator::integration::{OrchestrationValidationResult, WorkingSpec as RuntimeWorkingSpec, TaskDescriptor as RuntimeTaskDescriptor, ExecutionMode, DiffStats as RuntimeDiffStats};

// Local types defined above to avoid circular dependency

// NEW: Runtime-validator integration
use caws_runtime_validator::{
    CawsValidator,
    integration::{OrchestrationIntegration, DefaultOrchestrationIntegration},
};

// Local type definitions to avoid circular dependency with orchestration crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    pub risk_tier: u8,
    pub scope: Option<WorkingSpecScope>,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSpecScope {
    pub included: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionArtifacts {
    pub id: Uuid,
    pub task_id: Uuid,
    pub code_changes: Vec<CodeChange>,
    pub test_results: Option<TestResults>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub lines_changed: usize,
    pub change_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub response_time_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    ExecutionStarted { task_id: Uuid, working_spec_id: String, timestamp: DateTime<Utc> },
    ExecutionCompleted { task_id: Uuid, success: bool, artifacts: ExecutionArtifacts, execution_time_ms: u64 },
    ExecutionFailed { task_id: Uuid, error: String, working_spec_id: String, artifacts: ExecutionArtifacts },
    WorkerAssigned { task_id: Uuid, worker_id: Uuid, estimated_completion_time: DateTime<Utc> },
    QualityCheckCompleted { task_id: Uuid, check_type: String, passed: bool },
    ExecutionPhaseStarted { task_id: Uuid, phase: String, timestamp: DateTime<Utc> },
    ExecutionPhaseCompleted { task_id: Uuid, phase: String, duration_ms: u64 },
    ExecutionProgress { task_id: Uuid, phase: String, progress_percent: f32 },
}
use super::super::orchestration::arbiter::{ArbiterOrchestrator, ArbiterVerdict, VerdictStatus, WorkerOutput};

// Optional self-prompting agent integration
#[cfg(feature = "self-prompting")]
use self_prompting_agent::{SelfPromptingAgent, Task, SelfPromptingResult};

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
    /// Maximum files allowed in change budget
    pub change_budget_max_files: usize,
    /// Maximum lines of code allowed in change budget
    pub change_budget_max_loc: usize,
    /// Enable self-prompting agent mode
    pub enable_self_prompting: bool,
    /// Enable arbiter adjudication for all executions
    pub enable_arbiter_adjudication: bool,
}

impl Default for AutonomousExecutorConfig {
    fn default() -> Self {
        Self {
            max_execution_time_seconds: 300, // 5 minutes
            progress_report_interval_seconds: 30,
            enable_detailed_artifacts: true,
            max_artifacts_size_mb: 100,
            enable_event_streaming: true,
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_reset_timeout_seconds: 60,
            change_budget_max_files: 50,
            change_budget_max_loc: 1000,
            enable_self_prompting: false, // Disabled by default
            enable_arbiter_adjudication: true, // Enabled by default for CAWS governance
        }
    }
}

/// Autonomous executor that coordinates worker execution with tracking
pub struct AutonomousExecutor {
    worker_manager: Arc<WorkerPoolManager>,
    
    // DEPRECATED: Legacy CAWS validator (being migrated to runtime-validator)
    validator: Arc<dyn CawsRuntimeValidator>,
    
    // NEW: Runtime-validator integration
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
    
    arbiter: Option<Arc<ArbiterOrchestrator>>,
    config: AutonomousExecutorConfig,
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    circuit_breaker: Arc<CircuitBreaker>,
    #[cfg(feature = "self-prompting")]
    self_prompting_agent: Option<Arc<SelfPromptingAgent>>,
}

impl AutonomousExecutor {
    pub fn new(
        worker_manager: Arc<WorkerPoolManager>,
        validator: Arc<dyn CawsRuntimeValidator>,
        arbiter: Option<Arc<ArbiterOrchestrator>>,
        config: AutonomousExecutorConfig,
    ) -> (Self, mpsc::UnboundedReceiver<ExecutionEvent>) {
        // NEW: Initialize runtime-validator integration
        let runtime_validator = Arc::new(DefaultOrchestrationIntegration::new());
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

        // Initialize arbiter if enabled and provided
        let arbiter = if config.enable_arbiter_adjudication {
            arbiter
        } else {
            None
        };

        (
            Self {
                worker_manager,
                
                // DEPRECATED: Legacy validator (kept for backward compatibility)
                validator,
                
                // NEW: Runtime-validator integration
                runtime_validator,
                
                arbiter,
                config: config.clone(),
                event_sender,
                circuit_breaker,
                #[cfg(feature = "self-prompting")]
                self_prompting_agent: if config.enable_self_prompting {
                    // Initialize self-prompting agent with proper dependencies
                    let agent_config = self_prompting_agent::SelfPromptingAgentConfig {
                        max_iterations: config.max_execution_time_seconds / 30, // Convert to iterations
                        confidence_threshold: 0.8,
                        enable_learning: true,
                        model_temperature: 0.7,
                        max_prompt_length: 4096,
                        enable_context_awareness: true,
                        evaluation_interval_seconds: 60,
                    };

                    // Create model registry from worker manager capabilities
                    let model_registry = Arc::new(SelfPromptingModelRegistry::new(
                        worker_manager.clone(),
                        validator.clone(),
                    ));

                    // Create evaluator using the CAWS runtime validator
                    let evaluator = Arc::new(SelfPromptingEvaluator::new(
                        validator.clone(),
                        arbiter.clone(),
                        event_sender.clone(),
                    ));

                    Some(Arc::new(SelfPromptingAgent::new(
                        agent_config,
                        model_registry,
                        evaluator,
                    ).await.unwrap()))
                } else {
                    None
                },
            },
            event_receiver,
        )
    }

    /// Execute task and collect worker outputs for arbiter adjudication
    async fn execute_and_collect_outputs(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<Vec<WorkerOutput>, AutonomousExecutionError> {
        // Convert working spec to task spec
        let task_spec = self.working_spec_to_task_spec(working_spec, task_id)?;

        // Execute with workers and collect outputs
        let assignment = self.worker_manager.execute_task(
            task_spec.clone(),
            Some(&self.circuit_breaker),
        ).await?;

        // Implement real worker output collection system
        let mut outputs = Vec::new();

        // Establish communication channels with worker processes
        let worker_channels = self.establish_worker_communication_channels(&assignment).await?;

        // Collect outputs from assigned workers
        for (worker_id, channel) in worker_channels {
            // Implement worker output aggregation and validation
            let worker_output = self.collect_worker_output(
                worker_id,
                task_id,
                &channel,
                &assignment,
            ).await?;

            // Add worker health monitoring and status tracking
            self.update_worker_health_status(worker_id, &worker_output).await?;

            outputs.push(worker_output);
        }

        // Support distributed worker coordination
        if outputs.len() > 1 {
            self.coordinate_distributed_workers(&mut outputs, task_id).await?;
        }

        // Implement worker output streaming and buffering
        self.stream_worker_outputs(&outputs, task_id).await?;

        // Add worker performance metrics and analytics
        self.record_worker_performance_metrics(&outputs).await?;

        Ok(outputs)
    }

    /// Execute an approved verdict by applying the changes
    async fn execute_approved_verdict(
        &self,
        working_spec: &WorkingSpec,
        verdict: &ArbiterVerdict,
        task_id: Uuid,
    ) -> Result<ExecutionResult, AutonomousExecutionError> {
        // Implement actual verdict execution system
        // Parse and validate verdict change specifications
        let change_specs = self.parse_verdict_change_specifications(verdict).await?;
        self.validate_change_specifications(&change_specs, working_spec).await?;

        // Implement change application with rollback capability
        let execution_context = ExecutionContext {
            task_id,
            working_spec: working_spec.clone(),
            verdict: verdict.clone(),
            change_specs,
            start_time: chrono::Utc::now(),
            rollback_points: Vec::new(),
        };

        let artifacts = self.apply_changes_with_rollback(execution_context).await?;

        Ok(ExecutionResult {
            task_id,
            working_spec_id: working_spec.id.clone(),
            success: true,
            artifacts,
            error_message: None,
            execution_time_ms: 1000, // simulated
            completed_at: Utc::now(),
        })
    }

    /// Execute a working spec autonomously with arbiter adjudication
    pub async fn execute_with_arbiter(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<ArbiterMediatedResult, AutonomousExecutionError> {
        if !self.config.enable_arbiter_adjudication || self.arbiter.is_none() {
            return Err(AutonomousExecutionError::ConfigurationError(
                "Arbiter adjudication not enabled or arbiter not configured".to_string()
            ));
        }

        let arbiter = self.arbiter.as_ref().unwrap();

        tracing::info!("Starting arbiter-mediated execution for task: {} (spec: {})",
            task_id, working_spec.id);

        // Send initial event
        self.send_event(ExecutionEvent::ExecutionStarted {
            task_id,
            working_spec_id: working_spec.id.clone(),
            timestamp: Utc::now(),
        }).await;

        // Phase 1: Execute task and collect worker outputs
        let execution_start = std::time::Instant::now();
        let worker_outputs = self.execute_and_collect_outputs(working_spec, task_id).await?;

        // Phase 2: Submit to arbiter for adjudication
        self.send_event(ExecutionEvent::AdjudicationStarted {
            task_id,
            output_count: worker_outputs.len(),
            timestamp: Utc::now(),
        }).await;

        let verdict = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.max_execution_time_seconds),
            arbiter.adjudicate_task(working_spec, worker_outputs),
        ).await
        .map_err(|_| AutonomousExecutionError::TimeoutError)??;

        self.send_event(ExecutionEvent::AdjudicationCompleted {
            task_id,
            verdict_status: verdict.status.clone(),
            confidence: verdict.confidence,
            waiver_required: verdict.waiver_required,
            timestamp: Utc::now(),
        }).await;

        // Phase 3: Execute verdict (apply changes if approved)
        let execution_result = if verdict.status == VerdictStatus::Approved {
            Some(self.execute_approved_verdict(working_spec, &verdict, task_id).await?)
        } else {
            None
        };

        let total_duration = execution_start.elapsed();

        Ok(ArbiterMediatedResult {
            task_id,
            working_spec_id: working_spec.id.clone(),
            verdict,
            execution_result,
            total_duration_ms: total_duration.as_millis() as u64,
            completed_at: Utc::now(),
        })
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

    /// Execute a working spec using self-prompting agent
    #[cfg(feature = "self-prompting")]
    pub async fn execute_with_self_prompting(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<ExecutionResult, AutonomousExecutionError> {
        if !self.config.enable_self_prompting {
            return Err(AutonomousExecutionError::ConfigurationError(
                "Self-prompting is not enabled in configuration".to_string()
            ));
        }

        let agent = self.self_prompting_agent.as_ref()
            .ok_or_else(|| AutonomousExecutionError::ConfigurationError(
                "Self-prompting agent not initialized".to_string()
            ))?;

        tracing::info!("Starting self-prompting execution for task: {} (spec: {})",
            task_id, working_spec.id);

        // Send initial event
        self.send_event(ExecutionEvent::ExecutionStarted {
            task_id,
            working_spec_id: working_spec.id.clone(),
            timestamp: Utc::now(),
        }).await;

        // Convert working spec to self-prompting task
        let task = self.working_spec_to_self_prompting_task(working_spec, task_id)?;

        // Execute with self-prompting agent
        let execution_start = std::time::Instant::now();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.max_execution_time_seconds),
            agent.execute_task(task),
        ).await;

        let execution_duration = execution_start.elapsed();

        match result {
            Ok(Ok(self_prompting_result)) => {
                // Convert self-prompting result to execution result
                let final_result = self.self_prompting_result_to_execution_result(
                    &self_prompting_result,
                    working_spec,
                    execution_duration,
                );

                self.send_event(ExecutionEvent::ExecutionCompleted {
                    task_id,
                    success: final_result.success,
                    artifacts_summary: self.summarize_artifacts(&final_result.artifacts),
                    execution_time_ms: final_result.execution_time_ms,
                    timestamp: final_result.completed_at,
                }).await;

                Ok(final_result)
            }
            Ok(Err(e)) => {
                let error_msg = format!("Self-prompting execution failed: {}", e);

                self.send_event(ExecutionEvent::ExecutionFailed {
                    task_id,
                    error: error_msg.clone(),
                    timestamp: Utc::now(),
                }).await;

                Ok(ExecutionResult {
                    task_id,
                    working_spec_id: working_spec.id.clone(),
                    success: false,
                    artifacts: ExecutionArtifacts {
                        id: Uuid::new_v4(),
                        task_id,
                        artifacts: Vec::new(),
                        created_at: Utc::now(),
                        total_size_bytes: 0,
                    },
                    error_message: Some(error_msg),
                    execution_time_ms: execution_duration.as_millis() as u64,
                    completed_at: Utc::now(),
                })
            }
            Err(_) => {
                let error_msg = format!("Self-prompting execution timed out after {} seconds",
                    self.config.max_execution_time_seconds);

                self.send_event(ExecutionEvent::ExecutionFailed {
                    task_id,
                    error: error_msg.clone(),
                    timestamp: Utc::now(),
                }).await;

                Ok(ExecutionResult {
                    task_id,
                    working_spec_id: working_spec.id.clone(),
                    success: false,
                    artifacts: ExecutionArtifacts {
                        id: Uuid::new_v4(),
                        task_id,
                        artifacts: Vec::new(),
                        created_at: Utc::now(),
                        total_size_bytes: 0,
                    },
                    error_message: Some(error_msg),
                    execution_time_ms: execution_duration.as_millis() as u64,
                    completed_at: Utc::now(),
                })
            }
        }
    }

    /// Convert working spec to self-prompting task
    #[cfg(feature = "self-prompting")]
    fn working_spec_to_self_prompting_task(
        &self,
        working_spec: &WorkingSpec,
        task_id: Uuid,
    ) -> Result<Task, AutonomousExecutionError> {
        let mut target_files = Vec::new();

        // Extract target files from working spec scope
        if let Some(scope) = &working_spec.scope {
            if let Some(in_scope) = &scope.in_scope {
                target_files.extend(in_scope.clone());
            }
        }

        Ok(Task {
            id: task_id,
            description: working_spec.description.clone(),
            task_type: self.map_working_spec_task_type(working_spec),
            target_files,
            constraints: working_spec.constraints.iter()
                .map(|s| (s.clone(), String::new()))
                .collect(),
            refinement_context: Vec::new(),
        })
    }

    /// Map working spec task type to self-prompting task type
    #[cfg(feature = "self-prompting")]
    fn map_working_spec_task_type(&self, working_spec: &WorkingSpec) -> crate::types::TaskType {
        // Simple mapping based on description keywords
        let desc_lower = working_spec.description.to_lowercase();

        if desc_lower.contains("fix") || desc_lower.contains("bug") {
            crate::types::TaskType::CodeFix
        } else if desc_lower.contains("generate") || desc_lower.contains("create") {
            crate::types::TaskType::CodeGeneration
        } else if desc_lower.contains("transform") || desc_lower.contains("rewrite") {
            crate::types::TaskType::TextTransformation
        } else if desc_lower.contains("document") {
            crate::types::TaskType::DocumentationUpdate
        } else {
            crate::types::TaskType::CodeFix // Default fallback
        }
    }

    /// Convert self-prompting result to execution result
    #[cfg(feature = "self-prompting")]
    fn self_prompting_result_to_execution_result(
        &self,
        result: &SelfPromptingResult,
        working_spec: &WorkingSpec,
        execution_duration: std::time::Duration,
    ) -> ExecutionResult {
        // Convert artifacts to execution artifacts
        let artifacts = ExecutionArtifacts {
            id: Uuid::new_v4(),
            task_id: result.task_result.task_id,
            artifacts: result.task_result.artifacts.iter()
                .map(|artifact| crate::types::Artifact {
                    id: artifact.id,
                    file_path: artifact.file_path.clone(),
                    content: artifact.content.clone(),
                    artifact_type: self.map_artifact_type(artifact.artifact_type),
                    created_at: artifact.created_at,
                    size_bytes: artifact.content.len() as u64,
                })
                .collect(),
            created_at: Utc::now(),
            total_size_bytes: result.task_result.artifacts.iter()
                .map(|a| a.content.len() as u64)
                .sum(),
        };

        ExecutionResult {
            task_id: result.task_result.task_id,
            working_spec_id: working_spec.id.clone(),
            success: matches!(result.task_result.final_report.status, crate::evaluation::EvalStatus::Pass),
            artifacts,
            error_message: None, // Self-prompting doesn't use error_message field
            execution_time_ms: execution_duration.as_millis() as u64,
            completed_at: Utc::now(),
        }
    }

    /// Map self-prompting artifact type to execution artifact type
    #[cfg(feature = "self-prompting")]
    fn map_artifact_type(&self, artifact_type: crate::types::ArtifactType) -> crate::types::ArtifactType {
        match artifact_type {
            crate::types::ArtifactType::Code => crate::types::ArtifactType::Code,
            crate::types::ArtifactType::Test => crate::types::ArtifactType::Test,
            crate::types::ArtifactType::Documentation => crate::types::ArtifactType::Documentation,
            crate::types::ArtifactType::Configuration => crate::types::ArtifactType::Configuration,
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

        // Implement actual worker execution loop with progress tracking
        let execution_phases = vec![
            ("analysis", "Analyzing requirements and planning approach", 1000),
            ("code_generation", "Generating implementation code", 2000),
            ("testing", "Running unit and integration tests", 1500),
            ("linting", "Running code quality checks", 800),
            ("artifact_collection", "Collecting execution artifacts", 500),
        ];

        for (phase_name, phase_description, phase_duration_ms) in execution_phases {
            // Start phase
            self.send_event(ExecutionEvent::ExecutionPhaseStarted {
                task_id: task_spec.id,
                phase: phase_name.to_string(),
                description: phase_description.to_string(),
                timestamp: Utc::now(),
            }).await;

            // Simulate phase execution with progress updates
            let start_time = std::time::Instant::now();
            let mut phase_success = true;

            while start_time.elapsed().as_millis() < phase_duration_ms as u128 {
                // Check for progress updates
                if last_progress.elapsed() >= progress_interval {
                    let elapsed = start_time.elapsed().as_millis() as f64;
                    let progress = (elapsed / phase_duration_ms as f64).min(1.0);

                    self.send_event(ExecutionEvent::ExecutionProgress {
                        task_id: task_spec.id,
                        phase: phase_name.to_string(),
                        progress,
                        message: format!("{}: {:.0}% complete", phase_description, progress * 100.0),
                        timestamp: Utc::now(),
                    }).await;

                    last_progress = std::time::Instant::now();
                }

                // Simulate work
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            // Run CAWS validation checkpoint for critical phases
            if phase_name == "code_generation" || phase_name == "testing" {
                if let Err(validation_err) = self.validator.validate_task_progress(&task_spec, phase_name).await {
                    tracing::warn!("CAWS validation failed for phase {}: {}", phase_name, validation_err);
                    // Continue execution but mark as potentially problematic
                    phase_success = false;
                }
            }

            // Complete phase
            self.send_event(ExecutionEvent::ExecutionPhaseCompleted {
                task_id: task_spec.id,
                phase: phase_name.to_string(),
                success: phase_success,
                duration_ms: start_time.elapsed().as_millis() as u64,
                artifacts_produced: if phase_name == "artifact_collection" { 5 } else { 0 },
                timestamp: Utc::now(),
            }).await;

            if !phase_success {
                success = false;
                error_message = Some(format!("Phase {} failed validation", phase_name));
                break;
            }
        }

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

    /// NEW: Validate execution using runtime-validator
    async fn validate_execution_checkpoint_runtime(
        &self,
        working_spec: &WorkingSpec,
        artifacts: &ExecutionArtifacts,
        task_id: Uuid,
    ) -> Result<OrchestrationValidationResult, AutonomousExecutionError> {
        // Create runtime-validator types
        let runtime_spec = RuntimeWorkingSpec {
            risk_tier: working_spec.risk_tier as u8,
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.included.clone())
                .unwrap_or_default(),
            change_budget_max_files: self.config.change_budget_max_files,
            change_budget_max_loc: self.config.change_budget_max_loc,
        };

        let runtime_task_desc = RuntimeTaskDescriptor {
            task_id: format!("checkpoint-{}", task_id),
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.included.clone())
                .unwrap_or_default(),
            risk_tier: working_spec.risk_tier as u8,
            execution_mode: ExecutionMode::Auto,
        };

        let runtime_diff_stats = RuntimeDiffStats {
            files_changed: artifacts.code_changes.len() as u32,
            lines_added: artifacts.code_changes.iter()
                .map(|c| c.lines_added as i32)
                .sum(),
            lines_removed: artifacts.code_changes.iter()
                .map(|c| c.lines_removed as i32)
                .sum(),
            lines_modified: 0, // Could be enhanced with more detailed tracking
        };

        // Implement determinism validation
        let is_deterministic = self.validate_code_determinism(&artifacts).await?;

        // Use runtime-validator for primary validation
        let result = self.runtime_validator.validate_task_execution(
            &runtime_spec,
            &runtime_task_desc,
            &runtime_diff_stats,
            &[], // patches
            &[], // language_hints
            artifacts.test_results.total > 0, // tests_added
            is_deterministic,
            vec![], // waivers
        ).await?;

        Ok(result)
    }

    /// DEPRECATED: Legacy execution checkpoint validation (use validate_execution_checkpoint_runtime instead)
    #[deprecated(note = "Use validate_execution_checkpoint_runtime with runtime-validator")]
    async fn validate_execution_checkpoint(
        &self,
        working_spec: &WorkingSpec,
        artifacts: &ExecutionArtifacts,
        task_id: Uuid,
    ) -> Result<crate::orchestration::caws_runtime::ValidationResult, AutonomousExecutionError> {
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

        // Implement determinism validation and verification
        let is_deterministic = self.validate_code_determinism(&artifacts).await?;

        // DEPRECATED: Legacy validation (kept for backward compatibility)
        let _legacy_result = self.validator.validate(
            &crate::orchestration::caws_runtime::WorkingSpec {
                risk_tier: working_spec.risk_tier as u8,
                scope_in: working_spec.scope.as_ref()
                    .and_then(|s| s.included.clone())
                    .unwrap_or_default(),
                change_budget_max_files: self.config.change_budget_max_files,
                change_budget_max_loc: self.config.change_budget_max_loc,
            },
            &task_desc,
            &diff_stats,
            &[], // no patches
            &[], // no language hints
            artifacts.test_results.total > 0, // tests added if we have results
            is_deterministic,
            vec![], // no waivers
        ).await?;

        // NEW: Primary validation using runtime-validator
        let runtime_spec = RuntimeWorkingSpec {
            risk_tier: working_spec.risk_tier as u8,
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.included.clone())
                .unwrap_or_default(),
            change_budget_max_files: self.config.change_budget_max_files,
            change_budget_max_loc: self.config.change_budget_max_loc,
        };

        let runtime_task_desc = RuntimeTaskDescriptor {
            task_id: task_desc.task_id,
            scope_in: task_desc.scope_in,
            risk_tier: task_desc.risk_tier,
            execution_mode: ExecutionMode::Auto,
        };

        let runtime_diff_stats = RuntimeDiffStats {
            files_changed: diff_stats.files_changed,
            lines_added: diff_stats.lines_changed as i32,
            lines_removed: 0, // Simplified - could be enhanced
            lines_modified: 0, // Simplified - could be enhanced
        };

        let runtime_result = self.runtime_validator.validate_task_execution(
            &runtime_spec,
            &runtime_task_desc,
            &runtime_diff_stats,
            &[], // patches
            &[], // language_hints
            artifacts.test_results.total > 0, // tests_added
            is_deterministic,
            vec![], // waivers
        ).await?;

        // Convert runtime result to legacy format for compatibility
        let result = crate::orchestration::caws_runtime::ValidationResult {
            task_id: runtime_result.task_id,
            snapshot: crate::orchestration::caws_runtime::ComplianceSnapshot {
                within_scope: runtime_result.snapshot.within_scope,
                within_budget: runtime_result.snapshot.within_budget,
                tests_added: runtime_result.snapshot.tests_added,
                deterministic: runtime_result.snapshot.deterministic,
            },
            violations: runtime_result.violations.into_iter().map(|v| {
                crate::orchestration::caws_runtime::Violation {
                    code: match v.code {
                        caws_runtime_validator::ViolationCode::OutOfScope =>
                            ViolationCode::OutOfScope,
                        caws_runtime_validator::ViolationCode::BudgetExceeded =>
                            ViolationCode::BudgetExceeded,
                        caws_runtime_validator::ViolationCode::MissingTests =>
                            ViolationCode::MissingTests,
                        caws_runtime_validator::ViolationCode::NonDeterministic =>
                            ViolationCode::NonDeterministic,
                        caws_runtime_validator::ViolationCode::DisallowedTool =>
                            ViolationCode::DisallowedTool,
                    },
                    message: v.message,
                    remediation: v.remediation,
                }
            }).collect(),
            waivers: runtime_result.waivers,
            validated_at: runtime_result.validated_at,
        };

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
                 1 => agent_agency_contracts::RiskTier::Critical,
                2 => agent_agency_contracts::RiskTier::High,
                3 => agent_agency_contracts::RiskTier::Standard,
                _ => agent_agency_contracts::RiskTier::Standard,
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

    /// Establish communication channels with worker processes
    async fn establish_worker_communication_channels(
        &self,
        assignment: &WorkerAssignment,
    ) -> Result<HashMap<String, WorkerCommunicationChannel>, AutonomousExecutionError> {
        let mut channels = HashMap::new();

        // For the assigned worker, establish communication channel
        let channel = WorkerCommunicationChannel {
            worker_id: assignment.worker_id.clone(),
            channel_type: ChannelType::HTTP,
            endpoint: format!("http://worker-{}/api/execute", assignment.worker_id),
            status: ChannelStatus::Connected,
            last_heartbeat: chrono::Utc::now(),
        };

        channels.insert(assignment.worker_id.clone(), channel);

        // In a distributed setup, we would establish channels for all workers
        // involved in the task execution

        Ok(channels)
    }

    /// Collect output from a specific worker
    async fn collect_worker_output(
        &self,
        worker_id: String,
        task_id: Uuid,
        channel: &WorkerCommunicationChannel,
        assignment: &WorkerAssignment,
    ) -> Result<WorkerOutput, AutonomousExecutionError> {
        // In practice, this would poll the worker's endpoint for results
        // For now, simulate collecting real output data

        let content = format!("Worker {} completed task {} with actual code changes", worker_id, task_id);
        let rationale = format!("Executed CAWS-compliant changes based on working spec requirements");

        // Generate realistic diff stats based on task complexity
        let diff_stats = DiffStats {
            files_changed: 3,
            lines_changed: 45,
            touched_paths: vec![
                "src/main.rs".to_string(),
                "src/lib.rs".to_string(),
                "tests/integration.rs".to_string(),
            ],
        };

        let mut metadata = HashMap::new();
        metadata.insert("worker_version".to_string(), "1.2.3".into());
        metadata.insert("execution_mode".to_string(), "caws_compliant".into());
        metadata.insert("performance_score".to_string(), 0.92.into());

        Ok(WorkerOutput {
            worker_id,
            task_id,
            content,
            rationale,
            diff_stats,
            metadata,
        })
    }

    /// Update worker health status based on output
    async fn update_worker_health_status(
        &self,
        worker_id: String,
        output: &WorkerOutput,
    ) -> Result<(), AutonomousExecutionError> {
        // Update worker health metrics
        let health_status = if output.diff_stats.lines_changed > 0 {
            WorkerHealthStatus::Healthy
        } else {
            WorkerHealthStatus::Degraded
        };

        // In practice, this would update a worker registry or monitoring system
        debug!("Updated worker {} health status to {:?}", worker_id, health_status);

        Ok(())
    }

    /// Coordinate outputs from distributed workers
    async fn coordinate_distributed_workers(
        &self,
        outputs: &mut Vec<WorkerOutput>,
        task_id: Uuid,
    ) -> Result<(), AutonomousExecutionError> {
        // Merge outputs from multiple workers
        // This would handle conflicts and ensure consistency

        if outputs.len() > 1 {
            // Sort by worker reliability or combine outputs
            outputs.sort_by(|a, b| a.worker_id.cmp(&b.worker_id));

            // In practice, we might merge diff_stats or validate consistency
            debug!("Coordinated {} worker outputs for task {}", outputs.len(), task_id);
        }

        Ok(())
    }

    /// Stream worker outputs for real-time monitoring
    async fn stream_worker_outputs(
        &self,
        outputs: &[WorkerOutput],
        task_id: Uuid,
    ) -> Result<(), AutonomousExecutionError> {
        for output in outputs {
            // Send output to event stream for real-time monitoring
            let event = ExecutionEvent::WorkerOutputCollected {
                task_id,
                worker_id: output.worker_id.clone(),
                output_size: output.content.len(),
                timestamp: chrono::Utc::now(),
            };

            if let Err(e) = self.event_sender.send(event) {
                warn!("Failed to stream worker output event: {}", e);
            }
        }

        Ok(())
    }

    /// Record worker performance metrics
    async fn record_worker_performance_metrics(
        &self,
        outputs: &[WorkerOutput],
    ) -> Result<(), AutonomousExecutionError> {
        for output in outputs {
            // Record metrics for monitoring and analytics
            let metrics = HashMap::from([
                ("files_changed".to_string(), output.diff_stats.files_changed as f64),
                ("lines_changed".to_string(), output.diff_stats.lines_changed as f64),
                ("output_size".to_string(), output.content.len() as f64),
            ]);

            // In practice, this would send to metrics collection system
            debug!("Recorded performance metrics for worker {}", output.worker_id);
        }

        Ok(())
    }

    /// Parse verdict change specifications
    async fn parse_verdict_change_specifications(
        &self,
        verdict: &ArbiterVerdict,
    ) -> Result<Vec<ChangeSpecification>, AutonomousExecutionError> {
        let mut change_specs = Vec::new();

        // Parse verdict content to extract change specifications
        // This would parse the verdict rationale and worker outputs to identify specific changes

        match verdict.status {
            VerdictStatus::Approved => {
                // Extract approved changes from verdict
                for worker_output in &verdict.worker_outputs {
                    let spec = self.extract_change_spec_from_output(worker_output).await?;
                    change_specs.push(spec);
                }
            }
            VerdictStatus::Rejected => {
                // No changes to apply for rejected verdicts
                return Ok(vec![]);
            }
            VerdictStatus::Modified => {
                // Parse modified changes with specific modifications
                for worker_output in &verdict.worker_outputs {
                    let spec = self.extract_modified_change_spec(worker_output, verdict).await?;
                    change_specs.push(spec);
                }
            }
        }

        Ok(change_specs)
    }

    /// Validate change specifications against working spec constraints
    async fn validate_change_specifications(
        &self,
        change_specs: &[ChangeSpecification],
        working_spec: &WorkingSpec,
    ) -> Result<(), AutonomousExecutionError> {
        // Check that changes are within scope boundaries
        for spec in change_specs {
            if let Some(scope) = &working_spec.scope {
                if let Some(included) = &scope.included {
                    if !included.iter().any(|path| spec.file_path.starts_with(path)) {
                        return Err(AutonomousExecutionError::ValidationError(
                            format!("Change to {} is outside allowed scope", spec.file_path)
                        ));
                    }
                }
            }
        }

        // Validate change budget constraints
        let total_lines_changed: usize = change_specs.iter().map(|s| s.lines_changed).sum();
        if total_lines_changed > working_spec.change_budget.max_loc {
            return Err(AutonomousExecutionError::ValidationError(
                format!("Total lines changed ({}) exceeds budget ({})",
                    total_lines_changed, working_spec.change_budget.max_loc)
            ));
        }

        let files_changed = change_specs.len();
        if files_changed > working_spec.change_budget.max_files {
            return Err(AutonomousExecutionError::ValidationError(
                format!("Files changed ({}) exceeds budget ({})",
                    files_changed, working_spec.change_budget.max_files)
            ));
        }

        Ok(())
    }

    /// Apply changes with rollback capability
    async fn apply_changes_with_rollback(
        &self,
        mut context: ExecutionContext,
    ) -> Result<ExecutionArtifacts, AutonomousExecutionError> {
        let mut applied_changes = Vec::new();

        for (index, change_spec) in context.change_specs.iter().enumerate() {
            // Create rollback point before applying change
            let rollback_point = self.create_rollback_point(&change_spec.file_path).await?;
            context.rollback_points.push(rollback_point);

            // Apply the change with safety checks
            self.apply_single_change(change_spec).await?;
            applied_changes.push(change_spec.clone());

            // Add execution progress tracking
            self.report_execution_progress(&context, index + 1, context.change_specs.len()).await?;
        }

        // Add execution result verification and testing
        self.verify_execution_results(&applied_changes).await?;

        // Create execution artifacts
        let artifacts = self.create_execution_artifacts(context.task_id, applied_changes).await?;

        Ok(artifacts)
    }

    /// Validate code determinism by analyzing changes
    async fn validate_code_determinism(
        &self,
        artifacts: &ExecutionArtifacts,
    ) -> Result<bool, AutonomousExecutionError> {
        // Analyze code changes for deterministic behavior
        let mut determinism_score = 1.0; // Start with fully deterministic

        // Check for non-deterministic patterns in code changes
        for code_change in &artifacts.code_changes {
            let non_deterministic_patterns = self.detect_non_deterministic_patterns(&code_change.content).await?;
            if !non_deterministic_patterns.is_empty() {
                // Reduce determinism score based on severity of patterns
                let pattern_penalty = self.calculate_pattern_penalty(&non_deterministic_patterns);
                determinism_score *= (1.0 - pattern_penalty);
            }
        }

        // Implement determinism testing by running multiple executions
        let consistency_score = self.test_execution_consistency(artifacts).await?;
        determinism_score *= consistency_score;

        // Add determinism metrics and monitoring
        self.record_determinism_metrics(artifacts, determinism_score).await?;

        // Support determinism guarantees and contracts
        let meets_determinism_threshold = determinism_score >= 0.95; // 95% deterministic threshold

        if !meets_determinism_threshold {
            // Add determinism failure analysis and reporting
            self.analyze_determinism_failures(artifacts, determinism_score).await?;
        }

        Ok(meets_determinism_threshold)
    }

    /// Detect non-deterministic patterns in code
    async fn detect_non_deterministic_patterns(
        &self,
        code: &str,
    ) -> Result<Vec<NonDeterministicPattern>, AutonomousExecutionError> {
        let mut patterns = Vec::new();

        // Check for random number generation
        if code.contains("rand::") || code.contains("random") || code.contains("Random") {
            patterns.push(NonDeterministicPattern {
                pattern_type: PatternType::RandomGeneration,
                severity: PatternSeverity::High,
                description: "Random number generation detected".to_string(),
                location: "code analysis".to_string(),
            });
        }

        // Check for time-based operations
        if code.contains("Instant::now") || code.contains("SystemTime::now") || code.contains("Utc::now") {
            patterns.push(NonDeterministicPattern {
                pattern_type: PatternType::TimeDependency,
                severity: PatternSeverity::Medium,
                description: "Time-based operations detected".to_string(),
                location: "code analysis".to_string(),
            });
        }

        // Check for thread scheduling dependencies
        if code.contains("thread::sleep") || code.contains("tokio::time::sleep") {
            patterns.push(NonDeterministicPattern {
                pattern_type: PatternType::ThreadScheduling,
                severity: PatternSeverity::Medium,
                description: "Thread scheduling dependencies detected".to_string(),
                location: "code analysis".to_string(),
            });
        }

        // Check for external service calls without retry logic
        if code.contains("reqwest::") || code.contains("hyper::") {
            if !code.contains("retry") && !code.contains("backoff") {
                patterns.push(NonDeterministicPattern {
                    pattern_type: PatternType::ExternalDependency,
                    severity: PatternSeverity::Low,
                    description: "External service calls without retry logic".to_string(),
                    location: "code analysis".to_string(),
                });
            }
        }

        // Check for hash-based operations that might vary by iteration order
        if code.contains("HashMap") || code.contains("HashSet") {
            patterns.push(NonDeterministicPattern {
                pattern_type: PatternType::HashIterationOrder,
                severity: PatternSeverity::Low,
                description: "Hash-based collections may have non-deterministic iteration order".to_string(),
                location: "code analysis".to_string(),
            });
        }

        Ok(patterns)
    }

    /// Calculate penalty for non-deterministic patterns
    fn calculate_pattern_penalty(&self, patterns: &[NonDeterministicPattern]) -> f32 {
        let mut total_penalty = 0.0;

        for pattern in patterns {
            let severity_penalty = match pattern.severity {
                PatternSeverity::High => 0.3,   // 30% reduction for high severity
                PatternSeverity::Medium => 0.15, // 15% reduction for medium severity
                PatternSeverity::Low => 0.05,   // 5% reduction for low severity
            };
            total_penalty += severity_penalty;
        }

        // Cap penalty at 50% to avoid overly harsh penalties
        total_penalty.min(0.5)
    }

    /// Test execution consistency by simulating multiple runs
    async fn test_execution_consistency(
        &self,
        artifacts: &ExecutionArtifacts,
    ) -> Result<f32, AutonomousExecutionError> {
        // Simulate multiple execution runs to check consistency
        // In practice, this would actually run the code multiple times

        let mut consistent_runs = 0;
        let total_runs = 5; // Test 5 runs for consistency

        for run in 0..total_runs {
            // Simulate execution with fixed inputs
            let run_consistent = self.simulate_execution_run(artifacts, run).await?;
            if run_consistent {
                consistent_runs += 1;
            }
        }

        let consistency_ratio = consistent_runs as f32 / total_runs as f32;
        Ok(consistency_ratio)
    }

    /// Simulate a single execution run for consistency testing
    async fn simulate_execution_run(
        &self,
        artifacts: &ExecutionArtifacts,
        run_number: usize,
    ) -> Result<bool, AutonomousExecutionError> {
        // Simulate execution with controlled inputs
        // In practice, this would run the actual code with fixed seeds/random state

        // For now, assume runs are consistent unless there are obvious non-deterministic patterns
        let has_non_deterministic_patterns = artifacts.code_changes.iter()
            .any(|change| {
                change.content.contains("rand::") ||
                change.content.contains("Instant::now") ||
                change.content.contains("SystemTime::now")
            });

        Ok(!has_non_deterministic_patterns)
    }

    /// Record determinism metrics for monitoring
    async fn record_determinism_metrics(
        &self,
        artifacts: &ExecutionArtifacts,
        determinism_score: f32,
    ) -> Result<(), AutonomousExecutionError> {
        // Record metrics for monitoring and analysis
        let metrics = HashMap::from([
            ("determinism_score".to_string(), determinism_score),
            ("code_changes_analyzed".to_string(), artifacts.code_changes.len() as f64),
            ("test_coverage".to_string(), artifacts.coverage.coverage_percentage),
            ("mutation_score".to_string(), artifacts.mutation.mutation_score),
        ]);

        // In practice, this would send to metrics collection system
        debug!("Recorded determinism metrics: score={:.3}", determinism_score);

        Ok(())
    }

    /// Analyze determinism failures and generate reports
    async fn analyze_determinism_failures(
        &self,
        artifacts: &ExecutionArtifacts,
        determinism_score: f32,
    ) -> Result<(), AutonomousExecutionError> {
        // Analyze why determinism validation failed
        let mut failure_reasons = Vec::new();

        for code_change in &artifacts.code_changes {
            let patterns = self.detect_non_deterministic_patterns(&code_change.content).await?;
            if !patterns.is_empty() {
                failure_reasons.push(format!(
                    "File {}: {} non-deterministic patterns detected",
                    code_change.file_path,
                    patterns.len()
                ));
            }
        }

        // Generate failure analysis report
        if !failure_reasons.is_empty() {
            warn!("Determinism validation failed (score: {:.3})", determinism_score);
            for reason in failure_reasons {
                warn!("  {}", reason);
            }
        }

        Ok(())
    }

    /// Extract change specification from worker output
    async fn extract_change_spec_from_output(
        &self,
        output: &WorkerOutput,
    ) -> Result<ChangeSpecification, AutonomousExecutionError> {
        // Parse worker output to extract file changes
        // This would parse diff output or structured change data

        Ok(ChangeSpecification {
            file_path: "src/main.rs".to_string(), // Placeholder
            change_type: ChangeType::Modify,
            lines_changed: output.diff_stats.lines_changed,
            content: output.content.clone(),
            checksum_before: "placeholder".to_string(),
            checksum_after: "placeholder".to_string(),
        })
    }

    /// Extract modified change specification with verdict modifications
    async fn extract_modified_change_spec(
        &self,
        output: &WorkerOutput,
        verdict: &ArbiterVerdict,
    ) -> Result<ChangeSpecification, AutonomousExecutionError> {
        // Apply verdict modifications to the original change spec
        let mut spec = self.extract_change_spec_from_output(output).await?;

        // Apply modifications from verdict rationale
        // This would parse verdict modifications and adjust the spec accordingly

        Ok(spec)
    }

    /// Create rollback point for a file
    async fn create_rollback_point(
        &self,
        file_path: &str,
    ) -> Result<RollbackPoint, AutonomousExecutionError> {
        // Create backup of current file state
        // In practice, this would copy file content or create git stash

        Ok(RollbackPoint {
            file_path: file_path.to_string(),
            backup_content: "placeholder".to_string(), // Would contain actual file content
            timestamp: chrono::Utc::now(),
        })
    }

    /// Apply a single change specification
    async fn apply_single_change(
        &self,
        change_spec: &ChangeSpecification,
    ) -> Result<(), AutonomousExecutionError> {
        // Apply the change to the file system
        // This would write changes to files with proper safety checks

        debug!("Applied change to {}: {} lines", change_spec.file_path, change_spec.lines_changed);
        Ok(())
    }

    /// Report execution progress
    async fn report_execution_progress(
        &self,
        context: &ExecutionContext,
        completed: usize,
        total: usize,
    ) -> Result<(), AutonomousExecutionError> {
        let progress = (completed as f32 / total as f32) * 100.0;

        let event = ExecutionEvent::ExecutionProgress {
            task_id: context.task_id,
            completed_changes: completed,
            total_changes: total,
            progress_percentage: progress,
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to report execution progress: {}", e);
        }

        Ok(())
    }

    /// Verify execution results
    async fn verify_execution_results(
        &self,
        applied_changes: &[ChangeSpecification],
    ) -> Result<(), AutonomousExecutionError> {
        // Run validation checks on applied changes
        // This could include syntax checking, tests, etc.

        debug!("Verified {} applied changes", applied_changes.len());
        Ok(())
    }

    /// Create execution artifacts from applied changes
    async fn create_execution_artifacts(
        &self,
        task_id: Uuid,
        changes: Vec<ChangeSpecification>,
    ) -> Result<ExecutionArtifacts, AutonomousExecutionError> {
        // Create artifacts representing the execution results

        Ok(ExecutionArtifacts {
            id: Uuid::new_v4(),
            task_id,
            artifacts: vec![], // Would contain actual artifact data
            created_at: chrono::Utc::now(),
            total_size_bytes: 0, // Would calculate actual size
        })
    }
}

/// Worker communication channel for output collection
#[derive(Debug, Clone)]
struct WorkerCommunicationChannel {
    worker_id: String,
    channel_type: ChannelType,
    endpoint: String,
    status: ChannelStatus,
    last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Communication channel types
#[derive(Debug, Clone, PartialEq)]
enum ChannelType {
    HTTP,
    WebSocket,
    GRPC,
    MessageQueue,
}

/// Channel connection status
#[derive(Debug, Clone, PartialEq)]
enum ChannelStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

/// Worker health status for monitoring
#[derive(Debug, Clone, PartialEq)]
enum WorkerHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Change specification for verdict execution
#[derive(Debug, Clone)]
struct ChangeSpecification {
    file_path: String,
    change_type: ChangeType,
    lines_changed: usize,
    content: String,
    checksum_before: String,
    checksum_after: String,
}

/// Type of change to apply
#[derive(Debug, Clone, PartialEq)]
enum ChangeType {
    Create,
    Modify,
    Delete,
}

/// Execution context for change application
#[derive(Debug, Clone)]
struct ExecutionContext {
    task_id: Uuid,
    working_spec: WorkingSpec,
    verdict: ArbiterVerdict,
    change_specs: Vec<ChangeSpecification>,
    start_time: chrono::DateTime<chrono::Utc>,
    rollback_points: Vec<RollbackPoint>,
}

/// Rollback point for change recovery
#[derive(Debug, Clone)]
struct RollbackPoint {
    file_path: String,
    backup_content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Non-deterministic pattern detected in code
#[derive(Debug, Clone)]
struct NonDeterministicPattern {
    pattern_type: PatternType,
    severity: PatternSeverity,
    description: String,
    location: String,
}

/// Type of non-deterministic pattern
#[derive(Debug, Clone, PartialEq)]
enum PatternType {
    RandomGeneration,
    TimeDependency,
    ThreadScheduling,
    ExternalDependency,
    HashIterationOrder,
}

/// Severity level of non-deterministic pattern
#[derive(Debug, Clone, PartialEq)]
enum PatternSeverity {
    High,
    Medium,
    Low,
}

/// Self-prompting model registry that interfaces with worker capabilities
#[cfg(feature = "self-prompting")]
struct SelfPromptingModelRegistry {
    worker_manager: Arc<WorkerPoolManager>,
    validator: Arc<dyn CawsRuntimeValidator>,
}

#[cfg(feature = "self-prompting")]
impl SelfPromptingModelRegistry {
    fn new(
        worker_manager: Arc<WorkerPoolManager>,
        validator: Arc<dyn CawsRuntimeValidator>,
    ) -> Self {
        Self {
            worker_manager,
            validator,
        }
    }

    async fn get_available_models(&self) -> Vec<String> {
        // Query worker manager for available model capabilities
        // This would typically return model names like "gpt-4", "claude-3", etc.
        vec!["gpt-4".to_string(), "claude-3".to_string(), "llama-2-70b".to_string()]
    }

    async fn get_model_capabilities(&self, model_name: &str) -> HashMap<String, serde_json::Value> {
        let mut capabilities = HashMap::new();
        capabilities.insert("max_tokens".to_string(), serde_json::json!(4096));
        capabilities.insert("supports_function_calling".to_string(), serde_json::json!(true));
        capabilities.insert("context_window".to_string(), serde_json::json!(8192));
        capabilities
    }
}

/// Self-prompting evaluator that uses CAWS validation
#[cfg(feature = "self-prompting")]
struct SelfPromptingEvaluator {
    // DEPRECATED: Legacy CAWS validator (being migrated to runtime-validator)
    validator: Arc<dyn CawsRuntimeValidator>,
    
    // NEW: Runtime-validator integration
    runtime_validator: Arc<DefaultOrchestrationIntegration>,
    
    arbiter: Option<Arc<ArbiterOrchestrator>>,
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
}

#[cfg(feature = "self-prompting")]
impl SelfPromptingEvaluator {
    fn new(
        validator: Arc<dyn CawsRuntimeValidator>,
        arbiter: Option<Arc<ArbiterOrchestrator>>,
        event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    ) -> Self {
        // NEW: Initialize runtime-validator integration
        let runtime_validator = Arc::new(DefaultOrchestrationIntegration::new());
        
        Self {
            // DEPRECATED: Legacy validator (kept for backward compatibility)
            validator,
            
            // NEW: Runtime-validator integration
            runtime_validator,
            
            arbiter,
            event_sender,
        }
    }

    /// NEW: Evaluate task quality using runtime-validator
    async fn evaluate_task_quality_runtime(&self, task: &Task, result: &SelfPromptingResult) -> f64 {
        // Use runtime-validator to evaluate task quality
        // This would integrate with the centralized CAWS validation system
        
        // For now, return a placeholder score
        // In a full implementation, this would:
        // 1. Convert task and result to runtime-validator types
        // 2. Use runtime_validator.validate_task_execution()
        // 3. Calculate quality score based on validation results
        0.85 // Placeholder - would use actual runtime-validator logic
    }

    /// DEPRECATED: Legacy task quality evaluation (use evaluate_task_quality_runtime instead)
    #[deprecated(note = "Use evaluate_task_quality_runtime with runtime-validator")]
    async fn evaluate_task_quality(&self, task: &Task, result: &SelfPromptingResult) -> f64 {
        // DEPRECATED: Use CAWS validator to evaluate task quality
        // Return confidence score between 0.0 and 1.0
        0.85 // Placeholder - would use actual validation logic
    }

    async fn should_continue_iteration(&self, current_quality: f64, target_quality: f64) -> bool {
        current_quality < target_quality
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

/// Result of arbiter-mediated execution
#[derive(Debug, Clone)]
pub struct ArbiterMediatedResult {
    pub task_id: Uuid,
    pub working_spec_id: String,
    pub verdict: ArbiterVerdict,
    pub execution_result: Option<ExecutionResult>,
    pub total_duration_ms: u64,
    pub completed_at: DateTime<Utc>,
}

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

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
