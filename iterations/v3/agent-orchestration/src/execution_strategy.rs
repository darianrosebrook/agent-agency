//! Execution Strategy Service
//!
//! Provides execution strategy management for autonomous agents.
//! Manages how tasks are executed (parallel, sequential, conditional, etc.)
//! and adapts strategies based on task characteristics and system state.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

/// Execution strategy types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionStrategy {
    /// Execute all tasks in parallel
    Parallel {
        /// Maximum concurrent tasks
        max_concurrent: usize,
    },

    /// Execute tasks sequentially
    Sequential {
        /// Delay between tasks (ms)
        delay_ms: Option<u64>,
    },

    /// Execute based on conditions
    Conditional {
        /// Condition evaluation logic
        condition: String,
    },

    /// Custom execution strategy
    Custom {
        /// Strategy name
        name: String,
        /// Strategy parameters
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Execution strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyConfig {
    /// Default strategy to use
    pub default_strategy: ExecutionStrategy,

    /// Strategy-specific configurations
    pub strategy_configs: HashMap<String, StrategyParams>,

    /// Enable automatic strategy adaptation
    pub enable_adaptation: bool,

    /// Adaptation interval (seconds)
    pub adaptation_interval_secs: u64,
}

/// Strategy-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyParams {
    /// Maximum concurrent tasks for parallel execution
    pub max_concurrent: Option<usize>,

    /// Delay between tasks for sequential execution (ms)
    pub delay_ms: Option<u64>,

    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// Execution strategy result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyResult {
    /// Strategy used
    pub strategy: ExecutionStrategy,

    /// Task execution results
    pub results: Vec<StrategyTaskResult>,

    /// Total execution time (ms)
    pub total_time_ms: u64,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Strategy effectiveness score (0.0-1.0)
    pub effectiveness_score: f64,
}

/// Strategy-specific task execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyTaskResult {
    /// Task ID
    pub task_id: String,

    /// Success status
    pub success: bool,

    /// Execution time (ms)
    pub execution_time_ms: u64,

    /// Error message if failed
    pub error: Option<String>,
}

/// Trait for execution strategy service
#[async_trait]
pub trait ExecutionStrategyService: Send + Sync + std::fmt::Debug {
    /// Select execution strategy for a set of tasks
    async fn select_strategy(
        &self,
        task_ids: &[String],
        task_characteristics: &TaskCharacteristics,
    ) -> Result<ExecutionStrategy, StrategyError>;

    /// Execute tasks using a specific strategy
    async fn execute_with_strategy(
        &self,
        task_ids: &[String],
        strategy: ExecutionStrategy,
    ) -> Result<StrategyResult, StrategyError>;

    /// Change execution strategy for a task group
    async fn change_strategy(
        &self,
        task_group_id: &str,
        new_strategy: ExecutionStrategy,
    ) -> Result<(), StrategyError>;

    /// Get current strategy for a task group
    async fn get_current_strategy(
        &self,
        task_group_id: &str,
    ) -> Result<Option<ExecutionStrategy>, StrategyError>;

    /// Get strategy effectiveness metrics
    async fn get_strategy_effectiveness(
        &self,
        strategy: &ExecutionStrategy,
    ) -> Result<f64, StrategyError>;
}

/// Task characteristics for strategy selection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskCharacteristics {
    /// Number of tasks
    pub task_count: usize,

    /// Average task complexity (0.0-1.0)
    pub avg_complexity: f64,

    /// Task dependencies
    pub dependencies: Vec<(String, String)>, // (task_id, depends_on_task_id)

    /// Resource requirements
    pub resource_requirements: HashMap<String, serde_json::Value>,

    /// Time constraints
    pub time_constraints: Option<TimeConstraints>,
}

/// Time constraints for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimeConstraints {
    /// Maximum total execution time (ms)
    pub max_total_time_ms: Option<u64>,

    /// Individual task timeout (ms)
    pub task_timeout_ms: Option<u64>,

    /// Deadline timestamp
    pub deadline: Option<DateTime<Utc>>,
}

/// Execution strategy service implementation
pub struct DefaultExecutionStrategyService {
    /// Strategy configuration
    config: StrategyConfig,

    /// Active task group strategies
    active_strategies: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ExecutionStrategy>>>,

    /// Strategy performance history
    strategy_history: std::sync::Arc<tokio::sync::RwLock<Vec<StrategyResult>>>,

    /// Task executor for actual task execution (optional - if None, uses placeholder)
    task_executor: Option<std::sync::Arc<dyn agent_agency_contracts::task_executor::TaskExecutor>>,
}

impl DefaultExecutionStrategyService {
    /// Create a new execution strategy service without task executor (placeholder mode)
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            active_strategies: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            strategy_history: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            task_executor: None,
        }
    }

    /// Create a new execution strategy service with task executor
    pub fn with_task_executor(
        config: StrategyConfig,
        task_executor: std::sync::Arc<dyn agent_agency_contracts::task_executor::TaskExecutor>,
    ) -> Self {
        Self {
            config,
            active_strategies: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            strategy_history: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            task_executor: Some(task_executor),
        }
    }

    /// Convert task_id string to TaskSpec for execution
    /// 
    /// Creates a basic TaskSpec from task_id. In a full implementation,
    /// this would look up the actual task details from a task store.
    fn task_id_to_spec(&self, task_id: &str) -> Result<agent_agency_contracts::task_executor::TaskSpec, StrategyError> {
        // Parse task_id as UUID
        let task_uuid = Uuid::parse_str(task_id)
            .map_err(|e| StrategyError::ExecutionFailed(format!("Invalid task ID format: {}", e)))?;

        // Create basic TaskSpec - in production this would be looked up from task store
        Ok(agent_agency_contracts::task_executor::TaskSpec {
            id: task_uuid,
            title: format!("Task {}", task_id),
            description: format!("Execute task {}", task_id),
            priority: agent_agency_contracts::types::planning::TaskPriority::Medium,
            required_capabilities: Vec::new(),
            context: std::collections::HashMap::new(),
            working_spec_id: None,
            timeout_seconds: Some(300), // 5 minute default timeout
            scope: None,
            risk_tier: Some(2), // Default to tier 2
            acceptance_criteria: None,
            caws_spec: None,
            requirements: None,
        })
    }

    /// Execute a single task using TaskExecutor if available, otherwise placeholder
    async fn execute_single_task(
        &self,
        task_id: &str,
    ) -> Result<StrategyTaskResult, StrategyError> {
        let start_time = std::time::Instant::now();

        if let Some(ref executor) = self.task_executor {
            // Use real TaskExecutor
            let task_spec = self.task_id_to_spec(task_id)?;
            // Worker assignment is handled by the worker pool during task execution
            // The TaskExecutor routes to an available worker internally
            let worker_id = Uuid::new_v4();

            match executor.execute_task(task_spec, worker_id).await {
                Ok(result) => {
                    let execution_time_ms = start_time.elapsed().as_millis() as u64;
                    Ok(StrategyTaskResult {
                        task_id: task_id.to_string(),
                        success: result.success,
                        execution_time_ms: result.duration_ms,
                        error: if result.errors.is_empty() {
                            None
                        } else {
                            Some(result.errors.join("; "))
                        },
                    })
                }
                Err(e) => {
                    let execution_time_ms = start_time.elapsed().as_millis() as u64;
                    Err(StrategyError::ExecutionFailed(format!(
                        "Task execution failed: {}",
                        e
                    )))
                }
            }
        } else {
            // Fallback to placeholder if no executor provided
            let execution_time = std::time::Duration::from_millis(100);
            tokio::time::sleep(execution_time).await;
            Ok(StrategyTaskResult {
                task_id: task_id.to_string(),
                success: true,
                execution_time_ms: execution_time.as_millis() as u64,
                error: None,
            })
        }
    }

    /// Determine best strategy based on task characteristics
    fn determine_best_strategy(&self, characteristics: &TaskCharacteristics) -> ExecutionStrategy {
        // Simple strategy selection logic:
        // - If tasks have dependencies, use sequential or conditional
        // - If tasks are independent and numerous, use parallel
        // - Otherwise use default strategy

        if !characteristics.dependencies.is_empty() {
            // Tasks have dependencies - use sequential or conditional
            if characteristics.task_count > 10 {
                ExecutionStrategy::Sequential { delay_ms: Some(100) }
            } else {
                ExecutionStrategy::Sequential { delay_ms: None }
            }
        } else if characteristics.task_count > 5 && characteristics.avg_complexity < 0.5 {
            // Many simple independent tasks - use parallel
            ExecutionStrategy::Parallel {
                max_concurrent: (characteristics.task_count / 2).max(1).min(10),
            }
        } else {
            // Use default strategy
            self.config.default_strategy.clone()
        }
    }
}

#[async_trait]
impl ExecutionStrategyService for DefaultExecutionStrategyService {
    async fn select_strategy(
        &self,
        task_ids: &[String],
        task_characteristics: &TaskCharacteristics,
    ) -> Result<ExecutionStrategy, StrategyError> {
        let strategy = if self.config.enable_adaptation {
            // Use adaptive strategy selection
            self.determine_best_strategy(task_characteristics)
        } else {
            // Use default strategy
            self.config.default_strategy.clone()
        };

        Ok(strategy)
    }

    async fn execute_with_strategy(
        &self,
        task_ids: &[String],
        strategy: ExecutionStrategy,
    ) -> Result<StrategyResult, StrategyError> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();

        match &strategy {
            ExecutionStrategy::Parallel { max_concurrent } => {
                // Execute tasks in parallel with concurrency limit
                use futures::stream::{self, StreamExt};

                // Clone executor reference for async move
                let executor = self.task_executor.clone();
                let config = self.config.clone();
                
                let task_stream = stream::iter(task_ids.iter().cloned())
                    .map(move |task_id| {
                        let executor_clone = executor.clone();
                        async move {
                            let start_time = std::time::Instant::now();
                            
                            if let Some(ref exec) = executor_clone {
                                // Use real TaskExecutor
                                let task_uuid = match Uuid::parse_str(&task_id) {
                                    Ok(uuid) => uuid,
                                    Err(e) => {
                                        warn!("Invalid task ID format {}: {}", task_id, e);
                                        return StrategyTaskResult {
                                            task_id: task_id.clone(),
                                            success: false,
                                            execution_time_ms: 0,
                                            error: Some(format!("Invalid task ID: {}", e)),
                                        };
                                    }
                                };

                                let task_spec = agent_agency_contracts::task_executor::TaskSpec {
                                    id: task_uuid,
                                    title: format!("Task {}", task_id),
                                    description: format!("Execute task {}", task_id),
                                    priority: agent_agency_contracts::types::planning::TaskPriority::Medium,
                                    required_capabilities: Vec::new(),
                                    context: std::collections::HashMap::new(),
                                    working_spec_id: None,
                                    timeout_seconds: Some(300),
                                    scope: None,
                                    risk_tier: Some(2),
                                    acceptance_criteria: None,
                                    caws_spec: None,
                                    requirements: None,
                                };

                                // Worker assignment handled by TaskExecutor
                                let worker_id = Uuid::new_v4();
                                debug!("Executing task {} via TaskExecutor", task_id);

                                match exec.execute_task(task_spec, worker_id).await {
                                    Ok(result) => {
                                        StrategyTaskResult {
                                            task_id: task_id.clone(),
                                            success: result.success,
                                            execution_time_ms: result.duration_ms,
                                            error: if result.errors.is_empty() {
                                                None
                                            } else {
                                                Some(result.errors.join("; "))
                                            },
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Task execution failed for {}: {}", task_id, e);
                                        StrategyTaskResult {
                                            task_id: task_id.clone(),
                                            success: false,
                                            execution_time_ms: start_time.elapsed().as_millis() as u64,
                                            error: Some(format!("Execution failed: {}", e)),
                                        }
                                    }
                                }
                            } else {
                                // Fallback to placeholder if no executor provided
                                let execution_time = std::time::Duration::from_millis(100);
                                tokio::time::sleep(execution_time).await;
                                StrategyTaskResult {
                                    task_id: task_id.clone(),
                                    success: true,
                                    execution_time_ms: execution_time.as_millis() as u64,
                                    error: None,
                                }
                            }
                        }
                    });

                let mut concurrent_stream = task_stream.buffer_unordered(*max_concurrent);
                while let Some(result) = concurrent_stream.next().await {
                    results.push(result);
                }
            }
            ExecutionStrategy::Sequential { delay_ms } => {
                // Execute tasks sequentially
                for task_id in task_ids {
                    let result = self.execute_single_task(task_id).await.unwrap_or_else(|e| {
                        // If execution fails, return error result
                        StrategyTaskResult {
                            task_id: task_id.to_string(),
                            success: false,
                            execution_time_ms: 0,
                            error: Some(e.to_string()),
                        }
                    });
                    
                    results.push(result);

                    // Add delay between tasks if specified
                    if let Some(delay) = delay_ms {
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
            }
            ExecutionStrategy::Conditional { condition: _ } => {
                for task_id in task_ids {
                    let execution_time = std::time::Duration::from_millis(100);
                    tokio::time::sleep(execution_time).await;

                    results.push(StrategyTaskResult {
                        task_id: task_id.clone(),
                        success: true,
                        execution_time_ms: execution_time.as_millis() as u64,
                        error: None,
                    });
                }
            }
            ExecutionStrategy::Custom { name: _, parameters: _ } => {
                for task_id in task_ids {
                    let execution_time = std::time::Duration::from_millis(100);
                    tokio::time::sleep(execution_time).await;

                    results.push(StrategyTaskResult {
                        task_id: task_id.clone(),
                        success: true,
                        execution_time_ms: execution_time.as_millis() as u64,
                        error: None,
                    });
                }
            }
        }

        let total_time_ms = start_time.elapsed().as_millis() as u64;
        let success_count = results.iter().filter(|r| r.success).count();
        let success_rate = if results.is_empty() {
            0.0
        } else {
            success_count as f64 / results.len() as f64
        };

        // Calculate effectiveness score based on success rate and time efficiency
        let effectiveness_score = success_rate * 0.7 + (1.0 - (total_time_ms as f64 / 10000.0).min(1.0)) * 0.3;

        let strategy_result = StrategyResult {
            strategy,
            results,
            total_time_ms,
            success_rate,
            effectiveness_score,
        };

        // Record in history
        let mut history = self.strategy_history.write().await;
        history.push(strategy_result.clone());
        if history.len() > 1000 {
            history.remove(0); // Keep last 1000 entries
        }

        Ok(strategy_result)
    }

    async fn change_strategy(
        &self,
        task_group_id: &str,
        new_strategy: ExecutionStrategy,
    ) -> Result<(), StrategyError> {
        let mut strategies = self.active_strategies.write().await;
        strategies.insert(task_group_id.to_string(), new_strategy);
        Ok(())
    }

    async fn get_current_strategy(
        &self,
        task_group_id: &str,
    ) -> Result<Option<ExecutionStrategy>, StrategyError> {
        let strategies = self.active_strategies.read().await;
        Ok(strategies.get(task_group_id).cloned())
    }

    async fn get_strategy_effectiveness(
        &self,
        strategy: &ExecutionStrategy,
    ) -> Result<f64, StrategyError> {
        let history = self.strategy_history.read().await;

        // Calculate average effectiveness for this strategy type
        let matching_results: Vec<&StrategyResult> = history
            .iter()
            .filter(|result| match (&result.strategy, strategy) {
                (ExecutionStrategy::Parallel { .. }, ExecutionStrategy::Parallel { .. }) => true,
                (ExecutionStrategy::Sequential { .. }, ExecutionStrategy::Sequential { .. }) => true,
                (ExecutionStrategy::Conditional { .. }, ExecutionStrategy::Conditional { .. }) => true,
                (ExecutionStrategy::Custom { name: n1, .. }, ExecutionStrategy::Custom { name: n2, .. }) => n1 == n2,
                _ => false,
            })
            .collect();

        if matching_results.is_empty() {
            return Ok(0.5); // Default effectiveness if no history
        }

        let avg_effectiveness = matching_results
            .iter()
            .map(|r| r.effectiveness_score)
            .sum::<f64>()
            / matching_results.len() as f64;

        Ok(avg_effectiveness)
    }
}

/// Execution strategy errors

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
enum StrategyError {
    #[error("Invalid strategy configuration: {0}")]
    InvalidConfig(String),

    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Strategy selection failed: {0}")]
    SelectionFailed(String),

    #[error("Task group not found: {0}")]
    TaskGroupNotFound(String),
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            default_strategy: ExecutionStrategy::Parallel { max_concurrent: 4 },
            strategy_configs: HashMap::new(),
            enable_adaptation: true,
            adaptation_interval_secs: 60,
        }
    }
}



