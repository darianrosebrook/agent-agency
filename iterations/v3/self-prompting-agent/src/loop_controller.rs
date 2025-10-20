//! Self-prompting loop controller that orchestrates generate-evaluate-refine cycles

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

use crate::evaluation::{EvaluationOrchestrator, EvalReport, EvalStatus, SatisficingEvaluator, SatisficingDecision};
use crate::models::{ModelRegistry, ModelProvider, ModelContext};
use crate::prompting::{PromptingStrategy, AdaptivePromptingStrategy};
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, TaskResult, IterationContext, StopReason, Artifact, ArtifactType, ActionRequest, ActionValidationError};
use observability::diff_observability::{DiffGenerator, FileChange};
use observability::agent_telemetry::AgentTelemetryCollector;
use file_ops::{WorkspaceFactory, Workspace, ChangeSet, Patch, Hunk};

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

                // Generate diff artifact for observability
                if let Some(diff_artifact) = self.generate_diff_artifact(&action_request, iteration, &task).await {
                    artifacts.push(diff_artifact);
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

            // 5. Check satisficing with hysteresis and no-progress guards
            let mut satisficing_evaluator = self.satisficing_evaluator.borrow_mut();
            let satisficing_decision = satisficing_evaluator.should_continue(&eval_report, &history);

            // Additional no-progress checks
            if satisficing_decision.should_continue {
                // Check for no progress based on recent action (if available)
                // Note: In a full implementation, we'd track the changeset from the action
                // For now, we'll rely on the hysteresis logic
            }

            if !satisficing_decision.should_continue {
                info!("Iteration {}: Stopping - {}", iteration, match satisficing_decision.reason {
                    StopReason::Satisficed => "Satisficed",
                    StopReason::MaxIterations => "Max iterations reached",
                    StopReason::QualityCeiling => "Quality ceiling reached",
                    StopReason::FailedGates => "Failed mandatory gates",
                    StopReason::NoProgress => "No progress detected",
                    _ => "Unknown reason",
                });

                // Emit final result event
                self.emit_event(SelfPromptingEvent::LoopCompleted {
                    task_id: task.id,
                    total_iterations: iteration,
                    final_score: eval_report.score,
                    stop_reason: satisficing_decision.reason.clone(),
                    timestamp: chrono::Utc::now(),
                });

                let total_time = start_time.elapsed().as_millis() as u64;

                return Ok(SelfPromptingResult {
                    task_result: TaskResult {
                        task_id: task.id,
                        final_report: eval_report,
                        iterations: iteration,
                        stop_reason: Some(satisficing_decision.reason),
                        model_used: model_id,
                        total_time_ms: total_time,
                        artifacts,
                    },
                    iterations_performed: iteration,
                    models_used,
                    total_time_ms: total_time,
                    final_stop_reason: satisficing_decision.reason,
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

                    // Create re-prompt with error context
                    // For now, we'll log and continue - in production, you'd modify the prompt
                    // to include the error and request correction
                    continue;
                }
            }
        }
    }

    /// Apply an ActionRequest to the workspace
    async fn apply_action_request(
        &self,
        action_request: &ActionRequest,
        task: &Task,
    ) -> Result<(), SelfPromptingError> {
        match action_request.action_type {
            crate::types::ActionType::Write | crate::types::ActionType::Patch => {
                if let Some(changeset) = action_request.changeset() {
                    info!("Applying changeset with {} patches to workspace at {}",
                          changeset.patches.len(), task.project_path);

                    // Create an isolated workspace for this task
                    let workspace = self.workspace_factory.create(&task.project_path)
                        .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to create workspace: {}", e)))?;

                    // Apply the changeset to the workspace
                    let changeset_id = workspace.apply_changeset(changeset.clone())
                        .await
                        .map_err(|e| SelfPromptingError::WorkspaceError(format!("Failed to apply changeset: {}", e)))?;

                    info!("Successfully applied changeset {} with {} patches",
                          changeset_id, changeset.patches.len());

                    // Note: We don't promote here - that happens after evaluation passes
                    // The workspace remains in sandbox until evaluation succeeds
                } else {
                    warn!("ActionRequest has no changeset despite Write/Patch type");
                }
            }
            crate::types::ActionType::NoOp => {
                info!("Action request is NoOp: {}", action_request.reason);
            }
        }

        Ok(())
    }

    /// Generate a diff artifact for observability
    async fn generate_diff_artifact(
        &self,
        action_request: &ActionRequest,
        iteration: usize,
        task: &Task,
    ) -> Option<Artifact> {
        if action_request.changeset.is_none() {
            return None;
        }

        // Create a simplified diff representation
        // In a full implementation, this would compare actual file states
        let mut diff_content = format!(
            "# Unified Diff - Iteration {}\n",
            iteration
        );
        diff_content.push_str(&format!("Task: {}\n", task.id));
        diff_content.push_str(&format!("Agent: self-prompting-loop\n"));
        diff_content.push_str(&format!("Timestamp: {}\n\n", chrono::Utc::now().to_rfc3339()));

        if let Some(changeset) = &action_request.changeset {
            for patch in &changeset.patches {
                diff_content.push_str(&format!(
                    "diff --git a/{} b/{}\n",
                    patch.path, patch.path
                ));

                // Simplified hunk representation
                let old_lines = patch.hunks.iter().map(|h| h.old_lines).sum::<u32>();
                let new_lines = patch.hunks.iter().map(|h| h.new_lines).sum::<u32>();

                diff_content.push_str(&format!(
                    "@@ -1,{} +1,{} @@\n",
                    old_lines, new_lines
                ));

                for hunk in &patch.hunks {
                    for line in &hunk.lines {
                        diff_content.push_str(line);
                        diff_content.push('\n');
                    }
                }
                diff_content.push('\n');
            }
        }

        Some(Artifact {
            id: uuid::Uuid::new_v4(),
            file_path: format!("diffs/iteration-{}.diff", iteration),
            content: diff_content,
            artifact_type: ArtifactType::Diff,
            created_at: chrono::Utc::now(),
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
}
