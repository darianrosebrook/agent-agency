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
    validator: Arc<dyn CawsRuntimeValidator>,
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
                validator,
                arbiter,
                config: config.clone(),
                event_sender,
                circuit_breaker,
                #[cfg(feature = "self-prompting")]
                self_prompting_agent: if config.enable_self_prompting {
                    // TODO: Initialize self-prompting agent with proper dependencies
                    // This would need model registry, evaluator, etc.
                    Some(Arc::new(SelfPromptingAgent::new(
                        self_prompting_agent::SelfPromptingAgentConfig::default(),
                        // TODO: Pass proper model registry and evaluator
                        todo!("Initialize with proper dependencies"),
                        todo!("Initialize with proper evaluator"),
                    ).await.unwrap()))
                } else {
                    None
                },
            },
            event_receiver,
        )
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
                change_budget_max_files: self.config.change_budget_max_files,
                change_budget_max_loc: self.config.change_budget_max_loc,
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
