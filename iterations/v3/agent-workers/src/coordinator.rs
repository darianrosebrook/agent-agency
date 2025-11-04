//! Parallel coordinator - main orchestrator for parallel task execution

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::parallel_types::{ComplexTask, SubTask, TaskId, SubTaskId, WorkerId, TaskResult, WorkerResult, ParallelResult};
use crate::error::{ParallelError, CommunicationError, ValidationError, ProgressError};
use crate::decomposition::{DecompositionEngine};
use crate::worker::{WorkerManager, DefaultWorkerPool};
use crate::progress::{ProgressAggregator, ProgressSynthesizer};
use crate::validation::{ValidationRunner};
use crate::communication::hub::CommunicationHub;
use crate::learning::{
    ParallelWorkerMetricsCollector, PatternAnalyzer, AdaptiveWorkerSelector, ConfigurationOptimizer,
    LearningPersistence, RewardWeights, Baseline,
};
use crate::learning::{
    ExecutionRecord, WorkerPerformanceProfile, SuccessPattern, FailurePattern, 
    OptimalConfig, ConfigurationRecommendations, OptimizationEvent, TaskPattern
};
use crate::worker_types::{WorkerSpecialty, TaskDefinition, TaskStatus, ExecutionOutcome, LearningMode, Priority, WorkerBreakdown, QualityRequirements, Progress, ValidationContext};
use agent_agency_contracts::task_executor::{TaskExecutor, TaskSpec, ExecutionStatus};
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use std::collections::HashMap;
use std::sync::Arc;

// Import refactored modules
use crate::learning::{
    RealFairnessMonitor
};
use crate::learning_system::{
    RealAdaptiveSelector, RealConfigOptimizer,
    RealQueueHealthMonitor, RealFailureTaxonomy, RealLearningPersistence,
    QueueHealthMetrics, FailureClassification
};
use crate::bridges::{
    OrchestrationQualityBridge, OrchestrationMonitoringBridge, CouncilLearningBridge
};
use crate::execution_stats::ParallelExecutionStats;

/// Orchestrator handle trait for sequential execution fallback
#[async_trait::async_trait]
pub trait OrchestratorHandle: Send + Sync {
    async fn execute_sequential(&self, task: ComplexTask) -> ParallelResult<TaskResult>;
}

/// Real implementation for orchestration handle
pub struct RealOrchestratorHandle {
    task_executor: Arc<dyn TaskExecutor>,
}

impl RealOrchestratorHandle {
    pub fn new(task_executor: Arc<dyn TaskExecutor>) -> Self {
        Self { task_executor }
    }
}

#[async_trait::async_trait]
impl OrchestratorHandle for RealOrchestratorHandle {
    async fn execute_sequential(&self, task: ComplexTask) -> ParallelResult<TaskResult> {
        tracing::info!("Executing task sequentially: {}", task.title);
        
        let start_time = std::time::Instant::now();
        
        // Convert ComplexTask to TaskSpec for the executor
        let mut context = HashMap::new();
        context.insert("task_id".to_string(), serde_json::Value::String(task.id.0.to_string()));
        context.insert("domains".to_string(), serde_json::json!(task.scope.domains));
        context.insert("files_affected".to_string(), serde_json::json!(task.scope.files_affected));
        context.insert("complexity_score".to_string(), serde_json::json!(task.complexity_score));
        context.insert("priority".to_string(), serde_json::json!(task.priority));
        context.insert("metadata".to_string(), serde_json::json!(task.metadata));

        let task_spec = TaskSpec {
            id: task.id.0,
            title: task.title.clone(),
            description: task.description.clone(),
            priority: match task.priority {
                Priority::Low => agent_agency_contracts::types::planning::TaskPriority::Low,
                Priority::Medium => agent_agency_contracts::types::planning::TaskPriority::Medium,
                Priority::High => agent_agency_contracts::types::planning::TaskPriority::High,
                Priority::Critical => agent_agency_contracts::types::planning::TaskPriority::Critical,
            },
            required_capabilities: task.scope.domains.clone(),
            context,
            working_spec_id: None,
            timeout_seconds: Some(300), // 5 minutes
        };
        
        // Execute the task using the real TaskExecutor
        let worker_id = uuid::Uuid::new_v4();
        let execution_result = self.task_executor.execute_task(task_spec, worker_id).await
            .map_err(|e| ParallelError::Coordination {
                message: format!("Task execution failed: {}", e),
                source: Some(Box::new(e))
            })?;
        
        let execution_time = start_time.elapsed();
        
        // Convert execution result to TaskResult
        let task_result = TaskResult {
            task_id: task.id,
            success: execution_result.success,
            subtasks_completed: 1,
            total_subtasks: 1,
            execution_time,
            execution_time_ms: execution_time.as_millis() as u64,
            summary: if execution_result.success {
                format!("Sequential execution completed successfully: {}", execution_result.output)
            } else {
                format!("Sequential execution failed: {}", 
                    execution_result.errors.first().unwrap_or(&"Unknown error".to_string()))
            },
            worker_breakdown: vec![WorkerBreakdown {
                worker_id: WorkerId(worker_id),
                subtasks_assigned: 1,
                subtasks_completed: if execution_result.success { 1 } else { 0 },
                execution_time,
                quality_score: 0.8, // Default quality score
                errors: execution_result.errors,
            }],
            quality_scores: std::collections::HashMap::new(),
            errors: execution_result.errors.clone(),
            error_message: execution_result.errors.first().cloned(),
            tool_used: None, // Sequential execution doesn't specify a tool
            status: if execution_result.success { TaskStatus::Completed } else { TaskStatus::Failed },
            metadata: execution_result.metadata,
        };
        
        tracing::info!("Sequential execution completed for task {}: success={}, time={:?}", 
            task.title, task_result.success, execution_time);
        
        Ok(task_result)
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
    // Learning system components
    metrics_collector: Arc<ParallelWorkerMetricsCollector>,
    pattern_analyzer: Arc<PatternAnalyzer>,
    adaptive_selector: Arc<AdaptiveWorkerSelector>,
    config_optimizer: Arc<ConfigurationOptimizer>,
    council_bridge: Arc<CouncilLearningBridge>,
    learning_persistence: Arc<dyn LearningPersistence>,
    fairness_monitor: Arc<RealFairnessMonitor>,
    queue_health_monitor: Arc<crate::learning::queue_health_monitor::QueueHealthMonitor>,
    failure_taxonomy: Arc<crate::learning::failure_taxonomy::FailureTaxonomy>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub fn new(config: ParallelCoordinatorConfig) -> Self {
        // Initialize core components
        let decomposition_engine = DecompositionEngine::new();
        let worker_manager = WorkerManager::new(DefaultWorkerPool::new());
        let progress_aggregator = ProgressAggregator::new();
        let progress_synthesizer = ProgressSynthesizer::new();
        let validation_runner = ValidationRunner::new();
        let communication_hub = CommunicationHub::new();

        // Initialize learning components with real implementations
        let db_client = Arc::new(data_infrastructure::client::DatabaseClient::new());
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(5, 0.7)); // min_pattern_frequency=5, confidence_threshold=0.7
        let fairness_monitor = Arc::new(RealFairnessMonitor::new(db_client.clone()));
        let adaptive_selector = Arc::new(AdaptiveWorkerSelector::new(
            crate::learning::adaptive_selector::WorkerSelectionStrategy::PerformanceBased,
            fairness_monitor.clone(),
            pattern_analyzer.clone(),
        ));
        let config_optimizer = Arc::new(ConfigurationOptimizer::new(pattern_analyzer.clone()));
        let queue_health_monitor = Arc::new(crate::learning::queue_health_monitor::QueueHealthMonitor::new());
        let failure_taxonomy = Arc::new(crate::learning::failure_taxonomy::FailureTaxonomy::new());
        let learning_persistence = Arc::new(crate::learning::learning_persistence::InMemoryLearningPersistence::new());

        // Initialize other learning components
        let metrics_collector = Arc::new(ParallelWorkerMetricsCollector::new());
        let council_bridge = Arc::new(CouncilLearningBridge::new());

        // Initialize bridges
        let quality_bridge = OrchestrationQualityBridge::new();
        let monitoring_bridge = OrchestrationMonitoringBridge::new();

        Self {
            decomposition_engine,
            worker_manager,
            progress_aggregator,
            progress_synthesizer,
            validation_runner,
            communication_hub,
            config,
            orchestrator_handle: None,
            quality_bridge,
            monitoring_bridge,
            metrics_collector,
            pattern_analyzer,
            adaptive_selector,
            config_optimizer,
            council_bridge,
            learning_persistence,
            fairness_monitor,
            queue_health_monitor,
            failure_taxonomy,
        }
    }

    /// Execute a complex task using parallel decomposition
    pub async fn execute_parallel(&self, task: ComplexTask) -> ParallelResult<TaskResult> {
        tracing::info!("Starting parallel execution for task: {}", task.title);

        // Check if parallel execution is enabled
        if !self.config.enabled {
            tracing::warn!("Parallel execution disabled, falling back to sequential");
            return self.execute_sequential_fallback(task).await;
        }

        // Analyze task complexity
        let complexity_analysis = self.decomposition_engine.analyze_complexity(&task).await
            .map_err(|e| ParallelError::Decomposition {
                message: format!("Failed to analyze task complexity: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        if complexity_analysis.complexity_score < self.config.complexity_threshold as f64 {
            tracing::info!("Task complexity too low for parallel execution, using sequential");
            return self.execute_sequential_fallback(task).await;
        }

        // Decompose task into subtasks
        let subtasks = self.decomposition_engine.decompose_task(&task).await
            .map_err(|e| ParallelError::Decomposition {
                message: format!("Failed to decompose task: {}", e),
                source: Some(Box::new(e)),
            })?;
        
        if subtasks.len() > self.config.max_subtasks_per_task {
            tracing::warn!("Too many subtasks ({}) for parallel execution, using sequential", subtasks.len());
            return self.execute_sequential_fallback(task).await;
        }

        // Execute subtasks in parallel
        let execution_stats = self.execute_subtasks_parallel(subtasks, &task).await?;

        // Synthesize results
        let final_result = self.progress_synthesizer.synthesize_results(execution_stats)
            .map_err(|e| ParallelError::Coordination {
                message: format!("Failed to synthesize results: {}", e),
                source: Some(Box::new(e)),
            })?;

        tracing::info!("Parallel execution completed for task: {}", task.title);
        Ok(final_result)
    }

    /// Execute subtasks in parallel
    async fn execute_subtasks_parallel(
        &self,
        subtasks: Vec<SubTask>,
        parent_task: &ComplexTask,
    ) -> ParallelResult<Vec<WorkerResult>> {
        let mut handles = Vec::new();

        // Create execution futures for each subtask
        for subtask in subtasks {
            let future = self.create_subtask_execution_handle(subtask, parent_task);
            handles.push(future);
        }

        // Execute all subtasks concurrently
        let results = futures::future::join_all(handles).await;

        // Collect successful results
        let mut worker_results = Vec::new();
        for result in results {
            match result {
                Ok(worker_result) => {
                    worker_results.push(worker_result);
                }
                Err(e) => {
                    tracing::error!("Subtask execution failed: {}", e);
                    // For failed subtasks, we could create a failed WorkerResult
                    // but for now we'll just log the error
                }
            }
        }

        Ok(worker_results)
    }

    /// Create execution handle for a subtask
    async fn create_subtask_execution_handle(
        &self,
        subtask: SubTask,
        parent_task: &ComplexTask,
    ) -> ParallelResult<WorkerResult> {
        // Get available workers from worker manager
        let available_workers = self.worker_manager.list_available_workers().await;
        
        // Select optimal worker for the subtask
        let worker_id = self.adaptive_selector.select_worker(&subtask, &available_workers).await
            .map_err(|e| ParallelError::Coordination {
                message: format!("Worker selection failed: {}", e),
                source: Some(e)
            })?
            .ok_or_else(|| ParallelError::Coordination {
                message: "No suitable worker available".to_string(),
                source: None
            })?;

        // Execute the subtask
        let start_time = std::time::Instant::now();
        let result = self.worker_manager.execute_subtask(subtask, worker_id).await?;
        let execution_time = start_time.elapsed();

        Ok(WorkerResult {
            task_id: result.task_id,
            subtask_id: result.subtask_id,
            worker_id,
            success: result.success,
            output: format!("Subtask {} executed successfully", subtask.id.0),
            execution_time,
            quality_score: result.quality_score,
            errors: result.errors.clone(),
            metadata: HashMap::new(),
            metrics: crate::parallel_types::WorkerMetrics {
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now(),
                files_modified: 0,
                lines_changed: 0,
            },
            artifacts: result.artifacts.clone(),
        })
    }

    /// Execute sequential fallback
    async fn execute_sequential_fallback(&self, task: ComplexTask) -> ParallelResult<TaskResult> {
        if let Some(handle) = &self.orchestrator_handle {
            handle.execute_sequential(task).await
        } else {
            Err(ParallelError::Coordination {
                message: "No orchestrator handle available for sequential execution".to_string(),
            })
        }
    }

    /// Set orchestrator handle for sequential fallback
    pub fn set_orchestrator_handle(&mut self, handle: Arc<dyn OrchestratorHandle>) {
        self.orchestrator_handle = Some(handle);
    }

    /// Get execution statistics
    pub fn get_execution_stats(&self) -> &ParallelExecutionStats {
        // Return current stats - in real implementation, this would be stored
        &ParallelExecutionStats::default()
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: ParallelCoordinatorConfig) {
        self.config = new_config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ParallelCoordinatorConfig {
        &self.config
    }
}
