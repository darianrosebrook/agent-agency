//! Self-prompting loop controller
//!
//! Orchestrates the generate → evaluate → refine cycle for autonomous task execution.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::self_prompting_agent::evaluation::EvaluationOrchestrator;
use crate::self_prompting_agent::models::ModelRegistry;
use crate::self_prompting_agent::prompting_types::{Task, TaskResult, SelfPromptingAgentError};

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

            // Generate prompt with iteration context
            let prompt = self.generate_prompt(&current_task, iteration).await?;
            let event = SelfPromptingEvent::PromptGenerated {
                iteration,
                prompt: prompt.clone(),
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);

            // Execute task iteration using model registry
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
            let original_context_len = current_task.refinement_context.len();
            current_task = self.refine_task(&current_task, &evaluation).await?;
            let changes = current_task.refinement_context.len() - original_context_len;
            
            let event = SelfPromptingEvent::RefinementApplied {
                iteration,
                changes: changes.max(1), // Track actual changes made during refinement
            };
            events.push(event.clone());
            let _ = self.event_sender.send(event);
        }

        Err("Maximum iterations reached without satisfactory result".into())
    }

    /// Generate prompt for the current iteration
    async fn generate_prompt(&self, task: &Task, iteration: usize) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::self_prompting_agent::prompting::AdaptivePromptingStrategy;
        
        // Build comprehensive prompt with iteration context and refinement history
        let mut prompt_parts = Vec::new();
        
        // Task description
        prompt_parts.push(format!("Task: {}", task.description));
        
        // Add task type context
        prompt_parts.push(format!("Task Type: {:?}", task.task_type));
        
        // Add target files if specified
        if !task.target_files.is_empty() {
            prompt_parts.push(format!("Target Files: {}", task.target_files.join(", ")));
        }
        
        // Add constraints if any
        if !task.constraints.is_empty() {
            let constraint_lines: Vec<String> = task.constraints.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            prompt_parts.push(format!("Constraints:\n{}", constraint_lines.join("\n")));
        }
        
        // Add refinement context from previous iterations
        if !task.refinement_context.is_empty() {
            prompt_parts.push("Previous Iteration Feedback:".to_string());
            for (idx, context) in task.refinement_context.iter().enumerate() {
                prompt_parts.push(format!("  Iteration {}: {}", idx + 1, context));
            }
        }
        
        // Add iteration-specific instructions
        prompt_parts.push(format!(
            "\nIteration {} of self-prompting loop. Please provide a high-quality solution that addresses all requirements.",
            iteration
        ));
        
        let base_prompt = prompt_parts.join("\n\n");
        
        // Optimize prompt for task type using adaptive prompting strategy
        let strategy = AdaptivePromptingStrategy::new();
        let task_type_str = match task.task_type {
            crate::self_prompting_agent::prompting_types::TaskType::CodeGeneration => "coding",
            crate::self_prompting_agent::prompting_types::TaskType::CodeReview => "analysis",
            crate::self_prompting_agent::prompting_types::TaskType::CodeRefactor => "coding",
            crate::self_prompting_agent::prompting_types::TaskType::Testing => "coding",
            crate::self_prompting_agent::prompting_types::TaskType::Documentation => "analysis",
            crate::self_prompting_agent::prompting_types::TaskType::Research => "analysis",
            crate::self_prompting_agent::prompting_types::TaskType::Planning => "planning",
        };
        
        let optimized_prompt = strategy.optimize_for_task(&base_prompt, task_type_str);
        
        Ok(optimized_prompt)
    }

    /// Execute a single iteration
    async fn execute_single_iteration(
        &self,
        task: &Task,
        prompt: &str,
        model_registry: &Arc<ModelRegistry>,
    ) -> Result<TaskResult, Box<dyn std::error::Error + Send + Sync>> {
        use crate::self_prompting_agent::prompting_types::{EvalReport, EvalStatus, Artifact, ArtifactType};
        use crate::self_prompting_agent::models::GenerationOptions;
        use std::time::Instant;
        
        let start_time = Instant::now();
        
        // Generate response using model registry
        let generation_options = GenerationOptions {
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: Some(0.9),
            stop_sequences: vec![],
            model_name: None, // Use default model
        };
        
        let generated_content = match model_registry.generate(prompt, &generation_options).await {
            Ok(content) => content,
            Err(e) => {
                // If generation fails, return error result
                return Ok(TaskResult {
                    task_id: task.id,
                    task_type: task.task_type.clone(),
                    final_report: EvalReport {
                        score: 0.0,
                        status: EvalStatus::Fail,
                        thresholds_met: vec![],
                        failed_criteria: vec![format!("Model generation failed: {}", e)],
                    },
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    artifacts: vec![],
                });
            }
        };
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Create artifacts from generated content
        let artifact_type = match task.task_type {
            crate::self_prompting_agent::prompting_types::TaskType::CodeGeneration => ArtifactType::Code,
            crate::self_prompting_agent::prompting_types::TaskType::CodeReview => ArtifactType::Documentation,
            crate::self_prompting_agent::prompting_types::TaskType::Documentation => ArtifactType::Documentation,
            _ => ArtifactType::Text,
        };
        
        let artifact = Artifact {
            id: uuid::Uuid::new_v4(),
            file_path: if !task.target_files.is_empty() {
                task.target_files[0].clone()
            } else {
                format!("generated_{}.txt", task.id)
            },
            content: generated_content.clone(),
            artifact_type,
            created_at: chrono::Utc::now(),
        };
        
        // Initial score based on content quality (will be refined by evaluator)
        let initial_score = if generated_content.len() > 100 {
            0.7 // Good starting score for substantial content
        } else {
            0.4 // Lower score for minimal content
        };
        
        let result = TaskResult {
            task_id: task.id,
            task_type: task.task_type.clone(),
            final_report: EvalReport {
                score: initial_score,
                status: EvalStatus::Partial, // Will be refined by evaluator
                thresholds_met: vec!["Content generated".to_string()],
                failed_criteria: vec![],
            },
            execution_time_ms,
            artifacts: vec![artifact],
        };

        Ok(result)
    }

    /// Refine task based on evaluation feedback
    async fn refine_task(
        &self,
        task: &Task,
        evaluation: &crate::self_prompting_agent::evaluation::EvaluationResult,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let mut refined_task = task.clone();

        // Build comprehensive feedback context
        let mut feedback_parts = Vec::new();
        
        // Score feedback
        feedback_parts.push(format!("Score: {:.2}/1.0", evaluation.score));
        
        // Status feedback
        feedback_parts.push(format!("Status: {:?}", evaluation.status));
        
        // Issues to address
        if !evaluation.issues.is_empty() {
            feedback_parts.push("Issues to address:".to_string());
            for issue in &evaluation.issues {
                feedback_parts.push(format!("  - {}", issue));
            }
        }
        
        // Recommendations for improvement
        if !evaluation.recommendations.is_empty() {
            feedback_parts.push("Recommendations:".to_string());
            for rec in &evaluation.recommendations {
                feedback_parts.push(format!("  - {}", rec));
            }
        }
        
        // Add refined constraints based on issues
        if evaluation.score < 0.7 {
            refined_task.constraints.insert(
                "strict_validation".to_string(),
                "true".to_string()
            );
        }
        
        // Add feedback to refinement context
        refined_task.refinement_context.push(feedback_parts.join("\n"));

        Ok(refined_task)
    }

    /// Shutdown the loop controller
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Self-prompting loop controller shutdown");
        Ok(())
    }
}
