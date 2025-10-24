//! Self-prompting loop controller
//!
//! Orchestrates the generate → evaluate → refine cycle for autonomous task execution.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::evaluation::EvaluationOrchestrator;
use crate::models::ModelRegistry;
use crate::types::{Task, TaskResult, SelfPromptingAgentError};

/// Self-prompting loop controller
pub struct SelfPromptingLoop {
    max_iterations: usize,
    event_sender: mpsc::UnboundedSender<SelfPromptingEvent>,
}

#[derive(Debug, Clone)]
pub enum SelfPromptingEvent {
    IterationStarted { iteration: usize, task_id: String },
    PromptGenerated { iteration: usize, prompt: String },
    EvaluationCompleted { iteration: usize, score: f64 },
    RefinementApplied { iteration: usize, changes: usize },
    LoopCompleted { iterations: usize, final_score: f64 },
    Error { iteration: usize, error: String },
}

#[derive(Debug)]
pub struct SelfPromptingResult {
    pub task: Task,
    pub result: TaskResult,
    pub iterations: usize,
    pub events: Vec<SelfPromptingEvent>,
}

impl SelfPromptingLoop {
    /// Create a new self-prompting loop controller
    pub async fn new(
        max_iterations: usize,
        event_sender: mpsc::UnboundedSender<SelfPromptingEvent>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            max_iterations,
            event_sender,
        })
    }

    /// Execute a task using the self-prompting loop
    pub async fn execute_task(
        &self,
        task: Task,
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
    ) -> Result<SelfPromptingResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut events = Vec::new();
        let mut current_task = task.clone();
        let mut best_result: Option<TaskResult> = None;
        let mut best_score = 0.0;

        for iteration in 1..=self.max_iterations {
            // Emit iteration started event
            let event = SelfPromptingEvent::IterationStarted {
                iteration,
                task_id: current_task.id.to_string(),
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);

            // Generate prompt (stub implementation)
            let prompt = self.generate_prompt(&current_task, iteration).await?;
            let event = SelfPromptingEvent::PromptGenerated {
                iteration,
                prompt: prompt.clone(),
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);

            // Execute task (stub implementation)
            let result = self.execute_single_iteration(&current_task, &prompt, &model_registry).await?;
            let score = result.final_report.score;

            // Evaluate result
            let evaluation = evaluator.evaluate_result(&result).await
                .map_err(|e| format!("Evaluation failed: {}", e))?;

            let event = SelfPromptingEvent::EvaluationCompleted {
                iteration,
                score: evaluation.score,
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);

            // Check if this is the best result so far
            if score > best_score {
                best_score = score;
                best_result = Some(result.clone());
            }

            // Check if we should continue iterating
            if evaluation.score >= 0.9 || iteration == self.max_iterations {
                // Final result
                let final_result = best_result.unwrap_or(result);
                let event = SelfPromptingEvent::LoopCompleted {
                    iterations: iteration,
                    final_score: final_result.final_report.score,
                };
                events.push(event.clone());
                let _ = self.event_sender.send(event);

                return Ok(SelfPromptingResult {
                    task: current_task,
                    result: final_result,
                    iterations: iteration,
                    events,
                });
            }

            // Refine task for next iteration
            current_task = self.refine_task(&current_task, &evaluation).await?;
            let event = SelfPromptingEvent::RefinementApplied {
                iteration,
                changes: 1, // Stub: track actual changes
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);
        }

        Err("Maximum iterations reached without satisfactory result".into())
    }

    /// Generate prompt for the current iteration
    async fn generate_prompt(&self, task: &Task, iteration: usize) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - in real system this would use prompting strategies
        Ok(format!(
            "Iteration {}: Please complete the following task: {}\n\nContext: {:?}\n\nProvide a high-quality solution.",
            iteration,
            task.description,
            task.refinement_context
        ))
    }

    /// Execute a single iteration
    async fn execute_single_iteration(
        &self,
        task: &Task,
        prompt: &str,
        model_registry: &Arc<ModelRegistry>,
    ) -> Result<TaskResult, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - would use model registry to execute task
        use crate::types::{EvalReport, EvalStatus, Artifact, ArtifactType};

        let result = TaskResult {
            task_id: task.id,
            task_type: task.task_type.clone(),
            final_report: EvalReport {
                score: 0.8, // Mock score
                status: EvalStatus::Pass,
                thresholds_met: vec!["Basic requirements met".to_string()],
                failed_criteria: vec![],
            },
            execution_time_ms: 1000,
            artifacts: vec![
                Artifact {
                    id: uuid::Uuid::new_v4(),
                    file_path: "generated_output.txt".to_string(),
                    content: format!("Generated content for: {}", task.description),
                    artifact_type: ArtifactType::Code,
                    created_at: chrono::Utc::now(),
                }
            ],
        };

        Ok(result)
    }

    /// Refine task based on evaluation feedback
    async fn refine_task(
        &self,
        task: &Task,
        evaluation: &crate::evaluation::EvaluationResult,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        // Stub implementation - would analyze evaluation and refine task
        let mut refined_task = task.clone();

        // Add evaluation feedback to context
        refined_task.refinement_context.push(format!(
            "Iteration feedback: Score {:.2}, Issues: {:?}",
            evaluation.score,
            evaluation.issues
        ));

        Ok(refined_task)
    }

    /// Shutdown the loop controller
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Self-prompting loop controller shutdown");
        Ok(())
    }
}
