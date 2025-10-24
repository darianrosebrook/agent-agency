//! Parallel Task Execution
//!
//! Provides parallel task decomposition and coordination capabilities
//! consolidated from the parallel-workers/ crate.

use crate::types::*;
use crate::decomposition::TaskDecomposer;
use crate::execution::{ToolExecutor, ExecutionContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Configuration for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    pub max_parallel_tasks: usize,
    pub decomposition_depth: usize,
    pub enable_dependency_tracking: bool,
    pub coordination_timeout_seconds: u64,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallel_tasks: 10,
            decomposition_depth: 3,
            enable_dependency_tracking: true,
            coordination_timeout_seconds: 300,
        }
    }
}

/// Parallel execution coordinator
pub struct ParallelCoordinator {
    config: ParallelExecutionConfig,
    decomposer: Arc<TaskDecomposer>,
    tool_executor: Arc<ToolExecutor>,
    active_executions: Arc<RwLock<HashMap<TaskId, ParallelExecutionPlan>>>,
}

impl ParallelCoordinator {
    /// Create a new parallel coordinator
    pub fn new() -> Self {
        Self::with_config(ParallelExecutionConfig::default())
    }

    /// Create coordinator with custom configuration
    pub fn with_config(config: ParallelExecutionConfig) -> Self {
        Self {
            config: config.clone(),
            decomposer: Arc::new(TaskDecomposer::new()),
            tool_executor: Arc::new(ToolExecutor::new()),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Decompose a complex task into parallel subtasks
    pub async fn decompose_task(&self, task: &TaskDefinition) -> Result<ParallelExecutionPlan, ParallelError> {
        info!("Decomposing task {}: {}", task.id, task.name);

        // Analyze task complexity
        let analysis = self.decomposer.analyze_task(task).await?;

        // Create subtasks based on analysis
        let subtasks = self.create_subtasks(&analysis, task).await?;

        // Determine dependencies
        let dependencies = self.calculate_dependencies(&subtasks).await?;

        // Choose coordination strategy
        let strategy = self.select_coordination_strategy(&analysis);

        let plan = ParallelExecutionPlan {
            main_task: task.clone(),
            subtasks,
            dependencies,
            coordination_strategy: strategy,
        };

        // Store the execution plan
        let mut executions = self.active_executions.write().await;
        executions.insert(task.id, plan.clone());

        Ok(plan)
    }

    /// Execute a parallel execution plan
    pub async fn execute_parallel(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        info!("Executing parallel plan for task {}", plan.main_task.id);

        match plan.coordination_strategy {
            CoordinationStrategy::FullyParallel => {
                self.execute_fully_parallel(plan).await
            }
            CoordinationStrategy::SequentialDependencies => {
                self.execute_with_dependencies(plan).await
            }
            CoordinationStrategy::Adaptive => {
                self.execute_adaptive(plan).await
            }
        }
    }

    /// Execute all subtasks in parallel without dependencies
    async fn execute_fully_parallel(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        let mut handles = Vec::new();

        for subtask in &plan.subtasks {
            let tool_executor = Arc::clone(&self.tool_executor);
            let subtask = subtask.clone();

            let handle = tokio::spawn(async move {
                let context = ExecutionContext {
                    task_id: subtask.id,
                    worker_id: WorkerId::new_v4(), // Would be assigned by worker pool
                    tool_id: subtask.tool_id.clone(),
                    parameters: subtask.parameters.clone(),
                    timeout: std::time::Duration::from_secs(60),
                };

                tool_executor.execute_tool(context).await
            });

            handles.push(handle);
        }

        // Wait for all subtasks to complete
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(e) => {
                    warn!("Subtask execution failed: {}", e);
                    return Err(ParallelError::SubtaskFailed(e.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Execute subtasks respecting dependencies
    async fn execute_with_dependencies(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        let mut results = Vec::new();
        let mut completed_tasks = std::collections::HashSet::new();

        // Simple topological sort for dependencies
        let mut remaining_tasks: Vec<_> = plan.subtasks.iter().cloned().collect();

        while !remaining_tasks.is_empty() {
            // Find tasks with satisfied dependencies
            let mut executable_tasks = Vec::new();

            for task in &remaining_tasks {
                let dependencies_satisfied = plan.dependencies
                    .iter()
                    .filter(|dep| dep.dependent_task == task.id)
                    .all(|dep| completed_tasks.contains(&dep.dependency_task));

                if dependencies_satisfied {
                    executable_tasks.push(task.clone());
                }
            }

            if executable_tasks.is_empty() {
                return Err(ParallelError::CircularDependency);
            }

            // Execute executable tasks in parallel
            let mut handles = Vec::new();
            for task in executable_tasks {
                let tool_executor = Arc::clone(&self.tool_executor);
                let task = task.clone();

                let handle = tokio::spawn(async move {
                    let context = ExecutionContext {
                        task_id: task.id,
                        worker_id: WorkerId::new_v4(),
                        tool_id: task.tool_id.clone(),
                        parameters: task.parameters.clone(),
                        timeout: std::time::Duration::from_secs(60),
                    };

                    tool_executor.execute_tool(context).await
                });

                handles.push(handle);
            }

            // Wait for this batch to complete
            for handle in handles {
                let result = handle.await??;
                results.push(result);
            }

            // Mark tasks as completed
            for task in &remaining_tasks {
                completed_tasks.insert(task.id);
            }

            // Remove completed tasks
            remaining_tasks.retain(|task| !completed_tasks.contains(&task.id));
        }

        Ok(results)
    }

    /// Execute with adaptive coordination based on results
    async fn execute_adaptive(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        // Start with parallel execution, then adapt based on results
        let mut results = self.execute_fully_parallel(plan.clone()).await?;

        // Analyze results and potentially re-execute failed tasks
        let failed_tasks: Vec<_> = results.iter()
            .filter(|r| !r.success)
            .collect();

        if !failed_tasks.is_empty() {
            warn!("{} subtasks failed, attempting recovery", failed_tasks.len());

            // Retry failed tasks sequentially
            for failed_result in failed_tasks {
                let subtask = plan.subtasks.iter()
                    .find(|t| t.id == failed_result.task_id)
                    .ok_or(ParallelError::SubtaskNotFound)?;

                let context = ExecutionContext {
                    task_id: subtask.id,
                    worker_id: WorkerId::new_v4(),
                    tool_id: subtask.tool_id.clone(),
                    parameters: subtask.parameters.clone(),
                    timeout: std::time::Duration::from_secs(120), // Longer timeout for retry
                };

                let retry_result = self.tool_executor.execute_tool(context).await?;
                results.push(retry_result);
            }
        }

        Ok(results)
    }

    /// Synthesize results from multiple subtasks
    pub async fn synthesize_results(&self, results: Vec<TaskResult>) -> Result<TaskResult, ParallelError> {
        // Combine outputs from all subtasks
        let success = results.iter().all(|r| matches!(r.status, TaskStatus::Completed));

        let combined_output = if success {
            let outputs: Vec<_> = results.iter()
                .filter_map(|r| r.output.as_ref())
                .collect();

            Some(serde_json::json!({
                "subtask_results": outputs,
                "total_subtasks": results.len(),
                "successful_subtasks": results.iter().filter(|r| r.success).count()
            }))
        } else {
            None
        };

        let total_execution_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();

        Ok(TaskResult {
            task_id: TaskId::new_v4(), // Would be the main task ID
            status: if success { TaskStatus::Completed } else { TaskStatus::Failed },
            output: combined_output,
            error_message: if success { None } else { Some("Some subtasks failed".to_string()) },
            execution_time_ms: total_execution_time,
            tool_used: "parallel-coordinator".to_string(),
            quality_score: None,
        })
    }

    /// Create subtasks from task analysis
    async fn create_subtasks(&self, analysis: &crate::decomposition::TaskAnalysis, main_task: &TaskDefinition) -> Result<Vec<SubTask>, ParallelError> {
        let mut subtasks = Vec::new();

        // Create subtasks based on analysis patterns
        for pattern in &analysis.patterns {
            let subtask = SubTask {
                id: TaskId::new_v4(),
                parent_task_id: main_task.id,
                name: format!("{}-subtask-{}", main_task.name, subtasks.len()),
                description: pattern.description.clone(),
                tool_id: self.select_tool_for_pattern(pattern).await?,
                parameters: main_task.parameters.clone(),
                priority: main_task.priority,
            };
            subtasks.push(subtask);
        }

        Ok(subtasks)
    }

    /// Calculate dependencies between subtasks
    async fn calculate_dependencies(&self, subtasks: &[SubTask]) -> Result<Vec<TaskDependency>, ParallelError> {
        let mut dependencies = Vec::new();

        // Simple dependency calculation - in a real implementation,
        // this would analyze the task relationships
        for (i, subtask) in subtasks.iter().enumerate() {
            for j in 0..i {
                if self.has_dependency(subtask, &subtasks[j]) {
                    dependencies.push(TaskDependency {
                        dependent_task: subtask.id,
                        dependency_task: subtasks[j].id,
                        dependency_type: DependencyType::Completion,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    /// Select coordination strategy based on task analysis
    fn select_coordination_strategy(&self, _analysis: &crate::decomposition::TaskAnalysis) -> CoordinationStrategy {
        // For now, default to fully parallel
        // In a real implementation, this would analyze the task characteristics
        CoordinationStrategy::FullyParallel
    }

    /// Select appropriate tool for a task pattern
    async fn select_tool_for_pattern(&self, pattern: &crate::decomposition::TaskPattern) -> Result<ToolId, ParallelError> {
        match pattern.pattern_type.as_str() {
            "react-component" => Ok("react-generator".to_string()),
            "file-editing" => Ok("file-editor".to_string()),
            "research" => Ok("research-assistant".to_string()),
            _ => Ok("general-purpose".to_string()),
        }
    }

    /// Check if two subtasks have a dependency
    fn has_dependency(&self, _task1: &SubTask, _task2: &SubTask) -> bool {
        // Simple dependency check - in a real implementation,
        // this would analyze the actual task relationships
        false
    }
}

/// Errors from parallel execution
#[derive(Debug, thiserror::Error)]
pub enum ParallelError {
    #[error("Task decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Subtask not found")]
    SubtaskNotFound,

    #[error("Subtask execution failed: {0}")]
    SubtaskFailed(String),

    #[error("Result synthesis failed: {0}")]
    SynthesisFailed(String),
}
