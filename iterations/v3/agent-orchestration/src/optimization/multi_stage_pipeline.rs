//! Multi-Stage Decision Pipeline
//!
//! Implements a fast-path classification system with worker selection optimization
//! and dual-execution orchestration for low-latency task routing.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, debug, warn};
use chrono::Utc;

use agent_agency_contracts::planning_io::Milestone;
use crate::planning::worker_assignment::WorkerAssignmentStrategy;

/// Task classification result
#[derive(Debug, Clone)]
pub struct TaskClassification {
    /// Task complexity level
    pub complexity: TaskComplexity,
    
    /// Estimated execution time in milliseconds
    pub estimated_time_ms: u64,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    
    /// Classification confidence (0.0 - 1.0)
    pub confidence: f64,
    
    /// Classification latency in milliseconds
    pub classification_latency_ms: u64,
}

/// Task complexity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskComplexity {
    /// Simple task, fast-path eligible
    Simple,
    /// Medium complexity, standard path
    Medium,
    /// Complex task, requires full evaluation
    Complex,
    /// Critical task, requires careful handling
    Critical,
}

/// Worker selection optimization result
#[derive(Debug, Clone)]
pub struct WorkerSelectionResult {
    /// Selected worker ID
    pub worker_id: Uuid,
    
    /// Alternative worker IDs (for dual-execution)
    pub alternative_workers: Vec<Uuid>,
    
    /// Selection confidence
    pub confidence: f64,
    
    /// Selection latency in milliseconds
    pub selection_latency_ms: u64,
    
    /// Whether dual-execution is recommended
    pub dual_execution_recommended: bool,
}

/// Dual-execution configuration
#[derive(Debug, Clone)]
pub struct DualExecutionConfig {
    /// Enable dual-execution
    pub enabled: bool,
    
    /// Maximum time difference for dual-execution (ms)
    pub max_time_diff_ms: u64,
    
    /// Minimum confidence threshold for dual-execution
    pub min_confidence: f64,
}

impl Default for DualExecutionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_time_diff_ms: 1000, // 1 second
            min_confidence: 0.7,
        }
    }
}

/// Pipeline stage result
#[derive(Debug, Clone)]
pub enum PipelineStageResult {
    /// Fast-path classification completed
    FastPath(TaskClassification),
    /// Standard path classification completed
    StandardPath(TaskClassification),
    /// Worker selection completed
    WorkerSelected(WorkerSelectionResult),
    /// Dual-execution initiated
    DualExecution(WorkerSelectionResult),
}

/// Multi-stage decision pipeline
pub struct MultiStagePipeline {
    /// Worker assignment strategy
    worker_assignment_strategy: Arc<WorkerAssignmentStrategy>,
    
    /// Fast-path classification threshold (ms)
    fast_path_threshold_ms: u64,
    
    /// Dual-execution configuration
    dual_execution_config: DualExecutionConfig,
    
    /// Backpressure threshold (concurrent tasks)
    backpressure_threshold: usize,
    
    /// Current concurrent task count
    concurrent_tasks: Arc<tokio::sync::RwLock<usize>>,
}

impl MultiStagePipeline {
    /// Create a new multi-stage pipeline
    pub fn new(
        worker_assignment_strategy: Arc<WorkerAssignmentStrategy>,
        fast_path_threshold_ms: u64,
        dual_execution_config: DualExecutionConfig,
        backpressure_threshold: usize,
    ) -> Self {
        Self {
            worker_assignment_strategy,
            fast_path_threshold_ms,
            dual_execution_config,
            backpressure_threshold,
            concurrent_tasks: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    /// Process milestone through multi-stage pipeline
    pub async fn process_milestone(
        &self,
        milestone: &Milestone,
    ) -> Result<PipelineStageResult> {
        let start_time = Instant::now();

        // Stage 1: Fast-path classification (<50ms target)
        let classification = self.classify_task_fast(milestone).await?;
        
        let classification_latency = start_time.elapsed().as_millis() as u64;
        debug!(
            "Task classification completed in {}ms: {:?}",
            classification_latency, classification.complexity
        );

        // Check if we can use fast-path
        if classification_latency < self.fast_path_threshold_ms
            && matches!(classification.complexity, TaskComplexity::Simple)
        {
            // Fast-path: Simple task, quick worker selection
            let worker_id = self.select_worker_fast(milestone, &classification).await?;
            
            return Ok(PipelineStageResult::FastPath(classification));
        }

        // Stage 2: Standard path - full worker selection optimization
        let selection_result = self.optimize_worker_selection(milestone, &classification).await?;

        // Stage 3: Dual-execution decision
        if self.should_use_dual_execution(&selection_result, &classification) {
            return Ok(PipelineStageResult::DualExecution(selection_result));
        }

        Ok(PipelineStageResult::WorkerSelected(selection_result))
    }

    /// Fast-path task classification (<50ms target)
    async fn classify_task_fast(&self, milestone: &Milestone) -> Result<TaskClassification> {
        let start_time = Instant::now();

        // Fast heuristics for classification
        let complexity = self.estimate_complexity_fast(milestone);
        let estimated_time_ms = self.estimate_time_fast(milestone);
        let required_capabilities = milestone.scope.allowed_operations.clone();
        
        // Simple confidence calculation based on milestone metadata
        let confidence = if milestone.objective.len() > 100 {
            0.8 // Longer objectives are more reliable
        } else {
            0.6 // Shorter objectives less reliable
        };

        let classification_latency_ms = start_time.elapsed().as_millis() as u64;

        Ok(TaskClassification {
            complexity,
            estimated_time_ms,
            required_capabilities,
            confidence,
            classification_latency_ms,
        })
    }

    /// Estimate task complexity using fast heuristics
    fn estimate_complexity_fast(&self, milestone: &Milestone) -> TaskComplexity {
        // Check risk tier
        if milestone.risk_tier >= 3 {
            return TaskComplexity::Critical;
        }

        // Estimate based on description length and dependencies
        let description_length = milestone.objective.len();
        let dependency_count = milestone.dependencies.len();
        let estimated_duration = milestone.estimated_duration.unwrap_or(60);

        // Simple heuristic: short description + few dependencies + short duration = simple
        if description_length < 200
            && dependency_count == 0
            && estimated_duration < 300
        {
            TaskComplexity::Simple
        } else if description_length < 500
            && dependency_count < 3
            && estimated_duration < 1800
        {
            TaskComplexity::Medium
        } else {
            TaskComplexity::Complex
        }
    }

    /// Estimate execution time using fast heuristics
    fn estimate_time_fast(&self, milestone: &Milestone) -> u64 {
        // Use milestone's estimated duration if available
        if let Some(duration_minutes) = milestone.estimated_duration {
            return (duration_minutes * 60 * 1000) as u64; // Convert to milliseconds
        }

        // Fallback: estimate based on description length
        let base_time_ms = 5000; // 5 seconds base
        let description_factor = milestone.objective.len() as u64 * 10; // 10ms per character
        
        base_time_ms + description_factor
    }

    /// Fast worker selection for simple tasks
    async fn select_worker_fast(
        &self,
        milestone: &Milestone,
        classification: &TaskClassification,
    ) -> Result<Uuid> {
        // Use worker assignment strategy but with simplified evaluation
        self.worker_assignment_strategy.assign_worker(milestone).await
    }

    /// Optimize worker selection with full evaluation
    async fn optimize_worker_selection(
        &self,
        milestone: &Milestone,
        classification: &TaskClassification,
    ) -> Result<WorkerSelectionResult> {
        let start_time = Instant::now();

        // Check backpressure
        if self.is_backpressured().await {
            warn!("Pipeline backpressured, using fast selection");
            let worker_id = self.select_worker_fast(milestone, classification).await?;
            return Ok(WorkerSelectionResult {
                worker_id,
                alternative_workers: Vec::new(),
                confidence: 0.7,
                selection_latency_ms: start_time.elapsed().as_millis() as u64,
                dual_execution_recommended: false,
            });
        }

        // Get worker recommendations
        let recommendations = self
            .worker_assignment_strategy
            .get_assignment_recommendations(milestone)
            .await?;

        if recommendations.is_empty() {
            return Err(anyhow::anyhow!("No worker recommendations available"));
        }

        let primary_worker = recommendations[0];
        let alternative_workers = recommendations.iter().skip(1).take(2).cloned().collect();

        // Calculate confidence based on number of recommendations and classification
        let confidence = if recommendations.len() >= 3 {
            0.9 // Multiple good options
        } else if recommendations.len() >= 2 {
            0.8 // At least one alternative
        } else {
            0.7 // Single option
        };

        let selection_latency_ms = start_time.elapsed().as_millis() as u64;

        Ok(WorkerSelectionResult {
            worker_id: primary_worker,
            alternative_workers,
            confidence,
            selection_latency_ms,
            dual_execution_recommended: false, // Will be determined separately
        })
    }

    /// Determine if dual-execution should be used
    fn should_use_dual_execution(
        &self,
        selection_result: &WorkerSelectionResult,
        classification: &TaskClassification,
    ) -> bool {
        if !self.dual_execution_config.enabled {
            return false;
        }

        // Check confidence threshold
        if selection_result.confidence < self.dual_execution_config.min_confidence {
            return false;
        }

        // Check if we have alternative workers
        if selection_result.alternative_workers.is_empty() {
            return false;
        }

        // Use dual-execution for complex tasks with high confidence
        matches!(classification.complexity, TaskComplexity::Complex | TaskComplexity::Critical)
            && selection_result.confidence >= 0.8
    }

    /// Check if pipeline is backpressured
    async fn is_backpressured(&self) -> bool {
        let current = *self.concurrent_tasks.read().await;
        current >= self.backpressure_threshold
    }

    /// Increment concurrent task count
    pub async fn increment_concurrent_tasks(&self) {
        let mut count = self.concurrent_tasks.write().await;
        *count += 1;
    }

    /// Decrement concurrent task count
    pub async fn decrement_concurrent_tasks(&self) {
        let mut count = self.concurrent_tasks.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }

    /// Get current concurrent task count
    pub async fn get_concurrent_task_count(&self) -> usize {
        *self.concurrent_tasks.read().await
    }
}

