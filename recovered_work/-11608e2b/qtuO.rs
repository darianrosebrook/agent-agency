//! Tool Coordinator - Orchestrates Complex Tool Chains and Workflows
//!
//! Manages the execution of multi-step tool chains, handles dependencies,
//! error recovery, and result aggregation for complex reasoning workflows.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, debug, warn, error};

use crate::tool_registry::{ToolRegistry, Tool};
use crate::tool_execution::{ToolExecutor, ToolInvocation, ToolResult};

/// Tool coordinator for orchestrating complex workflows
#[derive(Debug)]
pub struct ToolCoordinator {
    /// Tool registry reference
    tool_registry: Arc<ToolRegistry>,
    /// Tool executor reference
    tool_executor: Arc<ToolExecutor>,
    /// Active tool chains
    active_chains: Arc<RwLock<HashMap<String, ToolChainExecution>>>,
    /// Chain execution results
    execution_results: Arc<RwLock<HashMap<String, ToolExecutionResult>>>,
    /// Enable parallel execution
    enable_parallel: bool,
    /// Maximum concurrent chains
    max_concurrent_chains: usize,
}

/// Tool chain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChain {
    /// Chain ID
    pub id: String,
    /// Chain name
    pub name: String,
    /// Chain steps
    pub steps: Vec<ToolChainStep>,
    /// Chain metadata
    pub metadata: ToolChainMetadata,
}

/// Tool chain step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainStep {
    /// Step ID (unique within chain)
    pub step_id: String,
    /// Tool name to execute
    pub tool_name: String,
    /// Input parameters
    pub parameters: serde_json::Value,
    /// Step dependencies (other step IDs)
    pub dependencies: Vec<String>,
    /// Conditional execution
    pub condition: Option<String>,
    /// Timeout (ms)
    pub timeout_ms: Option<u64>,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Tool chain metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainMetadata {
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// Version
    pub version: String,
    /// Expected execution time (ms)
    pub expected_duration_ms: Option<u64>,
    /// Success criteria
    pub success_criteria: Option<String>,
}

/// Tool chain execution state
#[derive(Debug)]
pub struct ToolChainExecution {
    /// Chain definition
    pub chain: ToolChain,
    /// Execution state
    pub state: ExecutionState,
    /// Step results
    pub step_results: HashMap<String, StepResult>,
    /// Execution start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Current executing steps
    pub active_steps: Vec<String>,
    /// Completed steps
    pub completed_steps: Vec<String>,
    /// Failed steps
    pub failed_steps: Vec<String>,
}

/// Execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionState {
    /// Chain is queued for execution
    Queued,
    /// Chain is executing
    Executing,
    /// Chain completed successfully
    Completed,
    /// Chain failed
    Failed(String),
    /// Chain was cancelled
    Cancelled,
}

/// Step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step ID
    pub step_id: String,
    /// Execution result
    pub result: Option<ToolResult>,
    /// Error if failed
    pub error: Option<String>,
    /// Execution start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Execution end time
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Retry count
    pub retry_count: u32,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Chain ID
    pub chain_id: String,
    /// Overall success
    pub success: bool,
    /// Final result (aggregated from steps)
    pub final_result: Option<serde_json::Value>,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Steps executed
    pub steps_executed: usize,
    /// Steps failed
    pub steps_failed: usize,
    /// Chain metadata
    pub metadata: ToolChainMetadata,
    /// Execution trace
    pub execution_trace: Vec<ExecutionTrace>,
}

/// Execution trace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: TraceEventType,
    /// Event details
    pub details: String,
    /// Associated step ID
    pub step_id: Option<String>,
}

/// Trace event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEventType {
    ChainStarted,
    StepStarted,
    StepCompleted,
    StepFailed,
    StepRetried,
    ChainCompleted,
    ChainFailed,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries (ms)
    pub base_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay (ms)
    pub max_delay_ms: u64,
}

impl ToolCoordinator {
    /// Create a new tool coordinator
    pub fn new(enable_parallel: bool) -> Self {
        Self {
            tool_registry: Arc::new(ToolRegistry::new()),
            tool_executor: Arc::new(ToolExecutor::new(10, 30000)), // 10 concurrent, 30s timeout
            active_chains: Arc::new(RwLock::new(HashMap::new())),
            execution_results: Arc::new(RwLock::new(HashMap::new())),
            enable_parallel,
            max_concurrent_chains: 5,
        }
    }

    /// Create a new tool coordinator with custom settings
    pub fn with_settings(enable_parallel: bool, max_concurrent_chains: usize) -> Self {
        Self {
            max_concurrent_chains,
            ..Self::new(enable_parallel)
        }
    }

    /// Validate a tool chain
    pub async fn validate_chain(&self, chain: &ToolChain) -> Result<()> {
        info!("Validating tool chain: {}", chain.id);

        // Check for duplicate step IDs
        let mut step_ids = std::collections::HashSet::new();
        for step in &chain.steps {
            if !step_ids.insert(step.step_id.clone()) {
                return Err(anyhow::anyhow!("Duplicate step ID: {}", step.step_id));
            }
        }

        // Validate step dependencies
        for step in &chain.steps {
            for dep in &step.dependencies {
                if !chain.steps.iter().any(|s| s.step_id == *dep) {
                    return Err(anyhow::anyhow!("Step '{}' depends on non-existent step '{}'", step.step_id, dep));
                }
            }
        }

        // Check for circular dependencies
        self.detect_circular_dependencies(chain)?;

        // Validate tool existence
        for step in &chain.steps {
            if self.tool_registry.get_tool(&step.tool_name).await.is_none() {
                return Err(anyhow::anyhow!("Tool '{}' not found in registry", step.tool_name));
            }
        }

        debug!("Tool chain validation successful");
        Ok(())
    }

    /// Execute a tool chain
    pub async fn execute_chain(&self, chain: &ToolChain) -> Result<ToolExecutionResult> {
        let chain_id = chain.id.clone();
        info!("Executing tool chain: {}", chain_id);

        let start_time = chrono::Utc::now();

        // Initialize execution state
        let execution = ToolChainExecution {
            chain: chain.clone(),
            state: ExecutionState::Executing,
            step_results: HashMap::new(),
            started_at: start_time,
            active_steps: Vec::new(),
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
        };

        // Store execution state
        {
            let mut active = self.active_chains.write().await;
            active.insert(chain_id.clone(), execution);
        }

        // Execute chain
        let result = self.execute_chain_internal(&chain_id, chain).await;

        // Update final state
        {
            let mut active = self.active_chains.write().await;
            if let Some(execution) = active.get_mut(&chain_id) {
                match &result {
                    Ok(_) => execution.state = ExecutionState::Completed,
                    Err(e) => execution.state = ExecutionState::Failed(e.to_string()),
                }
            }
        }

        let execution_time = (chrono::Utc::now() - start_time).num_milliseconds() as u64;

        // Create final result
        let final_result = match result {
            Ok(result) => ToolExecutionResult {
                chain_id: chain_id.clone(),
                success: true,
                final_result: Some(result),
                execution_time_ms: execution_time,
                steps_executed: chain.steps.len(),
                steps_failed: 0,
                metadata: chain.metadata.clone(),
                execution_trace: vec![], // Would be populated during execution
            },
            Err(e) => ToolExecutionResult {
                chain_id: chain_id.clone(),
                success: false,
                final_result: None,
                execution_time_ms: execution_time,
                steps_executed: 0,
                steps_failed: chain.steps.len(),
                metadata: chain.metadata.clone(),
                execution_trace: vec![],
            },
        };

        // Store result
        {
            let mut results = self.execution_results.write().await;
            results.insert(chain_id.clone(), final_result.clone());
        }

        // Clean up active chains
        {
            let mut active = self.active_chains.write().await;
            active.remove(&chain_id);
        }

        result.map(|r| final_result).or_else(|e| Err(e))
    }

    /// Execute chain internally (handles the actual execution logic)
    async fn execute_chain_internal(&self, chain_id: &str, chain: &ToolChain) -> Result<serde_json::Value> {
        let mut step_results = HashMap::new();
        let mut ready_queue = VecDeque::new();
        let mut completed_steps = std::collections::HashSet::new();

        // Initialize with steps that have no dependencies
        for step in &chain.steps {
            if step.dependencies.is_empty() {
                ready_queue.push_back(step.clone());
            }
        }

        while !ready_queue.is_empty() {
            // Execute ready steps (in parallel if enabled)
            let steps_to_execute: Vec<_> = if self.enable_parallel {
                // Take all ready steps for parallel execution
                ready_queue.drain(..).collect()
            } else {
                // Take only the first step for sequential execution
                ready_queue.pop_front().into_iter().collect()
            };

            // Execute steps concurrently
            let mut handles = Vec::new();
            for step in steps_to_execute {
                let chain_id = chain_id.to_string();
                let step_id = step.step_id.clone();
                let tool_name = step.tool_name.clone();
                let parameters = step.parameters.clone();

                let handle = tokio::spawn(async move {
                    // Execute step
                    match self.execute_step(&chain_id, &step_id, &tool_name, parameters).await {
                        Ok(result) => Ok((step_id, result)),
                        Err(e) => Err((step_id, e.to_string())),
                    }
                });

                handles.push(handle);
            }

            // Wait for all steps to complete
            for handle in handles {
                match handle.await {
                    Ok(Ok((step_id, result))) => {
                        step_results.insert(step_id.clone(), result);
                        completed_steps.insert(step_id.clone());

                        // Find steps that are now ready (all dependencies satisfied)
                        for step in &chain.steps {
                            if !completed_steps.contains(&step.step_id) &&
                               !ready_queue.iter().any(|s| s.step_id == step.step_id) {
                                let all_deps_satisfied = step.dependencies.iter()
                                    .all(|dep| completed_steps.contains(dep));

                                if all_deps_satisfied {
                                    ready_queue.push_back(step.clone());
                                }
                            }
                        }
                    }
                    Ok(Err((step_id, error))) => {
                        return Err(anyhow::anyhow!("Step '{}' failed: {}", step_id, error));
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Task join error: {}", e));
                    }
                }
            }
        }

        // Check if all steps completed
        if completed_steps.len() != chain.steps.len() {
            let missing: Vec<_> = chain.steps.iter()
                .filter(|s| !completed_steps.contains(&s.step_id))
                .map(|s| s.step_id.clone())
                .collect();
            return Err(anyhow::anyhow!("Incomplete execution. Missing steps: {:?}", missing));
        }

        // Aggregate final result (simplified - take last step's result)
        if let Some(last_step) = chain.steps.last() {
            if let Some(result) = step_results.get(&last_step.step_id) {
                return Ok(result.result.clone());
            }
        }

        Err(anyhow::anyhow!("No result produced by chain"))
    }

    /// Execute a single step
    async fn execute_step(&self, chain_id: &str, step_id: &str, tool_name: &str, parameters: serde_json::Value) -> Result<StepResult> {
        let start_time = chrono::Utc::now();

        // Get tool from registry
        let tool = self.tool_registry.get_tool(tool_name).await
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", tool_name))?;

        // Execute tool
        let invocation = ToolInvocation {
            tool_name: tool_name.to_string(),
            parameters,
            context: Some(format!("chain:{},step:{}", chain_id, step_id)),
            timeout_ms: Some(30000), // 30 second timeout
        };

        match self.tool_executor.execute_tool(invocation).await {
            Ok(result) => {
                let end_time = chrono::Utc::now();
                Ok(StepResult {
                    step_id: step_id.to_string(),
                    result: Some(result),
                    error: None,
                    started_at: start_time,
                    ended_at: Some(end_time),
                    retry_count: 0,
                })
            }
            Err(e) => {
                let end_time = chrono::Utc::now();
                Ok(StepResult {
                    step_id: step_id.to_string(),
                    result: None,
                    error: Some(e.to_string()),
                    started_at: start_time,
                    ended_at: Some(end_time),
                    retry_count: 0,
                })
            }
        }
    }

    /// Detect circular dependencies in chain
    fn detect_circular_dependencies(&self, chain: &ToolChain) -> Result<()> {
        // Build dependency graph
        let mut graph = HashMap::new();
        for step in &chain.steps {
            graph.insert(step.step_id.clone(), step.dependencies.clone());
        }

        // Check for cycles using DFS
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for step_id in graph.keys() {
            if self.has_cycle(step_id, &graph, &mut visited, &mut recursion_stack) {
                return Err(anyhow::anyhow!("Circular dependency detected involving step '{}'", step_id));
            }
        }

        Ok(())
    }

    /// Check for cycles in dependency graph
    fn has_cycle(
        &self,
        step_id: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(step_id.to_string());
        recursion_stack.insert(step_id.to_string());

        if let Some(dependencies) = graph.get(step_id) {
            for dep in dependencies {
                if !visited.contains(dep) && self.has_cycle(dep, graph, visited, recursion_stack) {
                    return true;
                } else if recursion_stack.contains(dep) {
                    return true;
                }
            }
        }

        recursion_stack.remove(step_id);
        false
    }

    /// Get active chain executions
    pub async fn get_active_chains(&self) -> HashMap<String, ToolChainExecution> {
        self.active_chains.read().await.clone()
    }

    /// Get chain execution result
    pub async fn get_chain_result(&self, chain_id: &str) -> Option<ToolExecutionResult> {
        self.execution_results.read().await.get(chain_id).cloned()
    }

    /// Cancel a running chain
    pub async fn cancel_chain(&self, chain_id: &str) -> Result<()> {
        let mut active = self.active_chains.write().await;
        if let Some(execution) = active.get_mut(chain_id) {
            execution.state = ExecutionState::Cancelled;
            info!("Cancelled chain execution: {}", chain_id);
        }
        Ok(())
    }

    /// Get chain execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        let active = self.active_chains.read().await;
        let results = self.execution_results.read().await;

        let active_chains = active.len();
        let completed_chains = results.values().filter(|r| r.success).count();
        let failed_chains = results.values().filter(|r| !r.success).count();
        let total_execution_time = results.values().map(|r| r.execution_time_ms).sum::<u64>();
        let avg_execution_time = if results.is_empty() {
            0
        } else {
            total_execution_time / results.len() as u64
        };

        ExecutionStats {
            active_chains,
            completed_chains,
            failed_chains,
            total_chains: results.len(),
            avg_execution_time_ms: avg_execution_time,
        }
    }
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Number of currently active chains
    pub active_chains: usize,
    /// Number of completed chains
    pub completed_chains: usize,
    /// Number of failed chains
    pub failed_chains: usize,
    /// Total number of chains executed
    pub total_chains: usize,
    /// Average execution time (ms)
    pub avg_execution_time_ms: u64,
}

impl ToolChain {
    /// Create a new empty tool chain
    pub fn new() -> Self {
        Self {
            id: format!("chain_{}", uuid::Uuid::new_v4()),
            name: "Unnamed Chain".to_string(),
            steps: Vec::new(),
            metadata: ToolChainMetadata {
                description: "Tool chain".to_string(),
                author: "system".to_string(),
                version: "1.0".to_string(),
                expected_duration_ms: None,
                success_criteria: None,
            },
        }
    }

    /// Add a step to the chain
    pub fn add_step(&mut self, step: ToolChainStep) {
        self.steps.push(step);
    }

    /// Get steps in topological order
    pub fn get_steps_topological(&self) -> Result<Vec<&ToolChainStep>> {
        // Simplified topological sort - assumes no cycles (validated elsewhere)
        let mut result = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for step in &self.steps {
            if step.dependencies.is_empty() {
                result.push(step);
                processed.insert(&step.step_id);
            }
        }

        // Add dependent steps
        for _ in 0..self.steps.len() {
            let mut added = false;
            for step in &self.steps {
                if !processed.contains(&step.step_id) {
                    let deps_satisfied = step.dependencies.iter()
                        .all(|dep| processed.contains(dep));

                    if deps_satisfied {
                        result.push(step);
                        processed.insert(&step.step_id);
                        added = true;
                    }
                }
            }
            if !added {
                break; // No more steps can be added
            }
        }

        if result.len() != self.steps.len() {
            return Err(anyhow::anyhow!("Cannot determine execution order due to unresolved dependencies"));
        }

        Ok(result)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
        }
    }
}

