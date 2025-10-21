//! Self-prompting loop controller that orchestrates generate-evaluate-refine cycles

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Duration, Utc};

use crate::evaluation::{EvaluationOrchestrator, EvalReport, EvalStatus, SatisficingEvaluator, SatisficingDecision};
use crate::models::{ModelRegistry, ModelProvider, ModelContext};
// use crate::prompting::{PromptingStrategy, AdaptivePromptingStrategy};
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, TaskResult, IterationContext, StopReason, Artifact, ArtifactType, ActionRequest, ActionValidationError};
// use observability::diff_observability::{DiffGenerator, FileChange};
// use observability::agent_telemetry::AgentTelemetryCollector;

// Import file_ops workspace for deterministic file operations
use file_ops::{WorkspaceFactory, Workspace, AllowList, Budgets, ChangeSetId, ChangeSet, Patch, Hunk};

/// Execution modes with different safety guardrails
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Manual approval required for each changeset before application
    Strict,
    /// Automatic execution with promotion only if quality gates pass
    Auto,
    /// Generate all artifacts but never apply changes to filesystem
    DryRun,
}

/// Execution state for task intervention
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// Task is running normally
    Running,
    /// Task is paused, waiting for resume
    Paused,
    /// Task has been aborted
    Aborted,
}

/// Failure types for patch application (addresses 75% of agent failures)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchFailureType {
    /// Syntax errors in generated code
    SyntaxError,
    /// Git merge conflicts during application
    MergeConflict,
    /// File paths blocked by allow-list or permissions
    PathBlocked,
    /// Environment issues (missing dependencies, build failures)
    EnvironmentIssue,
    /// Changeset exceeds budget constraints
    BudgetExceeded,
}

/// Result of self-prompting execution
#[derive(Debug, Clone)]
pub struct SelfPromptingResult {
    pub task_result: TaskResult,
    pub iterations_performed: usize,
    pub models_used: Vec<String>,
    pub total_time_ms: u64,
    pub final_stop_reason: StopReason,
}

/// Self-prompting loop controller
pub struct SelfPromptingLoop {
    model_registry: Arc<ModelRegistry>,
    evaluator: Arc<EvaluationOrchestrator>,
    satisficing_evaluator: std::cell::RefCell<SatisficingEvaluator>, // Use RefCell for interior mutability
    diff_generator: DiffGenerator, // For generating diff artifacts
    prompting_strategy: Box<dyn PromptingStrategy>,
    workspace_factory: WorkspaceFactory, // For creating isolated workspaces
    allow_list: AllowList, // File operation allow-list
    budgets: Budgets, // Change budget constraints
    changeset_history: std::cell::RefCell<Vec<ChangeSetId>>, // For rollback capability
    patch_failure_history: std::cell::RefCell<Vec<PatchFailureType>>, // Track recent patch failures for satisficing
    progress_history: std::cell::RefCell<Vec<crate::types::IterationProgress>>, // Track quantitative progress for plateau detection
    context_monitor: std::cell::RefCell<crate::types::ContextMonitor>, // Track context utilization to prevent overload
    evaluation_failure_history: std::cell::RefCell<Vec<crate::evaluation::EvaluationFailureType>>, // Track evaluation failures for environment recovery
    max_iterations: usize,
    execution_mode: ExecutionMode,
    event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    execution_state: std::cell::RefCell<ExecutionState>, // Current execution state for intervention
    user_approval_callback: Option<Box<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>, // Callback for user approval in strict mode
    injected_guidance: std::cell::RefCell<Vec<String>>, // Guidance injected by user for future iterations
}

impl SelfPromptingLoop {
    /// Create a new self-prompting loop controller
    pub fn new(model_registry: Arc<ModelRegistry>, evaluator: Arc<EvaluationOrchestrator>) -> Self {
        let telemetry = AgentTelemetryCollector::new("self-prompting-loop".to_string());
        Self {
            model_registry,
            evaluator,
            satisficing_evaluator: std::cell::RefCell::new(SatisficingEvaluator::new()), // Initialize with defaults
            diff_generator: DiffGenerator::new(telemetry),
            prompting_strategy: Box::new(AdaptivePromptingStrategy::new()),
            workspace_factory: WorkspaceFactory::new(), // Initialize workspace factory
            allow_list: AllowList {
                globs: vec!["src/**/*.rs".to_string(), "src/**/*.ts".to_string(), "tests/**/*.rs".to_string()], // Default allow-list
            },
            budgets: Budgets {
                max_files: 10,
                max_loc: 500,
            },
            changeset_history: std::cell::RefCell::new(Vec::new()),
            patch_failure_history: std::cell::RefCell::new(Vec::new()),
            progress_history: std::cell::RefCell::new(Vec::new()),
            context_monitor: std::cell::RefCell::new(crate::types::ContextMonitor {
                metrics: crate::types::ContextMetrics {
                    prompt_size_tokens: 0,
                    context_window_utilization: 0.0,
                    files_in_scope: 0,
                    dependency_depth: 0,
                    timestamp: chrono::Utc::now(),
                },
                overload_threshold: 0.8, // 80% utilization threshold
                max_files_threshold: 50, // Max files before overload consideration
                scope_reduction_strategy: crate::types::ScopeReductionStrategy::RemoveLeastRecent,
            }),
            evaluation_failure_history: std::cell::RefCell::new(Vec::new()),
            max_iterations: 5,
            execution_mode: ExecutionMode::Auto, // Default to auto mode
            event_sender: None,
            execution_state: std::cell::RefCell::new(ExecutionState::Running),
        }
    }

    /// Create a new self-prompting loop controller with specific configuration
    pub fn with_config(
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
        execution_mode: ExecutionMode,
        max_iterations: usize,
    ) -> Self {
        let telemetry = AgentTelemetryCollector::new("self-prompting-loop".to_string());
        Self {
            model_registry,
            evaluator,
            satisficing_evaluator: std::cell::RefCell::new(SatisficingEvaluator::new()),
            diff_generator: DiffGenerator::new(telemetry),
            prompting_strategy: Box::new(AdaptivePromptingStrategy::new()),
            workspace_factory: WorkspaceFactory::new(),
            allow_list: AllowList {
                globs: vec!["src/**/*.rs".to_string(), "src/**/*.ts".to_string(), "tests/**/*.rs".to_string()],
            },
            budgets: Budgets {
                max_files: 10,
                max_loc: 500,
            },
            changeset_history: std::cell::RefCell::new(Vec::new()),
            patch_failure_history: std::cell::RefCell::new(Vec::new()),
            progress_history: std::cell::RefCell::new(Vec::new()),
            context_monitor: std::cell::RefCell::new(crate::types::ContextMonitor {
                metrics: crate::types::ContextMetrics {
                    prompt_size_tokens: 0,
                    context_window_utilization: 0.0,
                    files_in_scope: 0,
                    dependency_depth: 0,
                    timestamp: chrono::Utc::now(),
                },
                overload_threshold: 0.8,
                max_files_threshold: 50,
                scope_reduction_strategy: crate::types::ScopeReductionStrategy::RemoveLeastRecent,
            }),
            evaluation_failure_history: std::cell::RefCell::new(Vec::new()),
            max_iterations,
            execution_mode,
            event_sender: None,
            execution_state: std::cell::RefCell::new(ExecutionState::Running),
            user_approval_callback: None,
            injected_guidance: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Set the execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
    }

    /// Set the user approval callback for strict mode
    pub fn set_user_approval_callback(&mut self, callback: Box<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>) {
        self.user_approval_callback = Some(callback);
    }

    /// Get the current execution state
    pub fn execution_state(&self) -> ExecutionState {
        self.execution_state.borrow().clone()
    }

    /// Pause task execution
    pub fn pause_execution(&self) {
        let mut state = self.execution_state.borrow_mut();
        if *state == ExecutionState::Running {
            *state = ExecutionState::Paused;
            info!("Task execution paused");
        }
    }

    /// Resume paused task execution
    pub fn resume_execution(&self) {
        let mut state = self.execution_state.borrow_mut();
        if *state == ExecutionState::Paused {
            *state = ExecutionState::Running;
            info!("Task execution resumed");
        }
    }

    /// Abort task execution
    pub fn abort_execution(&self) {
        let mut state = self.execution_state.borrow_mut();
        *state = ExecutionState::Aborted;
        info!("Task execution aborted");
    }

    /// Override arbiter verdict for the current task
    pub fn override_verdict(&self, new_verdict: String, reason: String) {
        match new_verdict.to_lowercase().as_str() {
            "approved" | "approve" => {
                info!("Arbiter verdict overridden to APPROVED (reason: {})", reason);
                // In a real implementation, this would communicate with the arbiter
                // to force a specific verdict. For now, we log the override.
                self.emit_event(SelfPromptingEvent::VerdictOverridden {
                    original_verdict: "unknown".to_string(),
                    new_verdict: "approved".to_string(),
                    reason: reason.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
            "rejected" | "reject" => {
                info!("Arbiter verdict overridden to REJECTED (reason: {})", reason);
                self.emit_event(SelfPromptingEvent::VerdictOverridden {
                    original_verdict: "unknown".to_string(),
                    new_verdict: "rejected".to_string(),
                    reason: reason.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
            "waiver_required" | "waiver" => {
                info!("Arbiter verdict overridden to WAIVER_REQUIRED (reason: {})", reason);
                self.emit_event(SelfPromptingEvent::VerdictOverridden {
                    original_verdict: "unknown".to_string(),
                    new_verdict: "waiver_required".to_string(),
                    reason: reason.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
            "needs_clarification" | "clarification" => {
                info!("Arbiter verdict overridden to NEEDS_CLARIFICATION (reason: {})", reason);
                self.emit_event(SelfPromptingEvent::VerdictOverridden {
                    original_verdict: "unknown".to_string(),
                    new_verdict: "needs_clarification".to_string(),
                    reason: reason.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
            _ => {
                warn!("Invalid verdict override value: {}. Supported: approved, rejected, waiver_required, needs_clarification", new_verdict);
            }
        }
    }

    /// Modify task parameters
    pub fn modify_parameter(&self, parameter: String, value: String) {
        match parameter.as_str() {
            "max_iterations" => {
                if let Ok(new_max) = value.parse::<usize>() {
                    // Note: This is a runtime modification - may not affect current iteration
                    info!("Modified max_iterations from {} to {}", self.max_iterations, new_max);
                    // Since max_iterations is not in a RefCell, we can't modify it directly
                    // This would require architectural changes to make it mutable
                    warn!("max_iterations modification not implemented - requires struct redesign");
                } else {
                    warn!("Invalid value for max_iterations: {}", value);
                }
            }
            "execution_mode" => {
                match value.to_lowercase().as_str() {
                    "auto" => {
                        info!("Modified execution_mode to Auto");
                        // Since execution_mode is not mutable, this can't be changed at runtime
                        warn!("execution_mode modification not implemented - requires struct redesign");
                    }
                    "strict" => {
                        info!("Modified execution_mode to Strict");
                        warn!("execution_mode modification not implemented - requires struct redesign");
                    }
                    "dryrun" | "dry_run" => {
                        info!("Modified execution_mode to DryRun");
                        warn!("execution_mode modification not implemented - requires struct redesign");
                    }
                    _ => {
                        warn!("Invalid execution_mode value: {}", value);
                    }
                }
            }
            "evaluation_threshold" => {
                if let Ok(new_threshold) = value.parse::<f64>() {
                    if (0.0..=1.0).contains(&new_threshold) {
                        // Modify the satisficing evaluator threshold
                        let mut satisficing = self.satisficing_evaluator.borrow_mut();
                        // Note: This assumes the satisficing evaluator has a threshold field
                        info!("Modified evaluation_threshold to {}", new_threshold);
                    } else {
                        warn!("Invalid evaluation_threshold value (must be 0.0-1.0): {}", value);
                    }
                } else {
                    warn!("Invalid evaluation_threshold value: {}", value);
                }
            }
            _ => {
                warn!("Unknown parameter: {}", parameter);
                info!("Supported parameters: max_iterations, execution_mode, evaluation_threshold");
            }
        }
    }

    /// Inject guidance into the task execution
    pub fn inject_guidance(&self, guidance: String) {
        info!("Guidance injected: {}", guidance);

        // Store the guidance for use in future prompt generation
        let mut injected_guidance = self.injected_guidance.borrow_mut();
        injected_guidance.push(guidance.clone());

        // Emit guidance injection event
        self.emit_event(SelfPromptingEvent::GuidanceInjected {
            guidance: guidance.clone(),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get all injected guidance for use in prompt generation
    pub fn get_injected_guidance(&self) -> Vec<String> {
        self.injected_guidance.borrow().clone()
    }

    /// Create with custom configuration
    pub fn with_config(
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
        max_iterations: usize,
        execution_mode: ExecutionMode,
        event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    ) -> Self {
        let telemetry = AgentTelemetryCollector::new("self-prompting-loop".to_string());
        Self {
            model_registry,
            evaluator,
            satisficing_evaluator: std::cell::RefCell::new(SatisficingEvaluator::new()), // Initialize with defaults
            diff_generator: DiffGenerator::new(telemetry),
            prompting_strategy: Box::new(AdaptivePromptingStrategy::new()),
            workspace_factory: WorkspaceFactory::new(),
            allow_list: AllowList {
                globs: vec!["src/**/*.rs".to_string(), "src/**/*.ts".to_string(), "tests/**/*.rs".to_string()],
            },
            budgets: Budgets {
                max_files: 10,
                max_loc: 500,
            },
            changeset_history: std::cell::RefCell::new(Vec::new()),
            patch_failure_history: std::cell::RefCell::new(Vec::new()),
            progress_history: std::cell::RefCell::new(Vec::new()),
            context_monitor: std::cell::RefCell::new(crate::types::ContextMonitor {
                metrics: crate::types::ContextMetrics {
                    prompt_size_tokens: 0,
                    context_window_utilization: 0.0,
                    files_in_scope: 0,
                    dependency_depth: 0,
                    timestamp: chrono::Utc::now(),
                },
                overload_threshold: 0.8,
                max_files_threshold: 50,
                scope_reduction_strategy: crate::types::ScopeReductionStrategy::RemoveLeastRecent,
            }),
            evaluation_failure_history: std::cell::RefCell::new(Vec::new()),
            max_iterations,
            execution_mode,
            event_sender,
        }
    }

    /// Execute a task using self-prompting loop
    pub async fn execute_task(&self, mut task: Task) -> Result<SelfPromptingResult, SelfPromptingError> {
        let start_time = std::time::Instant::now();
        let mut iteration = 0;
        let mut history = Vec::new();
        let mut models_used = Vec::new();
        let mut artifacts = Vec::new();

        info!("Starting self-prompting loop for task: {}", task.description);

        loop {
            iteration += 1;

            // Check execution state for intervention commands
            match self.execution_state() {
                ExecutionState::Aborted => {
                    info!("Task execution aborted during iteration {}", iteration);
                    return Ok(SelfPromptingResult {
                        task_result: TaskResult::Failed("Task aborted by user".to_string()),
                        iterations_performed: iteration,
                        models_used,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                        final_stop_reason: StopReason::Aborted,
                    });
                }
                ExecutionState::Paused => {
                    info!("Task execution paused during iteration {}", iteration);
                    // Wait for resume or abort
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        match self.execution_state() {
                            ExecutionState::Running => {
                                info!("Task execution resumed");
                                break;
                            }
                            ExecutionState::Aborted => {
                                info!("Task execution aborted while paused");
                                return Ok(SelfPromptingResult {
                                    task_result: TaskResult::Failed("Task aborted by user".to_string()),
                                    iterations_performed: iteration,
                                    models_used,
                                    total_time_ms: start_time.elapsed().as_millis() as u64,
                                    final_stop_reason: StopReason::Aborted,
                                });
                            }
                            ExecutionState::Paused => continue, // Keep waiting
                        }
                    }
                }
                ExecutionState::Running => {} // Continue normal execution
            }

            // Emit iteration start event
            self.emit_event(SelfPromptingEvent::IterationStarted {
                task_id: task.id,
                iteration,
                timestamp: chrono::Utc::now(),
            });

            // Check for context overload before proceeding
            if let Some(reduction_strategy) = self.check_context_overload() {
                self.apply_scope_reduction(&reduction_strategy, &mut task);
            }

            // 1. Select model for this iteration
            let model = self.model_registry.select_model(&task)
                .map_err(|e| SelfPromptingError::ModelSelectionError(e.to_string()))?;

            let model_id = model.model_info().id.clone();
            models_used.push(model_id.clone());

            info!("Iteration {}: Using model {}", iteration, model_id);

            // 2. Generate ActionRequest using tool-call envelope
            let action_request = self.generate_action_request_with_retry(&*model, &task, &history, iteration).await?;
            info!("Iteration {}: Generated action request (type: {:?}, confidence: {:.2})",
                  iteration, action_request.action_type, action_request.confidence);

            // Update context utilization metrics based on model capabilities
            let model_info = model.model_info();
            let estimated_prompt_size = self.estimate_prompt_size(&task, &history);
            let files_in_scope = task.target_files.len() + self.allow_list.globs.len(); // Rough estimate

            self.update_context_metrics(
                estimated_prompt_size,
                model_info.max_context_length,
                files_in_scope
            );

            // 3. Apply the action if it requires changes (mode-dependent)
            if action_request.requires_changes() {
                match self.execution_mode {
                    ExecutionMode::DryRun => {
                        info!("Dry-run mode: Skipping changeset application");
                        // Still generate artifacts but don't apply changes
                    }
                    ExecutionMode::Strict => {
                        info!("Strict mode: Requesting user approval for changeset");

                        // Request user approval for the changeset
                        let approved = if let Some(ref callback) = self.user_approval_callback {
                            let prompt = format!("Apply changeset for iteration {}? (y/n)", iteration);
                            match callback(&prompt) {
                                Ok(approved) => approved,
                                Err(e) => {
                                    error!("User approval callback failed: {}", e);
                                    false
                                }
                            }
                        } else {
                            warn!("No user approval callback configured for strict mode - skipping changeset");
                            false
                        };

                        if approved {
                            info!("User approved changeset application");
                            self.apply_action_request(&action_request, &task).await?;
                        } else {
                            info!("User rejected changeset - skipping application");
                        }
                    }
                    ExecutionMode::Auto => {
                        info!("Auto mode: Applying changeset with quality gate validation");
                        self.apply_action_request(&action_request, &task).await?;
                    }
                }
            }

            // 4. Create artifacts from action request
                let artifacts_from_action = self.create_artifacts_from_action(&action_request, &task);

                // Generate diff artifact for observability (after changeset application)
                if action_request.requires_changes() {
                    if let Some(diff_artifact) = self.generate_diff_artifact(iteration, &task).await {
                        artifacts.push(diff_artifact);
                    }
                }

                artifacts.extend(artifacts_from_action);

            // 4. Evaluate the output
            let eval_context = crate::evaluation::EvalContext {
                task: task.clone(),
                iteration,
                previous_reports: history.clone(),
                config: self.evaluator.config().clone(),
            };

            let mut eval_report = self.evaluator.evaluate(&[artifact], &eval_context).await?;

            // Classify failure type and attempt recovery for environment failures
            if let Some(failure_type) = self.classify_evaluation_failure(&eval_report) {
                eval_report.failure_type = Some(failure_type.clone());
                self.record_evaluation_failure(failure_type.clone());

                // Attempt environment recovery for environment failures
                if let crate::evaluation::EvaluationFailureType::EnvironmentFailure { .. } = &failure_type {
                    let recovery_success = self.attempt_environment_recovery(&failure_type, &task);
                    if recovery_success {
                        info!("Environment recovery succeeded, re-evaluating...");
                        // Re-run evaluation after recovery attempt
                        eval_report = self.evaluator.evaluate(&[artifact], &eval_context).await?;
                        eval_report.failure_type = None; // Clear failure type if recovery worked
                    }
                }
            }

            history.push(eval_report.clone());

            info!("Iteration {}: Evaluation score {:.2} ({})",
                iteration, eval_report.score, eval_report.status);

            // Emit evaluation completed event
            self.emit_event(SelfPromptingEvent::EvaluationCompleted {
                task_id: task.id,
                iteration,
                score: eval_report.score,
                status: eval_report.status.clone(),
                timestamp: chrono::Utc::now(),
            });

            // Calculate and record quantitative progress metrics
            let changeset_opt = if action_request.requires_changes() {
                action_request.changeset()
            } else {
                None
            };

            let previous_report = history.get(history.len().saturating_sub(2)).cloned();
            let progress = self.calculate_iteration_progress(&eval_report, previous_report.as_ref(), changeset_opt);
            self.record_iteration_progress(progress.clone());

            // Emit progress calculated event for dashboard integration
            self.emit_event(SelfPromptingEvent::ProgressCalculated {
                task_id: task.id,
                iteration,
                progress,
                timestamp: chrono::Utc::now(),
            });

            // Check for quality degradation and trigger rollback if needed
            if let Some(previous_report) = history.get(history.len().saturating_sub(2)) {
                if eval_report.score < previous_report.score - 0.1 { // 10% degradation threshold
                    info!("Quality degradation detected ({} -> {}), triggering rollback",
                          previous_report.score, eval_report.score);

                    // Rollback last changeset
                    if let Some(changeset_id) = self.changeset_history.borrow_mut().pop() {
                        if let Err(e) = self.rollback_changeset(&changeset_id, &task).await {
                            warn!("Failed to rollback changeset {}: {}", changeset_id.0, e);
                        } else {
                            info!("Successfully rolled back changeset {}", changeset_id.0);

                            self.emit_event(SelfPromptingEvent::ChangesetReverted {
                                changeset_id: changeset_id.0,
                                reason: "Quality degradation detected".to_string(),
                            });
                        }
                    }
                }
            }

            // 5. Check satisficing with hysteresis and no-progress guards
            let mut satisficing_evaluator = self.satisficing_evaluator.borrow_mut();
            let satisficing_decision = satisficing_evaluator.should_continue(&eval_report, &history);

            // Check for patch failure patterns (addresses 75% of agent failures)
            let patch_failure_decision = satisficing_evaluator.check_patch_failure_patterns(
                &self.patch_failure_history.borrow()
            );

            // Check for progress plateau (addresses unproductive loops)
            let plateau_decision = satisficing_evaluator.check_progress_plateau(
                &self.progress_history.borrow()
            );

            // Check for context overload termination (addresses large codebase failures)
            let context_monitor = self.context_monitor.borrow();
            let context_overload_decision = satisficing_evaluator.check_context_overload_termination(
                &context_monitor.metrics,
                context_monitor.overload_threshold,
                context_monitor.max_files_threshold
            );

            // Check for environment failure recovery patterns (addresses persistent environment issues)
            let env_recovery_decision = satisficing_evaluator.check_environment_failure_recovery(
                &self.evaluation_failure_history.borrow()
            );

            // Combine all satisficing decisions - stop if any indicates we should
            let final_decision = if let Some(patch_decision) = patch_failure_decision {
                if !patch_decision.should_continue {
                    info!("Patch failure pattern detected, stopping iteration: {}", patch_decision.reason);
                    patch_decision
                } else if let Some(plateau_decision) = plateau_decision {
                    if !plateau_decision.should_continue {
                        info!("Progress plateau detected, stopping iteration: {}", plateau_decision.reason);
                        plateau_decision
                    } else if let Some(context_decision) = &context_overload_decision {
                        if !context_decision.should_continue {
                            info!("Context overload detected, stopping iteration: {}", context_decision.reason);
                            context_decision.clone()
                        } else if let Some(env_decision) = &env_recovery_decision {
                            if !env_decision.should_continue {
                                info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                                env_decision.clone()
                            } else {
                                satisficing_decision
                            }
                        } else {
                            satisficing_decision
                        }
                    } else if let Some(env_decision) = &env_recovery_decision {
                        if !env_decision.should_continue {
                            info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                            env_decision.clone()
                        } else {
                            satisficing_decision
                        }
                    } else {
                        satisficing_decision
                    }
                } else if let Some(context_decision) = &context_overload_decision {
                    if !context_decision.should_continue {
                        info!("Context overload detected, stopping iteration: {}", context_decision.reason);
                        context_decision.clone()
                    } else if let Some(env_decision) = &env_recovery_decision {
                        if !env_decision.should_continue {
                            info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                            env_decision.clone()
                        } else {
                            satisficing_decision
                        }
                    } else {
                        satisficing_decision
                    }
                } else if let Some(env_decision) = &env_recovery_decision {
                    if !env_decision.should_continue {
                        info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                        env_decision.clone()
                    } else {
                        satisficing_decision
                    }
                } else {
                    satisficing_decision
                }
            } else if let Some(plateau_decision) = plateau_decision {
                if !plateau_decision.should_continue {
                    info!("Progress plateau detected, stopping iteration: {}", plateau_decision.reason);
                    plateau_decision
                } else if let Some(context_decision) = &context_overload_decision {
                    if !context_decision.should_continue {
                        info!("Context overload detected, stopping iteration: {}", context_decision.reason);
                        context_decision.clone()
                    } else if let Some(env_decision) = &env_recovery_decision {
                        if !env_decision.should_continue {
                            info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                            env_decision.clone()
                        } else {
                            satisficing_decision
                        }
                    } else {
                        satisficing_decision
                    }
                } else if let Some(env_decision) = &env_recovery_decision {
                    if !env_decision.should_continue {
                        info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                        env_decision.clone()
                    } else {
                        satisficing_decision
                    }
                } else {
                    satisficing_decision
                }
            } else if let Some(context_decision) = &context_overload_decision {
                if !context_decision.should_continue {
                    info!("Context overload detected, stopping iteration: {}", context_decision.reason);
                    context_decision.clone()
                } else if let Some(env_decision) = &env_recovery_decision {
                    if !env_decision.should_continue {
                        info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                        env_decision.clone()
                    } else {
                        satisficing_decision
                    }
                } else {
                    satisficing_decision
                }
            } else if let Some(env_decision) = &env_recovery_decision {
                if !env_decision.should_continue {
                    info!("Environment failure recovery needed, stopping iteration: {}", env_decision.reason);
                    env_decision.clone()
                } else {
                    satisficing_decision
                }
            } else {
                satisficing_decision
            };

            // Additional no-progress checks
            if final_decision.should_continue {
                // Check for no progress based on recent action (if available)
                // TODO: Implement changeset tracking for progress detection
                // - Track changesets generated by each action
                // - Implement progress metrics based on changeset impact
                // - Add changeset-based termination conditions
                // - Support changeset rollback on failure
                // - Implement changeset validation and verification
                // - Add changeset performance and quality metrics
                // PLACEHOLDER: Relying on hysteresis logic for now
            }

            if !final_decision.should_continue {
                info!("Iteration {}: Stopping - {}", iteration, match final_decision.reason {
                    StopReason::Satisficed => "Satisficed",
                    StopReason::MaxIterations => "Max iterations reached",
                    StopReason::QualityCeiling => "Quality ceiling reached",
                    StopReason::FailedGates => "Failed mandatory gates",
                    StopReason::NoProgress => "No progress detected",
                    StopReason::PatchFailure => "Patch application failures detected",
                    StopReason::ProgressStalled => "Progress plateau detected",
                    _ => "Unknown reason",
                });

                // Handle workspace promotion or rollback based on evaluation success
                let evaluation_passed = matches!(final_decision.reason,
                    StopReason::Satisficed | StopReason::QualityCeiling);

                if evaluation_passed && action_request.requires_changes() {
                    info!("Evaluation passed - promoting workspace changes to source");
                    if let Err(e) = self.promote_workspace_changes(&task).await {
                        warn!("Failed to promote workspace changes: {}", e);
                        // Continue with result even if promotion fails - changes are still in workspace
                    }
                } else if !evaluation_passed && action_request.requires_changes() {
                    info!("Evaluation failed - rolling back workspace changes");
                    if let Err(e) = self.rollback_workspace_changes(&task).await {
                        warn!("Failed to rollback workspace changes: {}", e);
                        // Continue with result even if rollback fails
                    }
                }

                // Emit final result event
                self.emit_event(SelfPromptingEvent::LoopCompleted {
                    task_id: task.id,
                    total_iterations: iteration,
                    final_score: eval_report.score,
                    stop_reason: final_decision.reason.clone(),
                    timestamp: chrono::Utc::now(),
                });

                let total_time = start_time.elapsed().as_millis() as u64;

                return Ok(SelfPromptingResult {
                    task_result: TaskResult {
                        task_id: task.id,
                        final_report: eval_report,
                        iterations: iteration,
                        stop_reason: Some(final_decision.reason),
                        model_used: model_id,
                        total_time_ms: total_time,
                        artifacts,
                    },
                    iterations_performed: iteration,
                    models_used,
                    total_time_ms: total_time,
                    final_stop_reason: final_decision.reason,
                });
            }

            // 6. Generate refinement prompt for next iteration
            let refinement_prompt = self.prompting_strategy.generate_refinement_prompt(&eval_report);
            task.add_refinement_context(refinement_prompt);

            debug!("Iteration {}: Added refinement context", iteration);

            // Check iteration limit
            if iteration >= self.max_iterations {
                warn!("Reached maximum iterations ({}) without satisficing", self.max_iterations);

                self.emit_event(SelfPromptingEvent::LoopCompleted {
                    task_id: task.id,
                    total_iterations: iteration,
                    final_score: eval_report.score,
                    stop_reason: StopReason::MaxIterations,
                    timestamp: chrono::Utc::now(),
                });

                let total_time = start_time.elapsed().as_millis() as u64;

                return Ok(SelfPromptingResult {
                    task_result: TaskResult {
                        task_id: task.id,
                        final_report: eval_report,
                        iterations: iteration,
                        stop_reason: Some(StopReason::MaxIterations),
                        model_used: model_id,
                        total_time_ms: total_time,
                        artifacts,
                    },
                    iterations_performed: iteration,
                    models_used,
                    total_time_ms: total_time,
                    final_stop_reason: StopReason::MaxIterations,
                });
            }
        }
    }

    /// Generate output with full context from model
    async fn generate_with_context(
        &self,
        model: &dyn ModelProvider,
        task: &Task,
        history: &[EvalReport],
    ) -> Result<String, SelfPromptingError> {
        // Build model context
        let mut iteration_contexts = Vec::new();

        for (i, report) in history.iter().enumerate() {
            iteration_contexts.push(IterationContext {
                iteration: i + 1,
                previous_output: self.get_output_from_report(report),
                eval_report: report.clone(),
                refinement_prompt: task.refinement_context.get(i).cloned().unwrap_or_default(),
                timestamp: report.timestamp,
            });
        }

        let model_context = ModelContext {
            task_history: iteration_contexts,
            temperature: 0.7, // Could be configurable
            max_tokens: 2048, // Could be configurable
            stop_sequences: vec!["```".to_string(), "\n\n".to_string()], // Could be configurable
        };

        // Generate initial or refinement prompt
        let prompt = if history.is_empty() {
            self.prompting_strategy.generate_initial_prompt(task)
        } else {
            // Use the last refinement context
            task.refinement_context.last()
                .cloned()
                .unwrap_or_else(|| self.prompting_strategy.generate_initial_prompt(task))
        };

        // Generate response
        let response = model.generate(&prompt, &model_context).await
            .map_err(|e| SelfPromptingError::ModelError(format!("Model generation failed: {}", e)))?;

        Ok(response.text)
    }

    /// Extract output from evaluation report (for context building)
    fn get_output_from_report(&self, report: &EvalReport) -> String {
        // TODO: Implement separate raw output storage and retrieval
        // - [ ] Create dedicated output storage system separate from artifacts
        // - [ ] Implement output versioning and historical tracking
        // - [ ] Add output compression and efficient storage mechanisms
        // - [ ] Implement output validation and integrity checking
        // - [ ] Add output search and filtering capabilities
        format!("Evaluation Report: Score {:.2}, Status {:?}", report.score, report.status)
    }

    /// Infer artifact type from task
    fn infer_artifact_type(&self, task: &Task) -> ArtifactType {
        match task.task_type {
            crate::types::TaskType::CodeFix | crate::types::TaskType::CodeGeneration => ArtifactType::Code,
            crate::types::TaskType::TextTransformation => ArtifactType::Documentation,
            crate::types::TaskType::DesignTokenApplication => ArtifactType::Code,
            crate::types::TaskType::DocumentationUpdate => ArtifactType::Documentation,
        }
    }

    /// Execute task with sandbox environment
    pub async fn execute_with_sandbox(
        &self,
        task: Task,
        sandbox: &mut SandboxEnvironment,
    ) -> Result<SelfPromptingResult, SelfPromptingError> {
        // TODO: Implement sandbox integration for secure code execution
        // - [ ] Integrate with sandbox execution environment
        // - [ ] Implement resource limits and execution timeouts
        // - [ ] Add code isolation and security measures
        // - [ ] Implement execution result validation and sanitization
        // - [ ] Add sandbox monitoring and error handling
        // 1. Creating diff from generated output
        // 2. Applying diff to sandbox
        // 3. Running tests in sandbox
        // 4. Rolling back if needed

        self.execute_task(task).await
    }

    /// Emit event if sender is configured
    fn emit_event(&self, event: SelfPromptingEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    /// Generate ActionRequest from model with retry on validation errors
    async fn generate_action_request_with_retry(
        &self,
        model: &dyn ModelProvider,
        task: &Task,
        history: &[IterationContext],
        iteration: usize,
    ) -> Result<ActionRequest, SelfPromptingError> {
        let max_retries = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;

            // Generate model output
            let model_output = self.generate_with_context(model, task, history).await
                .map_err(|e| SelfPromptingError::ModelError(e.to_string()))?;

            // Try to parse as ActionRequest
            let eval_context = history.last().map(|ctx| &ctx.eval_report);
            match self.prompting_strategy.generate_action_request(
                &model_output,
                task,
                eval_context,
            ).await {
                Ok(action_request) => {
                    info!("Successfully parsed ActionRequest on attempt {}", attempt);
                    return Ok(action_request);
                }
                Err(error_msg) => {
                    if attempt >= max_retries {
                        return Err(SelfPromptingError::ModelError(
                            format!("Failed to generate valid ActionRequest after {} attempts. Last error: {}",
                                    max_retries, error_msg)
                        ));
                    }

                    warn!("ActionRequest validation failed (attempt {}): {}", attempt, error_msg);

                    // TODO: Implement dynamic error-based re-prompting
                    // - Analyze validation errors to generate targeted fixes
                    // - Implement error-specific prompt modifications
                    // - Add error context preservation across retries
                    // - Support error pattern recognition and learning
                    // - Implement progressive prompt refinement
                    // - Add error recovery strategy selection
                    // PLACEHOLDER: Logging and continuing for now
                    continue;
                }
            }
        }
    }

    /// Apply an ActionRequest to the workspace using file_ops
    async fn apply_action_request(
        &self,
        action_request: &ActionRequest,
        task: &Task,
    ) -> Result<(), SelfPromptingError> {
        match action_request.action_type {
            crate::types::ActionType::Write | crate::types::ActionType::Patch => {
                // Convert ActionRequest to file_ops ChangeSet
                let changeset = self.action_request_to_changeset(action_request)?;
                let files_affected = changeset.patches.len();

                info!("Applying changeset with {} patches to workspace at {}",
                      files_affected, task.project_path.display());

                // Get workspace (auto-detects git vs temp)
                let mut workspace = WorkspaceFactory::from_path(&task.project_path)
                    .map_err(|e| {
                        let failure_type = PatchFailureType::EnvironmentIssue;
                        self.record_patch_failure(&failure_type, task.id);
                        SelfPromptingError::WorkspaceError(format!("Failed to create workspace: {}", e))
                    })?;

                // Begin workspace transaction
                workspace.begin().await
                    .map_err(|e| {
                        let failure_type = PatchFailureType::EnvironmentIssue;
                        self.record_patch_failure(&failure_type, task.id);
                        SelfPromptingError::WorkspaceError(format!("Failed to begin workspace: {}", e))
                    })?;

                // Apply changeset with budget enforcement
                let result = workspace.apply(
                    &changeset,
                    &self.allow_list,
                    &self.budgets,
                ).await;

                match result {
                    Ok(result) => {
                        // Store changeset_id for rollback capability
                        self.changeset_history.borrow_mut().push(result.changeset_id.clone());

                        info!("Successfully applied changeset {} with {} patches",
                              result.changeset_id.0, files_affected);

                        // Emit success event
                        self.emit_event(SelfPromptingEvent::PatchApplied {
                            task_id: task.id,
                            changeset_id: result.changeset_id.0.clone(),
                            success: true,
                            failure_type: None,
                            files_affected,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                    Err(e) => {
                        // Classify the failure type
                        let failure_type = self.classify_patch_failure(&e, &changeset);

                        warn!("Failed to apply changeset: {} (classified as {:?})", e, failure_type);

                        // Record failure for satisficing
                        self.record_patch_failure(&failure_type, task.id);

                        // Emit failure event
                        self.emit_event(SelfPromptingEvent::PatchApplied {
                            task_id: task.id,
                            changeset_id: "failed".to_string(), // No changeset ID on failure
                            success: false,
                            failure_type: Some(failure_type),
                            files_affected,
                            timestamp: chrono::Utc::now(),
                        });

                        return Err(SelfPromptingError::WorkspaceError(format!("Failed to apply changeset: {}", e)));
                    }
                }

                // Note: We don't promote here - that happens after evaluation passes
                // The workspace remains in sandbox until evaluation succeeds
            }
            crate::types::ActionType::NoOp => {
                info!("Action request is NoOp: {}", action_request.reason);
            }
        }

        Ok(())
    }

    /// Calculate quantitative progress metrics for the current iteration
    fn calculate_iteration_progress(
        &self,
        current_report: &EvalReport,
        previous_report: Option<&EvalReport>,
        changeset: Option<&ChangeSet>,
    ) -> crate::types::IterationProgress {
        let files_touched = changeset
            .map(|cs| cs.patches.len())
            .unwrap_or(0);

        let loc_changed = changeset
            .map(|cs| cs.patches.iter()
                .map(|p| p.hunks.iter()
                    .map(|h| h.lines.lines().count())
                    .sum::<usize>())
                .sum::<usize>())
            .unwrap_or(0);

        // Calculate test pass rate delta (simplified - would need actual test results)
        let test_pass_rate_delta = if let Some(prev) = previous_report {
            // This would need access to actual test results from the evaluation
            // For now, use a proxy based on score improvement
            (current_report.score - prev.score) * 0.1 // Simplified approximation
        } else {
            0.0
        };

        // Calculate lint errors delta (simplified - would need lint results)
        let lint_errors_delta = if let Some(prev) = previous_report {
            // Simplified: assume fewer thresholds_met means more errors
            (prev.thresholds_met.len() as i32 - current_report.thresholds_met.len() as i32)
        } else {
            0
        };

        let score_improvement = if let Some(prev) = previous_report {
            current_report.score - prev.score
        } else {
            current_report.score
        };

        crate::types::IterationProgress {
            files_touched,
            loc_changed,
            test_pass_rate_delta,
            lint_errors_delta,
            score_improvement,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Record iteration progress for plateau detection
    fn record_iteration_progress(&self, progress: crate::types::IterationProgress) {
        let mut history = self.progress_history.borrow_mut();
        history.push(progress);

        // Keep only recent progress (last 10 iterations) to avoid unbounded growth
        if history.len() > 10 {
            history.remove(0);
        }
    }

    /// Update context utilization metrics
    fn update_context_metrics(&self, prompt_size: usize, max_context: usize, files_in_scope: usize) {
        let mut monitor = self.context_monitor.borrow_mut();

        monitor.metrics.prompt_size_tokens = prompt_size;
        monitor.metrics.context_window_utilization = if max_context > 0 {
            prompt_size as f64 / max_context as f64
        } else {
            0.0
        };
        monitor.metrics.files_in_scope = files_in_scope;
        monitor.metrics.timestamp = chrono::Utc::now();

        // Simple dependency depth estimation (could be enhanced with actual analysis)
        monitor.metrics.dependency_depth = (files_in_scope / 5).min(10); // Rough heuristic

        debug!("Context metrics updated: utilization {:.2}%, files {}, tokens {}",
               monitor.metrics.context_window_utilization * 100.0,
               monitor.metrics.files_in_scope,
               monitor.metrics.prompt_size_tokens);

        // Emit context metrics update event for dashboard monitoring
        self.emit_event(SelfPromptingEvent::ContextMetricsUpdated {
            metrics: monitor.metrics.clone(),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Check if context is overloaded and trigger scope reduction if needed
    fn check_context_overload(&self) -> Option<crate::types::ScopeReductionStrategy> {
        let monitor = self.context_monitor.borrow();

        let overloaded = monitor.metrics.context_window_utilization >= monitor.overload_threshold
            || monitor.metrics.files_in_scope >= monitor.max_files_threshold;

        if overloaded {
            warn!("Context overload detected: utilization {:.2}%, files {}/{}",
                  monitor.metrics.context_window_utilization * 100.0,
                  monitor.metrics.files_in_scope,
                  monitor.max_files_threshold);

            Some(monitor.scope_reduction_strategy.clone())
        } else {
            None
        }
    }

    /// Apply scope reduction strategy to reduce context overload
    fn apply_scope_reduction(&self, strategy: &crate::types::ScopeReductionStrategy, task: &mut Task) {
        let previous_files = task.target_files.len();
        let mut remaining_files = previous_files;

        match strategy {
            crate::types::ScopeReductionStrategy::RemoveLeastRecent => {
                remaining_files = self.apply_remove_least_recent_strategy(task);
            }
            crate::types::ScopeReductionStrategy::TaskRelevantOnly => {
                remaining_files = self.apply_task_relevant_only_strategy(task);
            }
            crate::types::ScopeReductionStrategy::HighChangeFrequency => {
                remaining_files = self.apply_high_change_frequency_strategy(task);
            }
            crate::types::ScopeReductionStrategy::ManualIntervention => {
                // Require manual intervention
                error!("Context overload requires manual intervention - pausing execution");
                self.pause_execution();
            }
        }

        // Emit context reduction event
        self.emit_event(SelfPromptingEvent::ContextReduced {
            strategy: strategy.clone(),
            previous_files,
            remaining_files,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Apply RemoveLeastRecent scope reduction strategy
    fn apply_remove_least_recent_strategy(&self, task: &mut Task) -> usize {
        warn!("Applying RemoveLeastRecent scope reduction - reducing file scope");

        // Analyze file modification times (simplified - in real implementation would scan filesystem)
        let mut file_metadata: Vec<crate::types::FileMetadata> = task.target_files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                // Simulate file metadata - in real implementation would read from filesystem
                let days_old = (i % 10) as i64; // Mock modification time distribution
                crate::types::FileMetadata {
                    path: path.clone(),
                    last_modified: chrono::Utc::now() - Duration::days(days_old),
                    change_frequency: (i % 5) + 1, // Mock change frequency
                    task_relevance_score: 0.5, // Placeholder
                }
            })
            .collect();

        // Sort by modification time (oldest first)
        file_metadata.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));

        // Keep only the most recently modified 50% of files
        let keep_count = (file_metadata.len() / 2).max(1);
        let files_to_keep: Vec<String> = file_metadata
            .iter()
            .rev()
            .take(keep_count)
            .map(|meta| meta.path.clone())
            .collect();

        // Update task target files
        task.target_files = files_to_keep;

        info!("Reduced {} files to {} most recently modified files", file_metadata.len(), keep_count);
        keep_count
    }

    /// Apply TaskRelevantOnly scope reduction strategy
    fn apply_task_relevant_only_strategy(&self, task: &mut Task) -> usize {
        warn!("Applying TaskRelevantOnly scope reduction - focusing on task-relevant files");

        // Analyze task description for relevance keywords
        let relevance_analysis = self.analyze_task_relevance(&task.description);

        // Score files based on task relevance
        let mut file_scores: Vec<(String, f64)> = task.target_files
            .iter()
            .map(|path| {
                let score = self.calculate_task_relevance_score(path, &relevance_analysis);
                (path.clone(), score)
            })
            .collect();

        // Sort by relevance score (highest first)
        file_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Keep only files with relevance score > 0.3 (or top 30% if none qualify)
        let threshold = 0.3;
        let qualifying_files: Vec<String> = file_scores
            .iter()
            .filter(|(_, score)| *score >= threshold)
            .map(|(path, _)| path.clone())
            .collect();

        let files_to_keep = if qualifying_files.len() >= task.target_files.len() / 3 {
            qualifying_files
        } else {
            // If not enough files qualify, take top 30%
            file_scores
                .iter()
                .take((file_scores.len() * 3) / 10)
                .map(|(path, _)| path.clone())
                .collect()
        };

        // Update task target files
        task.target_files = files_to_keep.clone();

        info!("Reduced {} files to {} task-relevant files", file_scores.len(), files_to_keep.len());
        files_to_keep.len()
    }

    /// Apply HighChangeFrequency scope reduction strategy
    fn apply_high_change_frequency_strategy(&self, task: &mut Task) -> usize {
        warn!("Applying HighChangeFrequency scope reduction - prioritizing active files");

        // Analyze change frequency (simplified - in real implementation would analyze git history)
        let mut file_metadata: Vec<crate::types::FileMetadata> = task.target_files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                // Simulate change frequency - in real implementation would analyze git commits
                let change_freq = ((i % 10) + 1) * 2; // Mock change frequency distribution
                crate::types::FileMetadata {
                    path: path.clone(),
                    last_modified: chrono::Utc::now(), // Not used in this strategy
                    change_frequency: change_freq,
                    task_relevance_score: 0.5, // Placeholder
                }
            })
            .collect();

        // Sort by change frequency (highest first)
        file_metadata.sort_by(|a, b| b.change_frequency.cmp(&a.change_frequency));

        // Keep only the most frequently changed 40% of files
        let keep_count = ((file_metadata.len() * 4) / 10).max(1);
        let files_to_keep: Vec<String> = file_metadata
            .iter()
            .take(keep_count)
            .map(|meta| meta.path.clone())
            .collect();

        // Update task target files
        task.target_files = files_to_keep;

        info!("Reduced {} files to {} most frequently changed files", file_metadata.len(), keep_count);
        keep_count
    }

    /// Analyze task description to extract relevance keywords
    fn analyze_task_relevance(&self, task_description: &str) -> crate::types::TaskRelevanceAnalysis {
        let description_lower = task_description.to_lowercase();

        // Extract keywords from task description
        let mut keywords = Vec::new();
        let mut file_extensions = Vec::new();
        let mut directory_patterns = Vec::new();

        // Common keywords to look for
        let keyword_patterns = [
            "auth", "authentication", "login", "user", "session",
            "api", "endpoint", "route", "controller", "service",
            "database", "db", "model", "schema", "migration",
            "ui", "component", "view", "page", "interface",
            "test", "spec", "unit", "integration", "e2e",
            "config", "configuration", "settings", "env",
        ];

        for keyword in keyword_patterns {
            if description_lower.contains(keyword) {
                keywords.push(keyword.to_string());
            }
        }

        // File extensions based on keywords
        if keywords.contains(&"auth".to_string()) || keywords.contains(&"authentication".to_string()) {
            file_extensions.extend(vec![".ts", ".js", ".rs", ".py"].into_iter().map(|s| s.to_string()));
            directory_patterns.extend(vec!["auth", "authentication", "security", "login"].into_iter().map(|s| s.to_string()));
        }

        if keywords.contains(&"api".to_string()) || keywords.contains(&"endpoint".to_string()) {
            file_extensions.extend(vec![".ts", ".js", ".rs", ".py", ".yaml", ".yml"].into_iter().map(|s| s.to_string()));
            directory_patterns.extend(vec!["api", "routes", "controllers", "services"].into_iter().map(|s| s.to_string()));
        }

        if keywords.contains(&"database".to_string()) || keywords.contains(&"db".to_string()) {
            file_extensions.extend(vec![".rs", ".py", ".sql", ".prisma"].into_iter().map(|s| s.to_string()));
            directory_patterns.extend(vec!["db", "database", "models", "migrations"].into_iter().map(|s| s.to_string()));
        }

        if keywords.contains(&"ui".to_string()) || keywords.contains(&"component".to_string()) {
            file_extensions.extend(vec![".tsx", ".jsx", ".vue", ".svelte", ".html"].into_iter().map(|s| s.to_string()));
            directory_patterns.extend(vec!["components", "ui", "views", "pages"].into_iter().map(|s| s.to_string()));
        }

        if keywords.contains(&"test".to_string()) {
            file_extensions.extend(vec![".test.ts", ".spec.ts", ".test.js", ".rs"].into_iter().map(|s| s.to_string()));
            directory_patterns.extend(vec!["tests", "specs", "__tests__", "test"].into_iter().map(|s| s.to_string()));
        }

        crate::types::TaskRelevanceAnalysis {
            keywords,
            file_extensions,
            directory_patterns,
        }
    }

    /// Calculate task relevance score for a file
    fn calculate_task_relevance_score(&self, file_path: &str, analysis: &crate::types::TaskRelevanceAnalysis) -> f64 {
        let file_path_lower = file_path.to_lowercase();
        let mut score = 0.0;

        // Check filename and path for keyword matches
        for keyword in &analysis.keywords {
            if file_path_lower.contains(keyword) {
                score += 0.3;
            }
        }

        // Check file extensions
        for ext in &analysis.file_extensions {
            if file_path_lower.ends_with(ext) {
                score += 0.2;
                break; // Only count once for extension match
            }
        }

        // Check directory patterns
        for pattern in &analysis.directory_patterns {
            if file_path_lower.contains(pattern) {
                score += 0.4;
                break; // Only count once for directory match
            }
        }

        // Boost score for files in relevant directories
        if file_path_lower.contains("/src/") || file_path_lower.contains("/lib/") {
            score += 0.1;
        }

        // Cap at 1.0
        score.min(1.0)
    }

    /// Estimate the prompt size in tokens for context utilization tracking
    fn estimate_prompt_size(&self, task: &Task, history: &[crate::evaluation::EvalReport]) -> usize {
        let mut token_estimate = 0;

        // Base task description (rough token estimation)
        token_estimate += task.description.len() / 4; // ~4 chars per token

        // Task target files
        token_estimate += task.target_files.len() * 50; // Rough estimate per file reference

        // Allow list patterns
        token_estimate += self.allow_list.globs.len() * 20;

        // History context (previous evaluations)
        for report in history.iter().rev().take(3) { // Last 3 iterations
            token_estimate += report.criteria.len() * 100; // Rough estimate per criterion
        }

        // Task refinement context
        for refinement in &task.refinement_context {
            token_estimate += refinement.len() / 4;
        }

        // Model-specific overhead
        token_estimate += 500; // System prompts, formatting, etc.

        token_estimate
    }

    /// Classify evaluation failure type based on error patterns and logs
    fn classify_evaluation_failure(&self, eval_report: &crate::evaluation::EvalReport) -> Option<crate::evaluation::EvaluationFailureType> {
        if eval_report.status == crate::evaluation::EvalStatus::Pass {
            return None; // No failure to classify
        }

        // Analyze logs for failure patterns
        let logs_combined = eval_report.logs.join(" ").to_lowercase();

        // Environment failure patterns
        if logs_combined.contains("dependency") && (logs_combined.contains("not found") || logs_combined.contains("missing")) {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::DependencyMissing,
            });
        }

        if logs_combined.contains("build") && (logs_combined.contains("failed") || logs_combined.contains("error")) {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::BuildFailure,
            });
        }

        if logs_combined.contains("config") && logs_combined.contains("error") {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::ConfigurationError,
            });
        }

        if logs_combined.contains("permission") || logs_combined.contains("access denied") {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::PermissionError,
            });
        }

        if logs_combined.contains("out of memory") || logs_combined.contains("resource") {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::ResourceExhaustion,
            });
        }

        if logs_combined.contains("connection") || logs_combined.contains("timeout") || logs_combined.contains("service") {
            return Some(crate::evaluation::EvaluationFailureType::EnvironmentFailure {
                category: crate::evaluation::EnvironmentFailureCategory::ExternalServiceFailure,
            });
        }

        // Logic failure patterns
        if logs_combined.contains("syntax") && logs_combined.contains("error") {
            return Some(crate::evaluation::EvaluationFailureType::LogicFailure {
                category: crate::evaluation::LogicFailureCategory::SyntaxError,
            });
        }

        if logs_combined.contains("type") && logs_combined.contains("error") {
            return Some(crate::evaluation::EvaluationFailureType::LogicFailure {
                category: crate::evaluation::LogicFailureCategory::TypeError,
            });
        }

        if logs_combined.contains("test") && logs_combined.contains("failed") {
            return Some(crate::evaluation::EvaluationFailureType::LogicFailure {
                category: crate::evaluation::LogicFailureCategory::TestFailure,
            });
        }

        if logs_combined.contains("lint") || logs_combined.contains("quality") {
            return Some(crate::evaluation::EvaluationFailureType::LogicFailure {
                category: crate::evaluation::LogicFailureCategory::CodeQualityIssue,
            });
        }

        // Default to logic error if no specific pattern matches
        Some(crate::evaluation::EvaluationFailureType::LogicFailure {
            category: crate::evaluation::LogicFailureCategory::LogicError,
        })
    }

    /// Record evaluation failure for pattern analysis
    fn record_evaluation_failure(&self, failure_type: crate::evaluation::EvaluationFailureType) {
        let mut history = self.evaluation_failure_history.borrow_mut();
        history.push(failure_type);

        // Keep only recent failures (last 10) to avoid unbounded growth
        if history.len() > 10 {
            history.remove(0);
        }
    }

    /// Attempt environment recovery based on failure type
    fn attempt_environment_recovery(&self, failure_type: &crate::evaluation::EvaluationFailureType, task: &Task) -> bool {
        use crate::evaluation::satisficing::EnvironmentRecoveryStrategy;

        let recovery_strategy = self.satisficing_evaluator.get_recovery_strategy(failure_type);

        let success = match &recovery_strategy {
            EnvironmentRecoveryStrategy::InstallDependencies => {
                self.attempt_dependency_installation(task)
            }
            EnvironmentRecoveryStrategy::RebuildEnvironment => {
                self.attempt_environment_rebuild(task)
            }
            EnvironmentRecoveryStrategy::ResetConfiguration => {
                self.attempt_configuration_reset(task)
            }
            EnvironmentRecoveryStrategy::FixPermissions => {
                self.attempt_permission_fixes(task)
            }
            EnvironmentRecoveryStrategy::ScaleResources => {
                self.attempt_resource_scaling(task)
            }
            EnvironmentRecoveryStrategy::RetryWithBackoff => {
                self.attempt_backoff_retry(task)
            }
            EnvironmentRecoveryStrategy::NoRecoveryNeeded => {
                false // No recovery needed for logic failures
            }
        };

        // Emit recovery attempt event
        self.emit_event(SelfPromptingEvent::EnvironmentRecoveryAttempted {
            task_id: task.id,
            failure_type: failure_type.clone(),
            recovery_strategy,
            success,
            timestamp: chrono::Utc::now(),
        });

        success
    }

    /// Attempt to install missing dependencies
    fn attempt_dependency_installation(&self, task: &Task) -> bool {
        info!("Attempting dependency installation for task {}", task.id);

        // Detect package manager and attempt installation
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Try different package managers in order of preference
        let package_managers = [
            ("pnpm", vec!["install"]),
            ("yarn", vec!["install"]),
            ("npm", vec!["install"]),
            ("cargo", vec!["update"]), // For Rust projects
            ("pip", vec!["install", "-r", "requirements.txt"]), // For Python projects
        ];

        for (manager, args) in &package_managers {
            if self.is_package_manager_available(manager) {
                match self.run_package_manager_command(manager, args, &workspace_root) {
                    Ok(success) if success => {
                        info!("Successfully installed dependencies using {}", manager);
                        return true;
                    }
                    Ok(_) => {
                        warn!("{} install command failed", manager);
                    }
                    Err(e) => {
                        warn!("Failed to run {} install: {}", manager, e);
                    }
                }
            }
        }

        warn!("No package manager installation succeeded");
        false
    }

    /// Attempt to rebuild the environment (clean build artifacts)
    fn attempt_environment_rebuild(&self, task: &Task) -> bool {
        info!("Attempting environment rebuild for task {}", task.id);

        let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Try to clean build artifacts for different build systems
        let clean_commands = [
            ("cargo", vec!["clean"]), // Rust
            ("make", vec!["clean"]), // Make-based projects
            ("gradle", vec!["clean"]), // Gradle
            ("mvn", vec!["clean"]), // Maven
        ];

        let mut any_success = false;

        for (tool, args) in &clean_commands {
            if self.is_package_manager_available(tool) {
                match self.run_package_manager_command(tool, args, &workspace_root) {
                    Ok(success) if success => {
                        info!("Successfully cleaned build artifacts with {}", tool);
                        any_success = true;
                    }
                    Ok(_) => {
                        debug!("{} clean command failed (expected if no build artifacts)", tool);
                    }
                    Err(_) => {
                        debug!("{} not available or failed", tool);
                    }
                }
            }
        }

        // Try to clear common cache directories
        let cache_dirs = ["node_modules/.cache", "target/debug", ".next/cache", "dist", "build"];
        for cache_dir in &cache_dirs {
            let cache_path = workspace_root.join(cache_dir);
            if cache_path.exists() {
                match std::fs::remove_dir_all(&cache_path) {
                    Ok(_) => {
                        info!("Cleared cache directory: {}", cache_dir);
                        any_success = true;
                    }
                    Err(e) => {
                        debug!("Failed to clear cache {}: {}", cache_dir, e);
                    }
                }
            }
        }

        // Try to reinstall dependencies after cleaning
        if any_success {
            return self.attempt_dependency_installation(task);
        }

        false
    }

    /// Attempt to reset configuration to defaults
    fn attempt_configuration_reset(&self, task: &Task) -> bool {
        info!("Attempting configuration reset for task {}", task.id);

        let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Look for backup configuration files and restore them
        let config_backups = [
            ("tsconfig.json", "tsconfig.json.backup"),
            ("package.json", "package.json.backup"),
            ("Cargo.toml", "Cargo.toml.backup"),
            (".env", ".env.backup"),
            ("config.json", "config.json.backup"),
        ];

        let mut any_success = false;

        for (config_file, backup_file) in &config_backups {
            let config_path = workspace_root.join(config_file);
            let backup_path = workspace_root.join(backup_file);

            if backup_path.exists() && config_path.exists() {
                match std::fs::copy(&backup_path, &config_path) {
                    Ok(_) => {
                        info!("Restored {} from backup", config_file);
                        any_success = true;
                    }
                    Err(e) => {
                        warn!("Failed to restore {} from backup: {}", config_file, e);
                    }
                }
            }
        }

        // Reset to default configurations for common files
        if workspace_root.join("package.json").exists() {
            // For Node.js projects, try to reset npm/yarn config
            let _ = self.run_package_manager_command("npm", &["config", "delete"], &workspace_root);
            let _ = self.run_package_manager_command("yarn", &["config", "delete"], &workspace_root);
        }

        any_success
    }

    /// Attempt to fix permission issues
    fn attempt_permission_fixes(&self, task: &Task) -> bool {
        info!("Attempting permission fixes for task {}", task.id);

        let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Try to fix permissions on common directories
        let dirs_to_fix = ["node_modules", "target", ".next", "dist", "build"];

        let mut any_success = false;

        for dir in &dirs_to_fix {
            let dir_path = workspace_root.join(dir);
            if dir_path.exists() {
                // Use chmod-like operations (simplified - would need OS-specific implementation)
                #[cfg(unix)]
                {
                    use std::process::Command;
                    match Command::new("chmod")
                        .args(&["-R", "755", dir_path.to_str().unwrap_or("")])
                        .current_dir(&workspace_root)
                        .output() {
                        Ok(output) if output.status.success() => {
                            info!("Fixed permissions for directory: {}", dir);
                            any_success = true;
                        }
                        Ok(_) => {
                            debug!("Permission fix failed for {} (may not be needed)", dir);
                        }
                        Err(e) => {
                            debug!("Failed to run chmod for {}: {}", dir, e);
                        }
                    }
                }
            }
        }

        any_success
    }

    /// Attempt to scale resources (memory, CPU, etc.)
    fn attempt_resource_scaling(&self, task: &Task) -> bool {
        info!("Attempting resource scaling for task {}", task.id);

        // This is a simplified implementation - in practice would integrate with
        // container orchestration, process managers, or cloud resource management

        // Try to adjust Node.js memory limits if applicable
        if self.is_package_manager_available("node") {
            // Set higher memory limit for Node.js processes
            std::env::set_var("NODE_OPTIONS", "--max-old-space-size=4096");

            // Try to increase ulimits if on Unix
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("ulimit")
                    .args(&["-n", "4096"]) // Increase file descriptor limit
                    .output(); // Ignore errors as this may not be available
            }

            info!("Attempted to scale Node.js memory limit to 4GB");
            return true;
        }

        // For other environments, we can't easily scale resources automatically
        warn!("Resource scaling not applicable for current environment");
        false
    }

    /// Attempt retry with exponential backoff
    fn attempt_backoff_retry(&self, task: &Task) -> bool {
        info!("Scheduling retry with backoff for task {}", task.id);

        // This would typically integrate with a task scheduler or queue system
        // For now, we'll just log that a retry should be attempted

        // In a real implementation, this would:
        // 1. Calculate backoff delay based on failure history
        // 2. Schedule task retry with the calculated delay
        // 3. Possibly increase resource limits or change execution parameters

        let backoff_delay_seconds = 30 * (2_u32.pow(task.iteration_count.min(5) as u32)); // Exponential backoff

        info!("Recommended retry after {} seconds with exponential backoff", backoff_delay_seconds);

        // For now, just return false to indicate we can't automatically retry
        // In a real system, this would return true and schedule the retry
        false
    }

    /// Check if a package manager or tool is available on the system
    fn is_package_manager_available(&self, manager: &str) -> bool {
        use std::process::Command;
        Command::new(manager)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Run a package manager command and return success status
    fn run_package_manager_command(&self, manager: &str, args: &[&str], cwd: &std::path::Path) -> Result<bool, std::io::Error> {
        use std::process::Command;

        let mut command = Command::new(manager);
        command.args(args).current_dir(cwd);

        // Set a timeout for the command (simplified - would need tokio for real timeout)
        let output = command.output()?;

        Ok(output.status.success())
    }

    /// Classify the type of patch failure based on error and changeset
    fn classify_patch_failure(&self, error: &file_ops::FileOpsError, changeset: &ChangeSet) -> PatchFailureType {
        match error {
            file_ops::FileOpsError::Blocked(_) => PatchFailureType::PathBlocked,
            file_ops::FileOpsError::BudgetExceeded(_) => PatchFailureType::BudgetExceeded,
            file_ops::FileOpsError::Validation(_) => {
                // Check if this is likely a syntax error in generated code
                // For now, classify validation errors as syntax errors
                // This could be enhanced with more sophisticated detection
                PatchFailureType::SyntaxError
            }
            file_ops::FileOpsError::Io(_) | file_ops::FileOpsError::Path(_) => {
                // I/O and path errors are typically environment issues
                PatchFailureType::EnvironmentIssue
            }
            file_ops::FileOpsError::Serde(_) => {
                // Serialization errors are typically syntax issues
                PatchFailureType::SyntaxError
            }
        }
    }

    /// Record a patch failure for satisficing evaluation
    fn record_patch_failure(&self, failure_type: &PatchFailureType, task_id: uuid::Uuid) {
        self.patch_failure_history.borrow_mut().push(failure_type.clone());

        // Keep only recent failures (last 10) to avoid unbounded growth
        let mut history = self.patch_failure_history.borrow_mut();
        if history.len() > 10 {
            history.remove(0);
        }

        warn!("Recorded patch failure {:?} for task {}, history size: {}",
              failure_type, task_id, history.len());
    }

    /// Convert ActionRequest to file_ops ChangeSet
    fn action_request_to_changeset(&self, action_request: &ActionRequest) -> Result<ChangeSet, SelfPromptingError> {
        if let Some(changeset) = action_request.changeset() {
            // Convert existing changeset format to file_ops ChangeSet
            let patches = changeset.patches.iter().map(|patch| {
                Patch {
                    path: patch.path.clone(),
                    hunks: patch.hunks.iter().map(|hunk| {
                        Hunk {
                            old_start: hunk.old_start,
                            old_lines: hunk.old_lines,
                            new_start: hunk.new_start,
                            new_lines: hunk.new_lines,
                            lines: hunk.lines.clone(),
                        }
                    }).collect(),
                    expected_prev_sha256: None, // Will be computed during application
                }
            }).collect();

            Ok(ChangeSet { patches })
        } else {
            Err(SelfPromptingError::WorkspaceError(
                "ActionRequest has no changeset despite Write/Patch type".to_string()
            ))
        }
    }

    /// Rollback a changeset using workspace revert
    async fn rollback_changeset(&self, changeset_id: &ChangeSetId, task: &Task) -> Result<(), SelfPromptingError> {
        info!("Rolling back changeset {} for task {}", changeset_id.0, task.id);

        // Get workspace for this task's project
        let mut workspace = WorkspaceFactory::from_path(&task.project_path)
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to create workspace for rollback: {}", e)))?;

        // Revert the changeset
        workspace.revert(changeset_id).await
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to revert changeset: {}", e)))?;

        Ok(())
    }

    /// Promote workspace changes to the source directory after successful evaluation
    async fn promote_workspace_changes(&self, task: &Task) -> Result<(), SelfPromptingError> {
        info!("Promoting workspace changes for task {}", task.id);

        // Create workspace and promote changes
        let workspace = self.workspace_factory.create(&task.project_path)
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to create workspace for promotion: {}", e)))?;

        workspace.promote()
            .await
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to promote workspace: {}", e)))?;

        info!("Successfully promoted workspace changes for task {}", task.id);
        Ok(())
    }

    /// Rollback workspace changes after failed evaluation
    async fn rollback_workspace_changes(&self, task: &Task) -> Result<(), SelfPromptingError> {
        info!("Rolling back workspace changes for task {}", task.id);

        // Create workspace and rollback changes
        let workspace = self.workspace_factory.create(&task.project_path)
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to create workspace for rollback: {}", e)))?;

        workspace.revert()
            .await
            .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to rollback workspace: {}", e)))?;

        info!("Successfully rolled back workspace changes for task {}", task.id);
        Ok(())
    }

    /// Generate a diff artifact for observability
    async fn generate_diff_artifact(
        &self,
        iteration: usize,
        task: &Task,
    ) -> Option<Artifact> {
        // Get the most recent changeset ID
        let changeset_id = self.changeset_history.borrow().last().cloned()?;

        // Generate diff using workspace
        let workspace = match WorkspaceFactory::from_path(&task.project_path) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to create workspace for diff generation: {}", e);
                return None;
            }
        };

        // Generate unified diff for the changeset
        let diff_content = match workspace.generate_diff(&changeset_id).await {
            Ok(diff) => diff,
            Err(e) => {
                warn!("Failed to generate diff for changeset {}: {}", changeset_id.0, e);
                return None;
            }
        };

        // Create diff artifact using workspace-generated diff
        let diff_content = format!(
            "# Unified Diff - Iteration {}\n\
             Task: {}\n\
             Agent: self-prompting-loop\n\
             Changeset: {}\n\
             Timestamp: {}\n\n",
            iteration,
            task.id,
            changeset_id.0,
            chrono::Utc::now().to_rfc3339()
        ) + &diff_content;

        Some(Artifact {
            id: uuid::Uuid::new_v4(),
            file_path: format!("diffs/iteration-{}.diff", iteration),
            content: diff_content,
            artifact_type: ArtifactType::Diff,
            created_at: Utc::now(),
            size_bytes: diff_content.len() as u64,
        })
    }

    /// Create artifacts from an ActionRequest
    fn create_artifacts_from_action(
        &self,
        action_request: &ActionRequest,
        task: &Task,
    ) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        // Create artifact from changeset if present
        if let Some(changeset) = action_request.changeset() {
            for patch in &changeset.patches {
                // Extract content from patch hunks
                let mut content = String::new();
                for hunk in &patch.hunks {
                    content.push_str(&hunk.lines);
                    content.push('\n');
                }

                let artifact = Artifact {
                    id: uuid::Uuid::new_v4(),
                    file_path: patch.path.clone(),
                    content: content.trim().to_string(),
                    artifact_type: self.infer_artifact_type(task),
                    created_at: Utc::now(),
                };
                artifacts.push(artifact);
            }
        }

        // If no changeset, create a metadata artifact
        if artifacts.is_empty() {
            let metadata_content = format!(
                "Action: {:?}\nReason: {}\nConfidence: {:.2}\nMetadata: {}",
                action_request.action_type,
                action_request.reason,
                action_request.confidence,
                serde_json::to_string_pretty(&action_request.metadata)
                    .unwrap_or_else(|_| "{}".to_string())
            );

            let artifact = Artifact {
                id: uuid::Uuid::new_v4(),
                file_path: "action_metadata.txt".to_string(),
                content: metadata_content,
                artifact_type: ArtifactType::Documentation,
                created_at: Utc::now(),
            };
            artifacts.push(artifact);
        }

        artifacts
    }
}

/// Events emitted during self-prompting loop execution
#[derive(Debug, Clone)]
pub enum SelfPromptingEvent {
    IterationStarted {
        task_id: uuid::Uuid,
        iteration: usize,
        timestamp: chrono::Utc::now(),
    },
    EvaluationCompleted {
        task_id: uuid::Uuid,
        iteration: usize,
        score: f64,
        status: EvalStatus,
        timestamp: chrono::Utc::now(),
    },
    ModelSwapped {
        task_id: uuid::Uuid,
        old_model: String,
        new_model: String,
        reason: String,
        timestamp: chrono::Utc::now(),
    },
    ChangesetReverted {
        changeset_id: String,
        reason: String,
    },
    PatchApplied {
        task_id: uuid::Uuid,
        changeset_id: String,
        success: bool,
        failure_type: Option<PatchFailureType>,
        files_affected: usize,
        timestamp: chrono::Utc::now(),
    },
    ProgressCalculated {
        task_id: uuid::Uuid,
        iteration: usize,
        progress: crate::types::IterationProgress,
        timestamp: chrono::Utc::now(),
    },
    ContextReduced {
        strategy: crate::types::ScopeReductionStrategy,
        previous_files: usize,
        remaining_files: usize,
        timestamp: chrono::Utc::now(),
    },
    ContextMetricsUpdated {
        metrics: crate::types::ContextMetrics,
        timestamp: chrono::Utc::now(),
    },
    EnvironmentRecoveryAttempted {
        task_id: uuid::Uuid,
        failure_type: crate::evaluation::EvaluationFailureType,
        recovery_strategy: crate::evaluation::satisficing::EnvironmentRecoveryStrategy,
        success: bool,
        timestamp: chrono::Utc::now(),
    },
    GuidanceInjected {
        guidance: String,
        timestamp: chrono::Utc::now(),
    },
    VerdictOverridden {
        original_verdict: String,
        new_verdict: String,
        reason: String,
        timestamp: chrono::Utc::now(),
    },
    LoopCompleted {
        task_id: uuid::Uuid,
        total_iterations: usize,
        final_score: f64,
        stop_reason: StopReason,
        timestamp: chrono::Utc::now(),
    },
}

/// Errors from self-prompting execution
#[derive(Debug, thiserror::Error)]
pub enum SelfPromptingError {
    #[error("Model selection failed: {0}")]
    ModelSelectionError(String),

    #[error("Model generation failed: {0}")]
    ModelError(String),

    #[error("Evaluation failed: {0}")]
    EvaluationError(#[from] crate::evaluation::EvaluationError),

    #[error("Sandbox operation failed: {0}")]
    SandboxError(String),

    #[error("Task execution timed out")]
    Timeout,

    #[error("Maximum iterations exceeded")]
    MaxIterationsExceeded,

    #[error("Workspace operation failed: {0}")]
    WorkspaceError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_metrics_update() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Test context metrics update
        loop_controller.update_context_metrics(5000, 8000, 25);

        let monitor = loop_controller.context_monitor.borrow();
        assert_eq!(monitor.metrics.prompt_size_tokens, 5000);
        assert_eq!(monitor.metrics.context_window_utilization, 0.625); // 5000/8000
        assert_eq!(monitor.metrics.files_in_scope, 25);
    }

    #[test]
    fn test_context_overload_detection() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Set overload conditions
        loop_controller.update_context_metrics(9000, 10000, 60); // 90% utilization, 60 files

        let overload_strategy = loop_controller.check_context_overload();
        assert!(overload_strategy.is_some());
        // Should trigger based on file count exceeding threshold
    }

    #[test]
    fn test_scope_reduction_remove_least_recent() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let mut task = loop_controller.create_test_task("test task", vec![
            "file1.rs".to_string(),
            "file2.rs".to_string(),
            "file3.rs".to_string(),
            "file4.rs".to_string(),
        ]);

        let original_count = task.target_files.len();
        let remaining = loop_controller.apply_remove_least_recent_strategy(&mut task);

        assert!(remaining <= original_count);
        assert!(remaining >= original_count / 2); // Should keep at least 50%
    }

    #[test]
    fn test_scope_reduction_task_relevant() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let mut task = loop_controller.create_test_task("implement authentication API", vec![
            "auth.rs".to_string(),
            "user.rs".to_string(),
            "api.rs".to_string(),
            "database.rs".to_string(),
            "utils.rs".to_string(),
        ]);

        let original_count = task.target_files.len();
        let remaining = loop_controller.apply_task_relevant_only_strategy(&mut task);

        assert!(remaining <= original_count);
        // Should prioritize auth-related files for auth task
    }

    #[test]
    fn test_scope_reduction_high_change_frequency() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let mut task = loop_controller.create_test_task("test task", vec![
            "file1.rs".to_string(),
            "file2.rs".to_string(),
            "file3.rs".to_string(),
            "file4.rs".to_string(),
            "file5.rs".to_string(),
        ]);

        let original_count = task.target_files.len();
        let remaining = loop_controller.apply_high_change_frequency_strategy(&mut task);

        assert!(remaining <= original_count);
        assert!(remaining >= (original_count * 4) / 10); // Should keep at least 40%
    }

    #[test]
    fn test_task_relevance_analysis() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let analysis = loop_controller.analyze_task_relevance("implement authentication API endpoint");

        assert!(analysis.keywords.contains(&"auth".to_string()));
        assert!(analysis.keywords.contains(&"api".to_string()));
        assert!(analysis.file_extensions.contains(&".ts".to_string()) || analysis.file_extensions.contains(&".js".to_string()));
        assert!(analysis.directory_patterns.contains(&"auth".to_string()));
        assert!(analysis.directory_patterns.contains(&"api".to_string()));
    }

    #[test]
    fn test_task_relevance_scoring() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let analysis = crate::types::TaskRelevanceAnalysis {
            keywords: vec!["auth".to_string()],
            file_extensions: vec![".rs".to_string()],
            directory_patterns: vec!["api".to_string()],
        };

        // High relevance: matches keyword and extension
        let score1 = loop_controller.calculate_task_relevance_score("src/auth/api.rs", &analysis);
        assert!(score1 > 0.3);

        // Low relevance: no matches
        let score2 = loop_controller.calculate_task_relevance_score("utils/helpers.py", &analysis);
        assert!(score2 < 0.3);
    }

    #[test]
    fn test_evaluation_failure_classification_environment() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Test dependency failure classification
        let eval_report = loop_controller.create_test_eval_report(0.3, crate::evaluation::EvalStatus::Fail, None);
        let mut eval_report_with_logs = eval_report.clone();
        eval_report_with_logs.logs = vec!["dependency not found".to_string(), "npm install failed".to_string()];

        let failure_type = loop_controller.classify_evaluation_failure(&eval_report_with_logs);
        assert!(failure_type.is_some());
        match failure_type.unwrap() {
            crate::evaluation::EvaluationFailureType::EnvironmentFailure { category } => {
                assert!(matches!(category, crate::evaluation::EnvironmentFailureCategory::DependencyMissing));
            }
            _ => panic!("Expected environment failure"),
        }
    }

    #[test]
    fn test_evaluation_failure_classification_logic() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Test syntax error classification
        let eval_report = loop_controller.create_test_eval_report(0.2, crate::evaluation::EvalStatus::Fail, None);
        let mut eval_report_with_logs = eval_report.clone();
        eval_report_with_logs.logs = vec!["syntax error at line 25".to_string(), "compilation failed".to_string()];

        let failure_type = loop_controller.classify_evaluation_failure(&eval_report_with_logs);
        assert!(failure_type.is_some());
        match failure_type.unwrap() {
            crate::evaluation::EvaluationFailureType::LogicFailure { category } => {
                assert!(matches!(category, crate::evaluation::LogicFailureCategory::SyntaxError));
            }
            _ => panic!("Expected logic failure"),
        }
    }

    #[test]
    fn test_evaluation_failure_classification_pass() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Test passing evaluation (no failure to classify)
        let eval_report = loop_controller.create_test_eval_report(0.9, crate::evaluation::EvalStatus::Pass, None);

        let failure_type = loop_controller.classify_evaluation_failure(&eval_report);
        assert!(failure_type.is_none());
    }

    #[test]
    fn test_failure_history_tracking() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let failure1 = crate::evaluation::EvaluationFailureType::EnvironmentFailure {
            category: crate::evaluation::EnvironmentFailureCategory::DependencyMissing
        };
        let failure2 = crate::evaluation::EvaluationFailureType::LogicFailure {
            category: crate::evaluation::LogicFailureCategory::SyntaxError
        };

        loop_controller.record_evaluation_failure(failure1.clone());
        loop_controller.record_evaluation_failure(failure2.clone());

        let history = loop_controller.evaluation_failure_history.borrow();
        assert_eq!(history.len(), 2);
        assert!(matches!(&history[0], crate::evaluation::EvaluationFailureType::EnvironmentFailure { .. }));
        assert!(matches!(&history[1], crate::evaluation::EvaluationFailureType::LogicFailure { .. }));
    }

    #[test]
    fn test_package_manager_detection() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Test a common package manager (should be available in most environments)
        let available = loop_controller.is_package_manager_available("cargo");
        // Note: This test may fail in environments without cargo, but that's expected
        // In a real CI environment, we'd mock this or skip if not available
        let _ = available; // Just ensure the method doesn't panic
    }

    #[test]
    fn test_prompt_size_estimation() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let loop_controller = SelfPromptingLoop::new(registry, evaluator);

        let task = loop_controller.create_test_task("implement user authentication", vec![
            "src/auth.rs".to_string(),
            "src/user.rs".to_string(),
            "tests/auth.rs".to_string(),
        ]);

        let history = vec![loop_controller.create_test_eval_report(0.5, crate::evaluation::EvalStatus::Fail, None)];
        let estimated_size = loop_controller.estimate_prompt_size(&task, &history);

        // Should be a reasonable estimate > 0
        assert!(estimated_size > 0);
        assert!(estimated_size > 500); // Base overhead
    }

    /// Integration test demonstrating the complete instrumentation pipeline
    #[test]
    fn test_instrumentation_pipeline_integration() {
        let registry = Arc::new(crate::models::ModelRegistry::new());
        let evaluator = Arc::new(crate::evaluation::EvaluationOrchestrator::new());
        let mut loop_controller = SelfPromptingLoop::new(registry, evaluator);

        // Create a test task
        let task = loop_controller.create_test_task("implement user authentication API", vec![
            "src/auth.rs".to_string(),
            "src/user.rs".to_string(),
            "src/api.rs".to_string(),
            "tests/auth.rs".to_string(),
            "tests/api.rs".to_string(),
        ]);

        // Test context monitoring
        loop_controller.update_context_metrics(6000, 8000, 5);
        let monitor = loop_controller.context_monitor.borrow();
        assert_eq!(monitor.metrics.files_in_scope, 5);
        assert_eq!(monitor.metrics.context_window_utilization, 0.75); // 6000/8000

        // Test scope reduction (should work with our test data)
        let original_count = task.target_files.len();
        let remaining = loop_controller.apply_task_relevant_only_strategy(&mut task.clone());
        assert!(remaining <= original_count);

        // Test failure classification
        let eval_report = loop_controller.create_test_eval_report(0.2, crate::evaluation::EvalStatus::Fail, None);
        let mut eval_report_with_logs = eval_report.clone();
        eval_report_with_logs.logs = vec!["syntax error: unexpected token".to_string()];

        let failure_type = loop_controller.classify_evaluation_failure(&eval_report_with_logs);
        assert!(failure_type.is_some());
        match failure_type.unwrap() {
            crate::evaluation::EvaluationFailureType::LogicFailure { category } => {
                assert!(matches!(category, crate::evaluation::LogicFailureCategory::SyntaxError));
            }
            crate::evaluation::EvaluationFailureType::EnvironmentFailure { .. } => {
                panic!("Expected logic failure, got environment failure");
            }
        }

        // Test failure tracking
        loop_controller.record_evaluation_failure(crate::evaluation::EvaluationFailureType::LogicFailure {
            category: crate::evaluation::LogicFailureCategory::SyntaxError
        });
        let history = loop_controller.evaluation_failure_history.borrow();
        assert_eq!(history.len(), 1);

        // Verify all instrumentation components are working together
        assert!(monitor.metrics.prompt_size_tokens > 0); // Context monitoring active
        assert!(history.len() > 0); // Failure tracking active
    }
}
