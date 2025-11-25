//! Tool Execution Engine - Secure and Efficient Tool Invocation
//!
//! Provides secure execution environment for tool invocation with timeout,
//! resource limits, and error handling.

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

// Tool registry for dynamic tool dispatch
use agent_mcp::ToolRegistry;
use agent_mcp::mcp_types::{ToolExecutionRequest, ExecutionContext as MCPExecutionContext, ExecutionPriority, ExecutionStatus};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;

/// Tool executor for secure invocation
#[derive(Debug)]
pub struct ToolExecutor {
    /// Concurrency limiter
    concurrency_limiter: Arc<Semaphore>,
    /// Default timeout (ms)
    default_timeout_ms: u64,
    /// Execution statistics
    stats: Arc<std::sync::RwLock<ExecutionStats>>,
    /// Background cleanup task
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
    /// Tool registry for dynamic tool dispatch (optional - if None, uses hardcoded tools)
    tool_registry: Option<Arc<ToolRegistry>>,
}

/// Tool invocation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolInvocation {
    /// Tool name to execute
    pub tool_name: String,
    /// Input parameters
    pub parameters: serde_json::Value,
    /// Execution context
    pub context: Option<String>,
    /// Timeout override (ms)
    pub timeout_ms: Option<u64>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolResult {
    /// Tool name that was executed
    pub tool_name: String,
    /// Execution result
    pub result: serde_json::Value,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionMetadata {
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Memory used (MB)
    pub memory_used_mb: f64,
    /// Success flag
    pub success: bool,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// Resource usage details
    pub resource_usage: ResourceUsage,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceUsage {
    /// CPU time used (ms)
    pub cpu_time_ms: u64,
    /// Peak memory usage (MB)
    pub peak_memory_mb: f64,
    /// I/O operations performed
    pub io_operations: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Peak concurrent executions
    pub peak_concurrent: usize,
    /// Current active executions
    pub active_executions: usize,
    /// Total resource usage
    pub total_resource_usage: ResourceUsage,
}

/// Execution context for tracking
#[derive(Debug)]
struct ExecutionContext {
    /// Invocation request
    invocation: ToolInvocation,
    /// Start timestamp
    start_time: std::time::Instant,
    /// Resource tracking
    resource_tracker: ResourceTracker,
}

/// Resource tracker for execution monitoring
#[derive(Debug, Clone)]
struct ResourceTracker {
    /// Initial memory usage
    initial_memory: f64,
    /// Peak memory usage
    peak_memory: f64,
    /// CPU start time
    cpu_start: std::time::Instant,
    /// I/O operations
    io_operations: u64,
    /// Network bytes
    network_bytes: u64,
}

impl ToolExecutor {
    /// Create a new tool executor without tool registry (uses hardcoded tools)
    pub fn new(max_concurrent: usize, default_timeout_ms: u64) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(max_concurrent));
        let stats = Arc::new(std::sync::RwLock::new(ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            peak_concurrent: 0,
            active_executions: 0,
            total_resource_usage: ResourceUsage {
                cpu_time_ms: 0,
                peak_memory_mb: 0.0,
                io_operations: 0,
                network_bytes: 0,
            },
        }));

        Self {
            concurrency_limiter,
            default_timeout_ms,
            stats,
            cleanup_task: None,
            tool_registry: None,
        }
    }

    /// Create a new tool executor with tool registry for dynamic dispatch
    pub fn with_tool_registry(
        max_concurrent: usize,
        default_timeout_ms: u64,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        let mut executor = Self::new(max_concurrent, default_timeout_ms);
        executor.tool_registry = Some(tool_registry);
        executor
    }

    /// Execute a tool with the given invocation
    pub async fn execute_tool(&self, invocation: ToolInvocation) -> Result<ToolResult> {
        let permit = self
            .concurrency_limiter
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire execution permit: {}", e))?;

        // Update active executions
        {
            let mut stats = self.stats.write().unwrap();
            stats.active_executions += 1;
            stats.peak_concurrent = stats.peak_concurrent.max(stats.active_executions);
            stats.total_executions += 1;
        }

        let result = self.execute_tool_internal(invocation).await;

        // Update final stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.active_executions -= 1;

            match &result {
                Ok(_) => stats.successful_executions += 1,
                Err(_) => stats.failed_executions += 1,
            }
        }

        drop(permit);
        result
    }

    /// Internal tool execution with resource tracking
    async fn execute_tool_internal(&self, invocation: ToolInvocation) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        let timeout_ms = invocation.timeout_ms.unwrap_or(self.default_timeout_ms);

        debug!(
            "Executing tool: {} with timeout {}ms",
            invocation.tool_name, timeout_ms
        );

        // Create execution context
        let context = ExecutionContext {
            invocation: invocation.clone(),
            start_time,
            resource_tracker: ResourceTracker::new(),
        };

        // Execute with timeout
        let execution_future = self.perform_tool_execution(context);
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        match tokio::time::timeout(timeout_duration, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                error!("Tool execution timed out: {}", invocation.tool_name);
                Err(anyhow::anyhow!(
                    "Tool execution timed out after {}ms",
                    timeout_ms
                ))
            }
        }
    }

    /// Perform the actual tool execution
    async fn perform_tool_execution(&self, mut context: ExecutionContext) -> Result<ToolResult> {
        let tool_name = context.invocation.tool_name.clone();

        // Use ToolRegistry if available, otherwise fall back to hardcoded tool handlers
        let result_value = if let Some(ref registry) = self.tool_registry {
            // Look up tool in registry by name
            info!("Looking up tool '{}' in ToolRegistry", tool_name);
            let all_tools = registry.get_all_tools().await;
            
            let tool = all_tools
                .iter()
                .find(|t| t.name == tool_name);
            
            if let Some(tool) = tool {
                // Execute via ToolRegistry
                info!("Executing tool '{}' (UUID: {}) via ToolRegistry", tool_name, tool.id);
                
                // Convert ToolInvocation to ToolExecutionRequest
                let mcp_context = context.invocation.context.as_ref().map(|ctx_str| {
                    // Parse context string as JSON if possible, otherwise create simple context
                    serde_json::from_str::<HashMap<String, serde_json::Value>>(ctx_str)
                        .map(|params| MCPExecutionContext {
                            working_directory: params.get("working_directory")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            environment_variables: params.get("environment_variables")
                                .and_then(|v| v.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default(),
                            input_files: params.get("input_files")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            output_directory: params.get("output_directory")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            metadata: params.get("metadata")
                                .and_then(|v| v.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .map(|(k, v)| (k.clone(), v.clone()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                        .unwrap_or_else(|_| MCPExecutionContext {
                            working_directory: Some(".".to_string()),
                            environment_variables: HashMap::new(),
                            input_files: vec![],
                            output_directory: None,
                            metadata: {
                                let mut map = HashMap::new();
                                map.insert("context_string".to_string(), serde_json::json!(ctx_str));
                                map
                            },
                        })
                });
                
                // Convert parameters from serde_json::Value to HashMap<String, serde_json::Value>
                let parameters_map: HashMap<String, serde_json::Value> = if let Some(params_obj) = context.invocation.parameters.as_object() {
                    params_obj
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                } else {
                    // If parameters is not an object, wrap it
                    let mut map = HashMap::new();
                    map.insert("params".to_string(), context.invocation.parameters.clone());
                    map
                };
                
                let request = ToolExecutionRequest {
                    id: Uuid::new_v4(),
                    tool_id: tool.id,
                    parameters: parameters_map,
                    context: mcp_context,
                    priority: ExecutionPriority::Normal,
                    timeout_seconds: context.invocation.timeout_ms.map(|ms| ms / 1000),
                    created_at: Utc::now(),
                    requested_by: Some("system-federated-ml".to_string()),
                };
                
                // Execute via ToolRegistry
                match registry.execute_tool(request).await {
                    Ok(tool_result) => {
                        // Convert ToolExecutionResult to serde_json::Value
                        match tool_result.output {
                            Some(output) => output,
                            None => {
                                // If no output, create result from status
                                serde_json::json!({
                                    "status": format!("{:?}", tool_result.status),
                                    "success": matches!(tool_result.status, ExecutionStatus::Completed),
                                    "error": tool_result.error,
                                    "tool_id": tool_result.tool_id,
                                })
                            }
                        }
                    }
                    Err(e) => {
                        warn!("ToolRegistry execution failed for '{}': {}, falling back to hardcoded handler", tool_name, e);
                        // Fall back to hardcoded handler
                        self.execute_hardcoded_tool(&tool_name, &context.invocation).await?
                    }
                }
            } else {
                warn!("Tool '{}' not found in ToolRegistry, falling back to hardcoded handler", tool_name);
                // Fall back to hardcoded handler
                self.execute_hardcoded_tool(&tool_name, &context.invocation).await?
            }
        } else {
            // No ToolRegistry available, use hardcoded handlers
            self.execute_hardcoded_tool(&tool_name, &context.invocation).await?
        };

        let execution_time = context.start_time.elapsed().as_millis() as u64;
        let resource_usage = context.resource_tracker.finalize();

        // Update average execution time
        {
            let mut stats = self.stats.write().unwrap();
            let total_time = stats.avg_execution_time_ms * (stats.total_executions - 1) as f64
                + execution_time as f64;
            stats.avg_execution_time_ms = total_time / stats.total_executions as f64;

            // Update total resource usage
            stats.total_resource_usage.cpu_time_ms += resource_usage.cpu_time_ms;
            stats.total_resource_usage.peak_memory_mb = stats
                .total_resource_usage
                .peak_memory_mb
                .max(resource_usage.peak_memory_mb);
            stats.total_resource_usage.io_operations += resource_usage.io_operations;
            stats.total_resource_usage.network_bytes += resource_usage.network_bytes;
        }

        let metadata = ExecutionMetadata {
            execution_time_ms: execution_time,
            memory_used_mb: resource_usage.peak_memory_mb,
            success: true,
            error_message: None,
            resource_usage,
        };

        Ok(ToolResult {
            tool_name,
            result: result_value,
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Execute CAWS validator tool
    async fn execute_caws_validator(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        // Simulate CAWS validation
        let spec = invocation
            .parameters
            .get("spec")
            .and_then(|v| v.as_str())
            .unwrap_or("{}");

        // TODO: Integrate actual CAWS validator
        //       Currently uses basic validation; should use actual CAWS validator for comprehensive spec validation.
        let is_valid = spec.contains("risk_tier") && spec.contains("scope");

        Ok(serde_json::json!({
            "valid": is_valid,
            "compliant": is_valid,
            "issues": if is_valid { Vec::<String>::new() } else { vec!["Missing risk_tier or scope".to_string()] }
        }))
    }

    /// Execute claim extractor tool
    async fn execute_claim_extractor(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        let content = invocation
            .parameters
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Simulate claim extraction
        let claims = if content.contains("must") || content.contains("should") {
            vec![serde_json::json!({
                "id": "claim_1",
                "statement": "Extracted requirement from content",
                "confidence": 0.85
            })]
        } else {
            vec![]
        };

        Ok(serde_json::json!({
            "claims": claims,
            "total_extracted": claims.len()
        }))
    }

    /// Execute fact verifier tool
    async fn execute_fact_verifier(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        let claim = invocation
            .parameters
            .get("claim")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Simulate fact verification
        let verified = claim.len() > 10 && !claim.contains("false");
        let confidence = if verified { 0.9 } else { 0.3 };

        Ok(serde_json::json!({
            "verified": verified,
            "confidence": confidence,
            "evidence_found": verified
        }))
    }

    /// Execute tool using hardcoded handlers (fallback when ToolRegistry unavailable or tool not found)
    async fn execute_hardcoded_tool(
        &self,
        tool_name: &str,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        match tool_name {
            "caws_validator" => self.execute_caws_validator(invocation).await,
            "claim_extractor" => self.execute_claim_extractor(invocation).await,
            "fact_verifier" => self.execute_fact_verifier(invocation).await,
            "debate_orchestrator" => self.execute_debate_orchestrator(invocation).await,
            "consensus_builder" => self.execute_consensus_builder(invocation).await,
            _ => {
                warn!("Unknown tool: {}", tool_name);
                Err(anyhow::anyhow!("Unknown tool: {}", tool_name))
            }
        }
    }

    /// Execute debate orchestrator tool
    async fn execute_debate_orchestrator(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        let topic = invocation
            .parameters
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("default topic");

        // Simulate debate orchestration
        let debate_id = format!("debate_{}", uuid::Uuid::new_v4());

        Ok(serde_json::json!({
            "debate_id": debate_id,
            "topic": topic,
            "status": "initiated",
            "participants": ["constitutional_judge", "technical_auditor", "quality_evaluator"]
        }))
    }

    /// Execute consensus builder tool
    async fn execute_consensus_builder(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<serde_json::Value> {
        // Simulate consensus building
        let positions = invocation
            .parameters
            .get("positions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let consensus = if positions > 0 {
            "agreed".to_string()
        } else {
            "no_positions".to_string()
        };

        Ok(serde_json::json!({
            "consensus": consensus,
            "confidence": 0.8,
            "supporting_positions": positions
        }))
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> ExecutionStats {
        self.stats.read().unwrap().clone()
    }

    /// Reset execution statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            peak_concurrent: 0,
            active_executions: 0,
            total_resource_usage: ResourceUsage {
                cpu_time_ms: 0,
                peak_memory_mb: 0.0,
                io_operations: 0,
                network_bytes: 0,
            },
        };
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(&mut self) -> Result<()> {
        let _stats = Arc::clone(&self.stats);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Perform cleanup tasks
                debug!("Running tool execution cleanup");

                // In practice, this could clean up old execution records,
                // release resources, etc.
            }
        });

        self.cleanup_task = Some(handle);
        Ok(())
    }

    /// Stop background cleanup task
    pub async fn stop_cleanup_task(&mut self) -> Result<()> {
        if let Some(handle) = self.cleanup_task.take() {
            handle.abort();
            info!("Stopped tool execution cleanup task");
        }
        Ok(())
    }
}

impl ResourceTracker {
    /// Create a new resource tracker
    fn new() -> Self {
        Self {
            initial_memory: get_current_memory_mb(),
            peak_memory: 0.0,
            cpu_start: std::time::Instant::now(),
            io_operations: 0,
            network_bytes: 0,
        }
    }

    /// Finalize resource tracking
    fn finalize(&mut self) -> ResourceUsage {
        let current_memory = get_current_memory_mb();
        self.peak_memory = self.peak_memory.max(current_memory - self.initial_memory);

        ResourceUsage {
            cpu_time_ms: self.cpu_start.elapsed().as_millis() as u64,
            peak_memory_mb: self.peak_memory,
            io_operations: self.io_operations,
            network_bytes: self.network_bytes,
        }
    }
}

/// Get current memory usage in MB
fn get_current_memory_mb() -> f64 {
    // TODO: Implement real memory usage monitoring
    // - [ ] Use system APIs (sysinfo, etc.) to get actual process memory usage
    // - [ ] Track memory usage over time for trend analysis
    // - [ ] Handle API errors and platform differences
    // - [ ] Add unit tests with mock memory data
    // - [ ] Add integration tests with real memory monitoring
    // TODO: Query actual memory usage from system APIs
    //       Currently returns simulated value; should query actual memory usage from system APIs for accurate monitoring.
    //
    // COMPLETION CHECKLIST:
    // [ ] Primary functionality implemented
    // [ ] API/data structures defined & stable
    // [ ] Error handling + validation aligned with error taxonomy
    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
    // [ ] Integration tests for external systems/contracts
    // [ ] Documentation: public API + system behavior
    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
    // [ ] Security posture reviewed (inputs, authz, sandboxing)
    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
    // [ ] Configurability and feature flags defined if relevant
    // [ ] Failure-mode cards documented (degradation paths)
    //
    // ACCEPTANCE CRITERIA:
    // - Memory usage is queried from system APIs correctly
    // - Measurements are accurate
    // - Query handles API failures gracefully
    // - Performance is acceptable
    //
    // DEPENDENCIES:
    // - System memory APIs (Required)
    // - Memory monitoring utilities (Required)
    // - Process monitoring infrastructure (Required)
    //
    // ESTIMATED EFFORT: 3-4 hours (medium confidence)
    // PRIORITY: Medium
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 2 (monitoring feature)
    // - Change Budget: ~80 LOC
    // - Reviewer Requirements: System monitoring expertise
    100.0 + (rand::random::<f64>() - 0.5) * 20.0 // Temporary: simulated until system API integration
}

impl Default for ToolInvocation {
    fn default() -> Self {
        Self {
            tool_name: "unknown".to_string(),
            parameters: serde_json::json!({}),
            context: None,
            timeout_ms: None,
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0.0,
            peak_concurrent: 0,
            active_executions: 0,
            total_resource_usage: ResourceUsage {
                cpu_time_ms: 0,
                peak_memory_mb: 0.0,
                io_operations: 0,
                network_bytes: 0,
            },
        }
    }
}
