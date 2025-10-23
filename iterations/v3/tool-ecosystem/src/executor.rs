//! Chain Executor with Concurrency & Cancellation
//!
//! Tokio-based executor with bounded queues, semaphores, and CancellationToken
//! for safe, concurrent tool chain execution with circuit breakers.

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Topo;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn, error};
use std::time::{Duration, Instant};

use crate::tool_chain_planner::{ToolChain, ToolNode, ToolEdge, ChainResult};
use crate::tool_execution::{ToolExecutor, ToolInvocation, ToolResult};
use crate::schema_registry::{SchemaRegistry, Converter};
use crate::tool_registry::ToolRegistry;

/// Chain execution result
#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub chain_hash: u64,
    pub success: bool,
    pub results: HashMap<NodeIndex, Value>,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
    pub cancelled_steps: Vec<String>,
}

/// Chain executor with concurrency control
#[derive(Clone)]
pub struct ChainExecutor {
    tool_executor: Arc<ToolExecutor>,
    schema_registry: Arc<SchemaRegistry>,
    concurrency_limit: usize,
    semaphore: Arc<Semaphore>,
    default_timeout_ms: u64,
}

impl ChainExecutor {
    /// Create a new chain executor
    pub fn new(
        tool_executor: Arc<ToolExecutor>,
        schema_registry: Arc<SchemaRegistry>,
        concurrency_limit: usize,
        default_timeout_ms: u64,
    ) -> Self {
        Self {
            tool_executor,
            schema_registry,
            concurrency_limit,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            default_timeout_ms,
        }
    }

    /// Execute a tool chain with cancellation support
    pub async fn execute(
        &self,
        chain: &ToolChain,
        cancel: CancellationToken,
    ) -> Result<ExecutionResult, ChainExecutionError> {
        let start_time = Instant::now();
        info!("Executing tool chain with {} steps", chain.dag.node_count());

        let mut results = HashMap::new();
        let mut errors = Vec::new();
        let mut cancelled_steps = Vec::new();

        // Create topological iterator
        let mut topo = Topo::new(&chain.dag);

        while let Some(node_idx) = topo.next(&chain.dag) {
            // Check for cancellation
            if cancel.is_cancelled() {
                warn!("Chain execution cancelled");
                cancelled_steps.push(self.node_name(&chain.dag[node_idx]));
                break;
            }

            // Acquire concurrency permit
            let permit = self.semaphore.acquire().await
                .map_err(|_| ChainExecutionError::ConcurrencyError)?;

            let node = &chain.dag[node_idx];
            let node_name = self.node_name(node);

            debug!("Executing step: {}", node_name);

            // Gather inputs from predecessors
            let inputs = self.gather_inputs(&chain, node_idx, &results).await?;

            // Execute the step
            match self.execute_step(node, inputs, cancel.clone()).await {
                Ok(result) => {
                    debug!("Step {} completed successfully", node_name);
                    results.insert(node_idx, result);
                }
                Err(e) => {
                    error!("Step {} failed: {}", node_name, e);
                    errors.push(format!("{}: {}", node_name, e));

                    // Check if we should continue with fallback
                    if let Some(fallback) = &node.fallback {
                        warn!("Attempting fallback to tool: {}", fallback);
                        if let Ok(fallback_result) = self.execute_fallback(fallback, node, inputs, cancel.clone()).await {
                            results.insert(node_idx, fallback_result);
                            continue;
                        }
                    }

                    // No fallback or fallback failed - stop execution
                    break;
                }
            }

            // Release permit
            drop(permit);
        }

        let execution_time = start_time.elapsed().as_millis() as u64;
        let success = errors.is_empty() && !cancel.is_cancelled();

        let result = ExecutionResult {
            chain_hash: chain.plan_hash,
            success,
            results,
            execution_time_ms: execution_time,
            errors,
            cancelled_steps,
        };

        info!("Chain execution completed in {}ms (success: {})",
              execution_time, success);

        Ok(result)
    }

    /// Gather inputs from predecessor nodes
    async fn gather_inputs(
        &self,
        chain: &ToolChain,
        node_idx: NodeIndex,
        results: &HashMap<NodeIndex, Value>,
    ) -> Result<Value, ChainExecutionError> {
        let mut inputs = serde_json::Map::new();

        // Get all incoming edges
        for edge in chain.dag.edges_directed(node_idx, petgraph::Direction::Incoming) {
            let (from_idx, _) = (edge.source(), edge.target());
            let from_node = &chain.dag[from_idx];
            let edge_meta = edge.weight();

            // Get result from predecessor
            let predecessor_result = results.get(&from_idx)
                .ok_or_else(|| ChainExecutionError::MissingDependency(
                    self.node_name(from_node)
                ))?;

            // Apply codec if specified
            let processed_value = if let Some(codec) = &edge_meta.codec {
                self.apply_codec(codec, &edge_meta.from_port, &edge_meta.to_port, predecessor_result.clone()).await?
            } else {
                predecessor_result.clone()
            };

            // Validate against target schema
            let to_port = &edge_meta.to_port;
            let node = &chain.dag[node_idx];
            if let Some(target_port) = node.inputs.iter().find(|p| p.name == *to_port) {
                self.schema_registry.validate(&target_port.schema.registry_key, &processed_value)
                    .map_err(ChainExecutionError::SchemaValidation)?;
            }

            inputs.insert(to_port.clone(), processed_value);
        }

        Ok(Value::Object(inputs))
    }

    /// Execute a single step
    async fn execute_step(
        &self,
        node: &ToolNode,
        inputs: Value,
        cancel: CancellationToken,
    ) -> Result<Value, ChainExecutionError> {
        let timeout_ms = node.sla_ms.min(self.default_timeout_ms);

        // Create invocation
        let invocation = ToolInvocation {
            tool_name: node.tool_id.clone(),
            parameters: inputs,
            context: Some(format!("chain_execution_timeout_{}", timeout_ms)),
            timeout_ms: Some(timeout_ms),
        };

        // Execute with timeout and cancellation
        let execution_future = self.tool_executor.execute_tool(invocation);
        let result = tokio::select! {
            result = execution_future => result,
            _ = cancel.cancelled() => return Err(ChainExecutionError::Cancelled),
            _ = tokio::time::sleep(Duration::from_millis(timeout_ms)) => {
                return Err(ChainExecutionError::Timeout(timeout_ms));
            }
        }?;

        // Validate output schema
        for output in &node.outputs {
            self.schema_registry.validate(&output.schema.registry_key, &result.result)
                .map_err(ChainExecutionError::SchemaValidation)?;
        }

        Ok(result.result)
    }

    /// Execute fallback tool
    async fn execute_fallback(
        &self,
        fallback_tool_id: &str,
        original_node: &ToolNode,
        inputs: Value,
        cancel: CancellationToken,
    ) -> Result<Value, ChainExecutionError> {
        // Create a temporary node for the fallback
        let fallback_node = ToolNode {
            tool_id: fallback_tool_id.to_string(),
            inputs: original_node.inputs.clone(),
            outputs: original_node.outputs.clone(),
            params: original_node.params.clone(),
            fallback: None, // No fallback for fallback
            sla_ms: original_node.sla_ms * 2, // Give more time
            cost_hint: original_node.cost_hint * 1.5, // Allow higher cost
            retry_policy: original_node.retry_policy.clone(),
        };

        self.execute_step(&fallback_node, inputs, cancel).await
    }

    /// Apply codec to transform data between ports
    async fn apply_codec(
        &self,
        codec: &str,
        from_port: &str,
        to_port: &str,
        value: Value,
    ) -> Result<Value, ChainExecutionError> {
        // Get input/output schemas for conversion
        let from_schema = format!("codec_input_{}", from_port);
        let to_schema = format!("codec_output_{}", to_port);

        self.schema_registry.convert(&from_schema, &to_schema, value)
            .await
            .map_err(ChainExecutionError::CodecError)
    }

    /// Get human-readable node name
    fn node_name(&self, node: &ToolNode) -> String {
        format!("{}_{}", node.tool_id, node.sla_ms)
    }
}

/// Errors from chain execution
#[derive(Debug, thiserror::Error)]
pub enum ChainExecutionError {
    #[error("Missing dependency result for step: {0}")]
    MissingDependency(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Codec transformation failed: {0}")]
    CodecError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Tool execution failed: {0}")]
    ToolExecution(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Execution cancelled")]
    Cancelled,

    #[error("Concurrency limit exceeded")]
    ConcurrencyError,

    #[error("Circuit breaker open for tool: {0}")]
    CircuitBreakerOpen(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

/// Circuit breaker for tool reliability
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure: Option<Instant>,
}

#[derive(Clone, Debug, PartialEq)]
enum CircuitState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_threshold,
            recovery_timeout,
            last_failure: None,
        }
    }

    pub fn record_success(&mut self) {
        self.state = CircuitState::Closed;
        self.last_failure = None;
    }

    pub fn record_failure(&mut self) {
        if let Some(last) = self.last_failure {
            if last.elapsed() > self.recovery_timeout {
                // Reset failure count after timeout
                self.state = CircuitState::Closed;
            }
        }

        match &self.state {
            CircuitState::Closed => {
                // Would track failure count in real implementation
                self.last_failure = Some(Instant::now());
            }
            CircuitState::HalfOpen => {
                // Failure in half-open, go back to open
                let next_attempt = Instant::now() + self.recovery_timeout;
                self.state = CircuitState::Open { until: next_attempt };
            }
            CircuitState::Open { .. } => {
                // Already open, extend timeout
                let next_attempt = Instant::now() + self.recovery_timeout;
                self.state = CircuitState::Open { until: next_attempt };
            }
        }
    }

    pub fn should_attempt(&mut self) -> bool {
        match &self.state {
            CircuitState::Closed => true,
            CircuitState::Open { until } => {
                if Instant::now() >= *until {
                    self.state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}

/// Resource limiter for preventing overload
pub struct ResourceLimiter {
    memory_limit_mb: usize,
    cpu_limit_percent: usize,
    current_memory_mb: std::sync::atomic::AtomicUsize,
    current_cpu_percent: std::sync::atomic::AtomicUsize,
}

impl ResourceLimiter {
    pub fn new(memory_limit_mb: usize, cpu_limit_percent: usize) -> Self {
        Self {
            memory_limit_mb,
            cpu_limit_percent,
            current_memory_mb: std::sync::atomic::AtomicUsize::new(0),
            current_cpu_percent: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn check_limits(&self, required_memory_mb: usize, required_cpu_percent: usize) -> bool {
        let current_mem = self.current_memory_mb.load(std::sync::atomic::Ordering::Relaxed);
        let current_cpu = self.current_cpu_percent.load(std::sync::atomic::Ordering::Relaxed);

        current_mem + required_memory_mb <= self.memory_limit_mb &&
        current_cpu + required_cpu_percent <= self.cpu_limit_percent
    }

    pub fn allocate_resources(&self, memory_mb: usize, cpu_percent: usize) -> ResourceGuard {
        self.current_memory_mb.fetch_add(memory_mb, std::sync::atomic::Ordering::Relaxed);
        self.current_cpu_percent.fetch_add(cpu_percent, std::sync::atomic::Ordering::Relaxed);

        ResourceGuard {
            limiter: self,
            memory_mb,
            cpu_percent,
        }
    }
}

/// RAII guard for resource cleanup
pub struct ResourceGuard<'a> {
    limiter: &'a ResourceLimiter,
    memory_mb: usize,
    cpu_percent: usize,
}

impl<'a> Drop for ResourceGuard<'a> {
    fn drop(&mut self) {
        self.limiter.current_memory_mb.fetch_sub(self.memory_mb, std::sync::atomic::Ordering::Relaxed);
        self.limiter.current_cpu_percent.fetch_sub(self.cpu_percent, std::sync::atomic::Ordering::Relaxed);
    }
}
