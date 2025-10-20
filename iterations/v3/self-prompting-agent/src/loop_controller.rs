//! Self-prompting loop controller that orchestrates generate-evaluate-refine cycles

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};

use crate::evaluation::{EvaluationOrchestrator, EvalReport, EvalStatus, SatisficingEvaluator, SatisficingDecision};
use crate::models::{ModelRegistry, ModelProvider, ModelContext};
use crate::prompting::{PromptingStrategy, AdaptivePromptingStrategy};
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, TaskResult, IterationContext, StopReason, Artifact, ArtifactType, ActionRequest, ActionValidationError};
use observability::diff_observability::{DiffGenerator, FileChange};
use observability::agent_telemetry::AgentTelemetryCollector;

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
    max_iterations: usize,
    execution_mode: ExecutionMode,
    event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
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
            max_iterations: 5,
            execution_mode: ExecutionMode::Auto, // Default to auto mode
            event_sender: None,
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
            max_iterations,
            execution_mode,
            event_sender: None,
        }
    }

    /// Set the execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
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

            // Emit iteration start event
            self.emit_event(SelfPromptingEvent::IterationStarted {
                task_id: task.id,
                iteration,
                timestamp: chrono::Utc::now(),
            });

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

            // 3. Apply the action if it requires changes (mode-dependent)
            if action_request.requires_changes() {
                match self.execution_mode {
                    ExecutionMode::DryRun => {
                        info!("Dry-run mode: Skipping changeset application");
                        // Still generate artifacts but don't apply changes
                    }
                    ExecutionMode::Strict => {
                        info!("Strict mode: Requesting user approval for changeset");
                        // TODO: Implement user approval prompt
                        // For now, skip application in strict mode
                        warn!("Strict mode not yet implemented - skipping changeset application");
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

            let eval_report = self.evaluator.evaluate(&[artifact], &eval_context).await?;
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

            // Combine all satisficing decisions - stop if any indicates we should
            let final_decision = if let Some(patch_decision) = patch_failure_decision {
                if !patch_decision.should_continue {
                    info!("Patch failure pattern detected, stopping iteration: {}", patch_decision.reason);
                    patch_decision
                } else if let Some(plateau_decision) = plateau_decision {
                    if !plateau_decision.should_continue {
                        info!("Progress plateau detected, stopping iteration: {}", plateau_decision.reason);
                        plateau_decision
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
            created_at: chrono::Utc::now(),
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
                    created_at: chrono::Utc::now(),
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
                created_at: chrono::Utc::now(),
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
