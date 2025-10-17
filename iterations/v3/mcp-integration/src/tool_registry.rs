//! Tool Registry
//!
//! Manages registration, execution, and lifecycle of MCP tools.

use crate::types::*;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Tool registry for managing MCP tools
#[derive(Debug)]
pub struct ToolRegistry {
    registered_tools: Arc<DashMap<Uuid, MCPTool>>,
    execution_queue: Arc<RwLock<Vec<ToolExecutionRequest>>>,
    execution_history: Arc<RwLock<Vec<ToolExecutionResult>>>,
    statistics: Arc<RwLock<ToolRegistryStats>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            registered_tools: Arc::new(DashMap::new()),
            execution_queue: Arc::new(RwLock::new(Vec::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(ToolRegistryStats {
                total_tools: 0,
                active_tools: 0,
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_execution_time_ms: 0.0,
                most_used_tools: Vec::new(),
                last_updated: chrono::Utc::now(),
            })),
        }
    }

    /// Initialize tool registry
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing tool registry");
        // Reset statistics and ensure clean queues
        {
            let mut q = self.execution_queue.write().await;
            q.clear();
        }
        {
            let mut h = self.execution_history.write().await;
            h.clear();
        }
        {
            let mut stats = self.statistics.write().await;
            *stats = ToolRegistryStats {
                total_tools: self.registered_tools.len() as u64,
                active_tools: self.registered_tools.len() as u64,
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_execution_time_ms: 0.0,
                most_used_tools: Vec::new(),
                last_updated: chrono::Utc::now(),
            };
        }
        Ok(())
    }

    /// Register a new tool
    pub async fn register_tool(&self, tool: MCPTool) -> Result<()> {
        info!(
            tool_id = %tool.id,
            tool_name = %tool.name,
            version = %tool.version,
            tool_type = ?tool.tool_type,
            "Registering tool"
        );

        self.registered_tools.insert(tool.id, tool.clone());

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_tools += 1;
            stats.active_tools += 1;
            stats.last_updated = chrono::Utc::now();
        }

        info!(
            tool_id = %tool.id,
            tool_name = %tool.name,
            "Tool registered successfully"
        );
        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister_tool(&self, tool_id: Uuid) -> Result<()> {
        info!("Unregistering tool: {}", tool_id);

        if self.registered_tools.remove(&tool_id).is_some() {
            // Update statistics
            {
                let mut stats = self.statistics.write().await;
                stats.active_tools = stats.active_tools.saturating_sub(1);
                stats.last_updated = chrono::Utc::now();
            }

            info!("Tool unregistered successfully: {}", tool_id);
        } else {
            warn!("Tool not found for unregistration: {}", tool_id);
        }

        Ok(())
    }

    /// Get a registered tool
    pub async fn get_tool(&self, tool_id: Uuid) -> Option<MCPTool> {
        self.registered_tools
            .get(&tool_id)
            .map(|entry| entry.clone())
    }

    /// Get all registered tools
    pub async fn get_all_tools(&self) -> Vec<MCPTool> {
        self.registered_tools
            .iter()
            .map(|entry| entry.clone())
            .collect()
    }

    /// Execute a tool
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult> {
        info!(
            "Executing tool: {} (request: {})",
            request.tool_id, request.id
        );

        let start_time = std::time::Instant::now();
        let started_at = chrono::Utc::now();

        // Get tool
        let tool = self
            .registered_tools
            .get(&request.tool_id)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", request.tool_id))?;

        // Simulated execution router: respect timeout and return structured result
        let timeout = request.timeout_seconds.unwrap_or(30);
        let simulated = async {
            // placeholder for execution; sleep a tiny amount to simulate work
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            Ok::<serde_json::Value, anyhow::Error>(serde_json::json!({
                "tool": tool.name,
                "version": tool.version,
                "echo": request.parameters,
            }))
        };
        let output =
            tokio::time::timeout(std::time::Duration::from_secs(timeout as u64), simulated).await;

        let completed_at = chrono::Utc::now();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        let mut result = ToolExecutionResult {
            request_id: request.id,
            tool_id: request.tool_id,
            status: ExecutionStatus::Completed,
            output: None,
            error: None,
            logs: vec![LogEntry {
                timestamp: completed_at,
                level: LogLevel::Info,
                message: "Tool execution completed".to_string(),
                source: Some("tool_registry".to_string()),
                metadata: std::collections::HashMap::new(),
            }],
            performance_metrics: PerformanceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0,
                disk_io_bytes: 0,
                network_io_bytes: 0,
                execution_time_ms: duration_ms,
                queue_time_ms: 0,
            },
            caws_compliance_result: None,
            started_at,
            completed_at: Some(completed_at),
            duration_ms: Some(duration_ms),
        };

        match output {
            Ok(Ok(val)) => {
                result.output = Some(val);
                result.status = ExecutionStatus::Completed;
            }
            Ok(Err(e)) => {
                result.error = Some(format!("execution error: {e}"));
                result.status = ExecutionStatus::Failed;
            }
            Err(_) => {
                result.error = Some("execution timed out".into());
                result.status = ExecutionStatus::Timeout;
            }
        }

        // Store execution result
        {
            let mut history = self.execution_history.write().await;
            history.push(result.clone());

            // Keep only last 1000 executions
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_executions += 1;
            match result.status {
                ExecutionStatus::Completed => {
                    stats.successful_executions += 1;
                    // Only include successful executions in average time calculation
                    if stats.successful_executions == 1 {
                        stats.average_execution_time_ms = duration_ms as f64;
                    } else {
                        stats.average_execution_time_ms = (stats.average_execution_time_ms
                            * (stats.successful_executions - 1) as f64
                            + duration_ms as f64)
                            / stats.successful_executions as f64;
                    }
                }
                ExecutionStatus::Failed | ExecutionStatus::Timeout => {
                    stats.failed_executions += 1;
                    // Failed/timeout executions are not included in average execution time
                }
                _ => {}
            }
            stats.last_updated = chrono::Utc::now();
        }

        info!(
            "Tool execution completed: {} in {}ms",
            request.tool_id, duration_ms
        );
        Ok(result)
    }

    /// Update tool usage statistics
    pub async fn update_tool_usage(&self, tool_id: Uuid) -> Result<()> {
        if let Some(mut tool) = self.registered_tools.get_mut(&tool_id) {
            tool.usage_count += 1;
            tool.last_updated = chrono::Utc::now();
        }

        Ok(())
    }

    /// Get execution history
    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<ToolExecutionResult> {
        let history = self.execution_history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> ToolRegistryStats {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    /// Shutdown tool registry
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down tool registry");
        // Clean queues/history; idempotent
        self.execution_queue.write().await.clear();
        self.execution_history.write().await.clear();
        Ok(())
    }
}
