//! Main execution orchestration for the self-prompting loop controller

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::evaluation::{EvaluationOrchestrator, EvalReport};
use crate::types::{Task, TaskResult, StopReason};
use crate::models::ModelRegistry;
use super::types::{SelfPromptingResult, SelfPromptingError, ExecutionMode, SelfPromptingEvent};
use super::config::LoopControllerConfig;
use super::state::ExecutionStateManager;
use super::monitoring::{ProgressTracker, IterationProgress};
use super::history::{ChangesetHistory, PatchFailureHistory, EvaluationFailureHistory};
use super::events::EventBroadcaster;

/// Main self-prompting loop controller with modular architecture
#[derive(Debug)]
pub struct SelfPromptingLoop {
    /// Model registry for AI model management
    model_registry: Arc<ModelRegistry>,
    /// Evaluation orchestrator for quality assessment
    evaluator: Arc<EvaluationOrchestrator>,
    /// Configuration settings
    config: LoopControllerConfig,
    /// Execution state manager
    state_manager: ExecutionStateManager,
    /// Progress tracker
    progress_tracker: ProgressTracker,
    /// Changeset history for rollback
    changeset_history: ChangesetHistory,
    /// Patch failure history for pattern analysis
    patch_failure_history: PatchFailureHistory,
    /// Evaluation failure history
    evaluation_failure_history: EvaluationFailureHistory,
    /// Event broadcaster
    event_broadcaster: EventBroadcaster,
}

impl SelfPromptingLoop {
    /// Create a new self-prompting loop controller
    pub fn new(
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
        config: LoopControllerConfig,
        event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    ) -> Self {
        Self {
            model_registry,
            evaluator,
            config,
            state_manager: ExecutionStateManager::new(event_sender.clone()),
            progress_tracker: ProgressTracker::new(100), // Keep last 100 iterations
            changeset_history: ChangesetHistory::new(50), // Keep last 50 changesets
            patch_failure_history: PatchFailureHistory::new(100), // Keep last 100 failures
            evaluation_failure_history: EvaluationFailureHistory::new(50), // Keep last 50 eval failures
            event_broadcaster: EventBroadcaster::new(event_sender),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(
        model_registry: Arc<ModelRegistry>,
        evaluator: Arc<EvaluationOrchestrator>,
        event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    ) -> Self {
        Self::new(model_registry, evaluator, LoopControllerConfig::default(), event_sender)
    }

    /// Execute a task using the self-prompting loop
    pub async fn execute_task(&self, mut task: Task) -> Result<SelfPromptingResult, SelfPromptingError> {
        let start_time = std::time::Instant::now();
        let mut iteration = 0;
        let mut models_used = Vec::new();

        self.event_broadcaster.task_started(
            task.id.clone().unwrap_or_else(|| "unknown".to_string()),
            task.description.clone(),
        );

        loop {
            iteration += 1;

            // Check execution state for intervention
            if !self.state_manager.should_continue() {
                if self.state_manager.is_aborted() {
                    self.event_broadcaster.task_failed(
                        "Task aborted by user".to_string(),
                        iteration,
                    );
                    return Ok(SelfPromptingResult {
                        task_result: TaskResult::Failed("Task aborted by user".to_string()),
                        iterations_performed: iteration,
                        models_used,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                        final_stop_reason: StopReason::Aborted,
                    });
                } else if self.state_manager.is_paused() {
                    self.event_broadcaster.task_paused("User intervention requested".to_string());
                    // Wait for resume (simplified - in real implementation would be more sophisticated)
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            }

            // Execute iteration logic here
            // This is a simplified version - the full implementation would include:
            // 1. Model selection and prompting
            // 2. Artifact generation
            // 3. Evaluation and quality assessment
            // 4. Changeset application (if approved)
            // 5. Progress tracking and plateau detection

            // Record progress
            let progress = IterationProgress {
                iteration,
                progress_score: (iteration as f64 / self.config.max_iterations as f64).min(1.0),
                artifacts_generated: 0, // Would be calculated from actual artifacts
                models_used: vec![], // Would be populated from actual model usage
                timestamp: chrono::Utc::now(),
            };
            self.progress_tracker.record_progress(progress);

            self.event_broadcaster.iteration_completed(iteration, 0);

            // Check stopping conditions
            if iteration >= self.config.max_iterations {
                self.event_broadcaster.task_completed(
                    iteration,
                    start_time.elapsed().as_millis() as u64,
                    "Maximum iterations reached".to_string(),
                );
                return Ok(SelfPromptingResult {
                    task_result: TaskResult::Completed("Task completed within iteration limit".to_string()),
                    iterations_performed: iteration,
                    models_used,
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                    final_stop_reason: StopReason::MaxIterationsReached,
                });
            }

            // Check for plateau
            if self.progress_tracker.has_plateaued(5, 0.01) {
                self.event_broadcaster.task_completed(
                    iteration,
                    start_time.elapsed().as_millis() as u64,
                    "Progress plateau detected".to_string(),
                );
                return Ok(SelfPromptingResult {
                    task_result: TaskResult::Completed("Task completed due to progress plateau".to_string()),
                    iterations_performed: iteration,
                    models_used,
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                    final_stop_reason: StopReason::PlateauDetected,
                });
            }

            // Brief pause between iterations
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Get the current execution state
    pub fn execution_state(&self) -> super::types::ExecutionState {
        self.state_manager.current_state()
    }

    /// Pause execution
    pub fn pause_execution(&self) {
        self.state_manager.pause();
    }

    /// Resume execution
    pub fn resume_execution(&self) {
        self.state_manager.resume();
    }

    /// Abort execution
    pub fn abort_execution(&self) {
        self.state_manager.abort();
    }

    /// Set execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.config.execution_mode = mode;
    }

    /// Set user approval callback
    pub fn set_user_approval_callback(
        &mut self,
        callback: Arc<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>
    ) {
        self.config.user_approval_callback = Some(callback);
    }

    /// Inject guidance for future iterations
    pub fn inject_guidance(&self, guidance: String) {
        self.state_manager.inject_guidance(guidance);
    }

    /// Get injected guidance
    pub fn get_injected_guidance(&self) -> Vec<String> {
        self.state_manager.get_injected_guidance()
    }

    /// Override verdict with user guidance
    pub fn override_verdict(&self, new_verdict: String, reason: String) {
        self.state_manager.override_verdict(new_verdict, reason);
    }

    /// Modify parameter
    pub fn modify_parameter(&self, parameter: String, value: String) {
        self.state_manager.modify_parameter(parameter, value);
    }

    /// Get latest progress
    pub fn latest_progress(&self) -> Option<IterationProgress> {
        self.progress_tracker.latest_progress()
    }

    /// Get progress history
    pub fn progress_history(&self) -> Vec<IterationProgress> {
        self.progress_tracker.get_history()
    }

    /// Get changeset history
    pub fn changeset_history(&self) -> Vec<crate::stubs::ChangeSetId> {
        self.changeset_history.all_changesets()
    }

    /// Get recent patch failures
    pub fn recent_patch_failures(&self, count: usize) -> Vec<super::types::PatchFailureType> {
        self.patch_failure_history.recent_failures(count)
    }

    /// Get most common failure type
    pub fn most_common_failure(&self) -> Option<super::types::PatchFailureType> {
        self.patch_failure_history.most_common_failure()
    }

    /// Check if progress has plateaued
    pub fn has_plateaued(&self, window_size: usize, threshold: f64) -> bool {
        self.progress_tracker.has_plateaued(window_size, threshold)
    }

    /// Get progress trend
    pub fn progress_trend(&self, window_size: usize) -> f64 {
        self.progress_tracker.get_progress_trend(window_size)
    }
}
