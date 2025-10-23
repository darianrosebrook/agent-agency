//! Parallel coordinator - main orchestrator for parallel task execution

use crate::types::*;
use crate::error::*;
use crate::decomposition::{DecompositionEngine, TaskAnalysis};
use crate::worker::{WorkerManager, DefaultWorkerPool};
use crate::progress::{ProgressAggregator, ProgressSynthesizer};
use crate::validation::{ValidationRunner, ValidationContext};
use crate::orchestrator_bridge::{OrchestrationQualityBridge, StubOrchestrationQualityHandle, ExecutionArtifacts};
use crate::monitoring_bridge::{OrchestrationMonitoringBridge, StubOrchestrationMonitoringHandle, ExecutionStatus};
use crate::communication::hub::CommunicationHub;
use std::sync::Arc;

// Stub implementations for missing workspace dependencies
// TODO: Replace with actual workspace crate integrations

/// Stub implementation for orchestration handle
#[async_trait::async_trait]
pub trait OrchestratorHandle: Send + Sync {
    async fn execute_sequential(&self, task: ComplexTask) -> ParallelResult<TaskResult>;
}

/// Stub implementation for orchestration handle
pub struct StubOrchestratorHandle;

#[async_trait::async_trait]
impl OrchestratorHandle for StubOrchestratorHandle {
    async fn execute_sequential(&self, _task: ComplexTask) -> ParallelResult<TaskResult> {
        // Stub implementation - always return a basic success result
        Ok(TaskResult {
            task_id: TaskId::new(),
            success: true,
            subtasks_completed: 1,
            total_subtasks: 1,
            execution_time: std::time::Duration::from_secs(1),
            summary: "Sequential execution completed (stub)".to_string(),
            worker_breakdown: vec![],
            quality_scores: std::collections::HashMap::new(),
        })
    }
}

/// Main coordinator for parallel task execution
pub struct ParallelCoordinator {
    decomposition_engine: DecompositionEngine,
    worker_manager: WorkerManager,
    progress_aggregator: ProgressAggregator,
    progress_synthesizer: ProgressSynthesizer,
    validation_runner: ValidationRunner,
    communication_hub: CommunicationHub,
    config: ParallelCoordinatorConfig,
    orchestrator_handle: Option<Arc<dyn OrchestratorHandle>>, // Integration point
    quality_bridge: OrchestrationQualityBridge,
    monitoring_bridge: OrchestrationMonitoringBridge,
}

#[derive(Debug, Clone)]
pub struct ParallelCoordinatorConfig {
    pub enabled: bool,
    pub max_concurrent_workers: usize,
    pub max_subtasks_per_task: usize,
    pub task_timeout_seconds: u64,
    pub complexity_threshold: f32,
    pub enable_quality_gates: bool,
    pub enable_dependency_resolution: bool,
}

impl Default for ParallelCoordinatorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_workers: 8,
            max_subtasks_per_task: 20,
            task_timeout_seconds: 300,
            complexity_threshold: 0.6,
            enable_quality_gates: true,
            enable_dependency_resolution: true,
        }
    }
}

impl ParallelCoordinator {
    /// Create a new parallel coordinator
    pub fn new(config: ParallelCoordinatorConfig) -> Self {
        let worker_pool = Arc::new(DefaultWorkerPool::new());
        let communication_hub = CommunicationHub::new(Default::default());
        let quality_bridge = OrchestrationQualityBridge::new(Arc::new(StubOrchestrationQualityHandle));
        let monitoring_bridge = OrchestrationMonitoringBridge::new(Arc::new(StubOrchestrationMonitoringHandle));

        Self {
            decomposition_engine: DecompositionEngine::new(),
            worker_manager: WorkerManager::new(worker_pool),
            progress_aggregator: ProgressAggregator::new(TaskId::new()),
            progress_synthesizer: ProgressSynthesizer::new(),
            validation_runner: ValidationRunner::new(4), // Run 4 validations in parallel
            communication_hub,
            config,
            orchestrator_handle: None,
            quality_bridge,
            monitoring_bridge,
        }
    }

    /// Set the orchestrator handle for fallback sequential execution
    pub fn with_orchestrator_handle(mut self, handle: Arc<dyn OrchestratorHandle>) -> Self {
        self.orchestrator_handle = Some(handle);
        self
    }

    /// Main entry point for parallel execution
    pub async fn execute_parallel(&mut self, task: ComplexTask) -> ParallelResult<TaskResult> {
        // 1. Analyze task complexity and determine if parallel execution is beneficial
        let analysis = self.analyze_task(&task).await?;

        // Publish analysis event
        self.monitoring_bridge.publish_event(
            task.id.clone(),
            "task_analysis_completed".to_string(),
            serde_json::json!({
                "complexity_score": analysis.subtask_scores.parallelization_score,
                "should_parallelize": analysis.subtask_scores.parallelization_score > 0.6,
                "estimated_workers": analysis.recommended_workers,
            }),
        ).await.ok(); // Don't fail execution if monitoring fails

        if !self.should_execute_parallel(&analysis) {
            // Update status to sequential execution
            self.monitoring_bridge.update_task_progress(
                &task.id,
                ExecutionStatus::Running,
                0.0,
                Some("sequential_fallback".to_string()),
                std::collections::HashMap::new(),
            ).await.ok();

            // Fall back to sequential execution
            return self.execute_sequential(task).await;
        }

        // Update status to parallel execution
        self.monitoring_bridge.update_task_progress(
            &task.id,
            ExecutionStatus::Running,
            0.1,
            Some("decomposition".to_string()),
            std::collections::HashMap::new(),
        ).await.ok();

        // 2. Decompose the task into subtasks
        let subtasks = self.decomposition_engine.decompose(analysis)?;

        // Publish decomposition event
        self.monitoring_bridge.publish_event(
            task.id.clone(),
            "task_decomposed".to_string(),
            serde_json::json!({
                "subtask_count": subtasks.len(),
                "total_estimated_effort": subtasks.iter().map(|s| s.estimated_effort.as_secs()).sum::<u64>(),
            }),
        ).await.ok();

        // Update progress to execution phase
        self.monitoring_bridge.update_task_progress(
            &task.id,
            ExecutionStatus::Running,
            0.2,
            Some("execution".to_string()),
            std::collections::HashMap::new(),
        ).await.ok();

        // 3. Initialize progress tracking
        self.progress_aggregator = ProgressAggregator::new(task.id.clone());

        // 4. Execute subtasks in parallel
        let results = self.execute_subtasks_parallel(subtasks).await?;

        // Publish execution completion event
        let successful_results = results.iter().filter(|r| r.success).count();
        self.monitoring_bridge.publish_event(
            task.id.clone(),
            "parallel_execution_completed".to_string(),
            serde_json::json!({
                "total_subtasks": results.len(),
                "successful_subtasks": successful_results,
                "failed_subtasks": results.len() - successful_results,
            }),
        ).await.ok();

        // Update progress to validation phase
        self.monitoring_bridge.update_task_progress(
            &task.id,
            ExecutionStatus::Running,
            0.8,
            Some("validation".to_string()),
            std::collections::HashMap::new(),
        ).await.ok();

        // 5. Validate quality gates (if enabled)
        if self.config.enable_quality_gates {
            self.validate_results(&task.id, &results).await?;
        }

        // 6. Synthesize final result
        let task_result = self.progress_synthesizer.synthesize_results(results)?;

        // Update final progress
        self.monitoring_bridge.update_task_progress(
            &task.id,
            ExecutionStatus::Completed,
            1.0,
            Some("completed".to_string()),
            std::collections::HashMap::new(),
        ).await.ok();

        Ok(task_result)
    }

    /// Analyze task to determine execution strategy
    async fn analyze_task(&self, task: &ComplexTask) -> ParallelResult<TaskAnalysis> {
        tracing::info!("Analyzing task complexity: {}", task.description);

        let analysis = self.decomposition_engine.analyze(task)
            .await
            .map_err(|e| ParallelError::Decomposition {
                message: format!("Failed to analyze task: {:?}", e),
                source: None,
            })?;

        tracing::info!(
            "Task analysis complete: {} patterns, {} recommended workers, parallelizable: {}",
            analysis.patterns.len(),
            analysis.recommended_workers,
            analysis.should_parallelize
        );

        Ok(analysis)
    }

    /// Determine if task should be executed in parallel
    fn should_execute_parallel(&self, analysis: &TaskAnalysis) -> bool {
        analysis.should_parallelize && analysis.subtask_scores.parallelization_score >= self.config.complexity_threshold
    }

    /// Execute subtasks in parallel
    async fn execute_subtasks_parallel(&mut self, subtasks: Vec<SubTask>) -> ParallelResult<Vec<WorkerResult>> {
        tracing::info!("Executing {} subtasks in parallel", subtasks.len());

        // Spawn workers for each subtask
        let mut worker_handles = Vec::new();

        for subtask in subtasks {
            // Register subtask with progress tracker
            self.progress_aggregator.register_worker(
                WorkerId(subtask.id.0.clone()),
                subtask.id.clone(),
                1.0, // Equal weight for now
            )?;

            // Spawn worker
            let worker_id = self.worker_manager.spawn_worker(subtask).await?;
            worker_handles.push(worker_id);
        }

        // Wait for all workers to complete
        let mut results = Vec::new();
        for worker_id in worker_handles {
            match self.worker_manager.wait_for_worker(&worker_id).await {
                Ok(result) => {
                    // TODO: Update progress tracking with worker completion
                    // For now, just collect the results

                    results.push(result);
                }
                Err(e) => {
                    tracing::error!("Worker {} failed: {:?}", worker_id.0, e);
                    // Continue with other workers
                }
            }
        }

        tracing::info!("Parallel execution complete: {}/{} subtasks successful",
                      results.iter().filter(|r| r.success).count(),
                      results.len());

        Ok(results)
    }

    /// Validate results against quality gates
    async fn validate_results(&self, task_id: &TaskId, results: &[WorkerResult]) -> ParallelResult<()> {
        tracing::info!("Running quality gate validation");

        // Create validation context
        let validation_context = ValidationContext {
            package_name: "parallel-execution".to_string(), // TODO: Make configurable
            workspace_root: std::env::current_dir()
                .map_err(|e| ParallelError::Io {
                    message: format!("Failed to get workspace root: {}", e),
                    source: e,
                })?,
            results: results.to_vec(),
            execution_time: std::time::Duration::from_secs(0), // TODO: Track actual time
        };

        // Run validation
        let report = self.validation_runner.run_parallel(&validation_context).await?;

        if !report.passed() {
            return Err(ParallelError::Validation {
                message: format!("Internal quality gates failed: {}", report.summary.failed_gates),
                source: None,
            });
        }

        // Run orchestration quality gates for additional validation
        tracing::info!("Running orchestration quality gates");

        // Convert results to execution artifacts for orchestration validation
        let artifacts = self.convert_results_to_artifacts(results);

        let orchestration_validation = self.quality_bridge.validate_with_orchestration_gates(
            task_id,
            &artifacts,
            &QualityRequirements::default(), // TODO: Extract from task
        ).await?;

        match orchestration_validation {
            crate::ValidationResult::Pass { .. } => {
                tracing::info!("Orchestration quality gates passed");
            }
            crate::ValidationResult::Fail { details, .. } => {
                return Err(ParallelError::Validation {
                    message: format!("Orchestration quality gates failed: {}", details),
                    source: None,
                });
            }
            crate::ValidationResult::Warning { details, .. } => {
                tracing::warn!("Orchestration quality gates warning: {}", details);
                // Warnings don't fail execution, just log
            }
        }

        tracing::info!("All quality gates passed: {}/{} internal gates successful",
                      report.summary.passed_gates,
                      report.summary.total_gates);

        Ok(())
    }

    /// Convert worker results to execution artifacts for orchestration validation
    fn convert_results_to_artifacts(&self, _results: &[WorkerResult]) -> ExecutionArtifacts {
        // TODO: Implement proper artifact conversion from worker results
        // For now, return minimal artifacts
        ExecutionArtifacts {
            test_results: None,
            coverage_report: None,
            lint_report: None,
            type_check_report: None,
            mutation_report: None,
            provenance_record: None,
        }
    }

    /// Fall back to sequential execution
    async fn execute_sequential(&self, task: ComplexTask) -> ParallelResult<TaskResult> {
        tracing::info!("Falling back to sequential execution for task: {}", task.description);

        if let Some(orchestrator) = &self.orchestrator_handle {
            // Convert ComplexTask back to regular Task
            // This is a simplified conversion - in practice would need proper mapping
            orchestrator.execute_sequential(task).await
        } else {
            Err(ParallelError::Coordination {
                message: "No orchestrator handle available for sequential fallback".to_string(),
                source: None,
            })
        }
    }

    /// Get current progress
    pub fn get_progress(&self) -> Progress {
        self.progress_aggregator.get_overall_progress()
    }

    /// Cancel all running workers
    pub async fn cancel_all(&mut self) -> ParallelResult<()> {
        tracing::info!("Cancelling all workers");

        let active_worker_ids = self.worker_manager.active_worker_ids();

        for worker_id in active_worker_ids {
            if let Err(e) = self.worker_manager.cancel_worker(&worker_id).await {
                tracing::error!("Failed to cancel worker {}: {:?}", worker_id.0, e);
            }
        }

        Ok(())
    }

    /// Get execution statistics
    pub fn get_statistics(&self) -> ParallelExecutionStats {
        let worker_stats = self.worker_manager.get_statistics();
        let progress = self.get_progress();

        ParallelExecutionStats {
            active_workers: worker_stats.total_workers,
            completed_subtasks: progress.completed_subtasks,
            total_subtasks: progress.total_subtasks,
            overall_progress: progress.percentage,
            estimated_completion: progress.estimated_completion,
        }
    }
}


/// Statistics for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionStats {
    pub active_workers: usize,
    pub completed_subtasks: usize,
    pub total_subtasks: usize,
    pub overall_progress: f32,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

/// Integration helpers for orchestration layer
pub mod integration {
    use super::*;

    /// Check if a task should be routed to parallel execution
    pub fn should_route_to_parallel(
        task_description: &str,
        complexity_score: f32,
        config: &ParallelCoordinatorConfig,
    ) -> bool {
        // Check for parallelizable keywords
        let parallelizable_keywords = [
            "fix", "errors", "compile", "refactor", "test", "document",
            "parallel", "concurrent", "multiple", "batch",
        ];

        let has_parallelizable_content = parallelizable_keywords
            .iter()
            .any(|keyword| task_description.to_lowercase().contains(keyword));

        let meets_complexity_threshold = complexity_score >= config.complexity_threshold;

        has_parallelizable_content && meets_complexity_threshold
    }

    /// Estimate parallelization benefit
    pub fn estimate_parallelization_benefit(
        task_description: &str,
        estimated_subtasks: Option<usize>,
    ) -> f32 {
        let base_benefit = 0.5; // Base parallelization benefit

        // Adjust based on task characteristics
        let mut multiplier = 1.0;

        if task_description.to_lowercase().contains("error") {
            multiplier += 0.3; // Error fixing is highly parallelizable
        }

        if task_description.to_lowercase().contains("test") {
            multiplier += 0.2; // Testing can be parallelized
        }

        if task_description.to_lowercase().contains("refactor") {
            multiplier += 0.1; // Refactoring has some parallelization potential
        }

        // Adjust based on estimated subtasks
        if let Some(subtask_count) = estimated_subtasks {
            if subtask_count > 4 {
                multiplier += 0.2; // Many subtasks = good parallelization candidate
            } else if subtask_count <= 2 {
                multiplier -= 0.3; // Few subtasks = may not benefit from parallelism
            }
        }

        ((base_benefit * multiplier) as f32).min(1.0f32).max(0.0f32)
    }

    // TODO: Add convert_to_complex_task method when integrating with orchestration
    // This will convert Task from orchestration crate to ComplexTask for parallel execution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParallelCoordinatorConfig::default();
        assert_eq!(config.max_concurrent_workers, 8);
        assert_eq!(config.complexity_threshold, 0.6);
        assert!(config.enable_quality_gates);
    }

    #[test]
    fn test_should_route_to_parallel() {
        let config = ParallelCoordinatorConfig::default();

        // Should route compilation errors
        assert!(integration::should_route_to_parallel(
            "Fix compilation errors in the codebase",
            0.8,
            &config
        ));

        // Should not route simple tasks
        assert!(!integration::should_route_to_parallel(
            "Add a simple comment",
            0.2,
            &config
        ));
    }

    #[test]
    fn test_estimate_parallelization_benefit() {
        // High benefit for error fixing
        let benefit = integration::estimate_parallelization_benefit(
            "Fix compilation errors",
            Some(10)
        );
        assert!(benefit > 0.7);

        // Low benefit for simple tasks
        let benefit = integration::estimate_parallelization_benefit(
            "Add a comment",
            Some(1)
        );
        assert!(benefit < 0.5);
    }
}
