//! Self-prompting loop controller that orchestrates generate-evaluate-refine cycles

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

use crate::evaluation::{EvaluationOrchestrator, EvalReport, EvalStatus};
use crate::models::{ModelRegistry, ModelProvider, ModelContext};
use crate::prompting::{PromptingStrategy, AdaptivePromptingStrategy};
use crate::sandbox::SandboxEnvironment;
use crate::types::{Task, TaskResult, IterationContext, StopReason, Artifact, ArtifactType};

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
    prompting_strategy: Box<dyn PromptingStrategy>,
    max_iterations: usize,
    event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
}

impl SelfPromptingLoop {
    /// Create a new self-prompting loop controller
    pub fn new(model_registry: Arc<ModelRegistry>, evaluator: Arc<EvaluationOrchestrator>) -> Self {
        Self {
            model_registry,
            evaluator,
            prompting_strategy: Box::new(AdaptivePromptingStrategy::new()),
            max_iterations: 5,
            event_sender: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
        max_iterations: usize,
        event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    ) -> Self {
        Self {
            model_registry,
            evaluator,
            prompting_strategy: Box::new(AdaptivePromptingStrategy::new()),
            max_iterations,
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

            // 2. Generate output using current context
            let output = self.generate_with_context(&*model, &task, &history).await?;
            info!("Iteration {}: Generated output ({} chars)", iteration, output.len());

            // 3. Create artifacts from output (for now, assume it's code/text)
            let artifact = Artifact {
                id: uuid::Uuid::new_v4(),
                file_path: task.target_files.first().cloned().unwrap_or_else(|| "generated.txt".to_string()),
                content: output.clone(),
                artifact_type: self.infer_artifact_type(&task),
                created_at: chrono::Utc::now(),
            };
            artifacts.push(artifact.clone());

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

            // 5. Check satisficing - should we continue?
            let satisficing_decision = self.evaluator.satisficing_decision(&eval_report, &history);

            if !satisficing_decision.should_continue {
                info!("Iteration {}: Stopping - {}", iteration, match satisficing_decision.reason {
                    StopReason::Satisficed => "Satisficed",
                    StopReason::MaxIterations => "Max iterations reached",
                    StopReason::QualityCeiling => "Quality ceiling reached",
                    StopReason::FailedGates => "Failed mandatory gates",
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
        // For now, assume the output is stored in artifacts
        // In a real implementation, we'd store the raw output separately
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
        // For now, just execute normally - sandbox integration would be added here
        // This would involve:
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
