//! Tool Executor
//!
//! Coordinates tool execution with validation and auditing.

use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use v4_types::operators::OperatorResult;
use v4_types::OperatorType;

use crate::registry::ToolRegistry;
use crate::tool::{ExecutionContext, Tool, ToolError};

/// Tool executor for coordinated tool execution
pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
    config: ExecutorConfig,
}

/// Executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Default timeout in milliseconds
    pub default_timeout_ms: u64,
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    /// Whether to require sandbox for write operations
    pub require_sandbox_for_writes: bool,
    /// Whether to hash all outputs
    pub hash_outputs: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_concurrent: 4,
            require_sandbox_for_writes: true,
            hash_outputs: true,
        }
    }
}

impl ToolExecutor {
    /// Create a new executor
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            config: ExecutorConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(registry: Arc<ToolRegistry>, config: ExecutorConfig) -> Self {
        Self { registry, config }
    }

    /// Execute an operator using the appropriate tool
    pub async fn execute(
        &self,
        operator: &OperatorType,
        context: &ExecutionContext,
    ) -> Result<ExecutionRecord, ExecutorError> {
        let start = Instant::now();
        let started_at = Utc::now();

        // Find a tool that can execute this operator
        let tool = self.find_tool_for(operator)?;

        // Validate the operator
        tool.validate(operator)?;

        // Check sandbox requirements
        if self.config.require_sandbox_for_writes && operator.has_side_effects() && !context.sandboxed {
            return Err(ExecutorError::SandboxRequired(
                "Write operations require sandbox mode".to_string(),
            ));
        }

        // Execute with timeout
        let timeout = std::time::Duration::from_millis(context.timeout_ms);
        let result = tokio::time::timeout(timeout, tool.execute(operator)).await;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(mut op_result)) => {
                // Hash output if configured
                if self.config.hash_outputs {
                    op_result.content_hash = Some(self.hash_result(&op_result));
                }
                op_result.execution_time_ms = elapsed_ms;

                Ok(ExecutionRecord {
                    tool_id: tool.id().to_string(),
                    operator: operator.clone(),
                    result: op_result,
                    context: context.clone(),
                    started_at,
                    completed_at: Utc::now(),
                    elapsed_ms,
                })
            }
            Ok(Err(e)) => Err(ExecutorError::ToolError(e)),
            Err(_) => Err(ExecutorError::Timeout(context.timeout_ms)),
        }
    }

    /// Find a tool that can execute the given operator
    fn find_tool_for(&self, operator: &OperatorType) -> Result<Arc<dyn Tool>, ExecutorError> {
        let operator_class = operator.class();
        let tools = self.registry.find_for_operator(operator_class);

        if tools.is_empty() {
            return Err(ExecutorError::NoToolAvailable(format!(
                "No tool available for operator class {}",
                operator_class
            )));
        }

        // Find a tool that can actually execute this specific operator
        for tool_meta in &tools {
            if let Some(tool) = self.registry.get(&tool_meta.id) {
                if tool.can_execute(operator) {
                    return Ok(tool);
                }
            }
        }

        Err(ExecutorError::NoToolAvailable(format!(
            "No tool can execute operator {:?}",
            operator
        )))
    }

    /// Hash a result for audit trail
    fn hash_result(&self, result: &OperatorResult) -> String {
        let mut hasher = Sha256::new();
        if let Some(ref data) = result.data {
            hasher.update(data.to_string().as_bytes());
        }
        if let Some(ref error) = result.error {
            hasher.update(error.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    /// Execute multiple operators in sequence
    pub async fn execute_sequence(
        &self,
        operators: &[OperatorType],
        context: &ExecutionContext,
    ) -> Vec<Result<ExecutionRecord, ExecutorError>> {
        let mut results = Vec::with_capacity(operators.len());

        for operator in operators {
            let result = self.execute(operator, context).await;
            let should_continue = result.is_ok();
            results.push(result);

            if !should_continue {
                break;
            }
        }

        results
    }
}

/// Execution error
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("No tool available: {0}")]
    NoToolAvailable(String),

    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Sandbox required: {0}")]
    SandboxRequired(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Record of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Tool that executed
    pub tool_id: String,
    /// Operator that was executed
    pub operator: OperatorType,
    /// Execution result
    pub result: OperatorResult,
    /// Execution context
    pub context: ExecutionContext,
    /// When execution started
    pub started_at: DateTime<Utc>,
    /// When execution completed
    pub completed_at: DateTime<Utc>,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u64,
}

impl ExecutionRecord {
    /// Check if execution succeeded
    pub fn succeeded(&self) -> bool {
        self.result.success
    }

    /// Get content hash for audit
    pub fn content_hash(&self) -> Option<&str> {
        self.result.content_hash.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{ToolCapability, ToolCategory, ToolMetadata};
    use async_trait::async_trait;
    use v4_types::operators::SeekOp;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn id(&self) -> &str {
            "test-tool"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                id: "test-tool".to_string(),
                name: "Test Tool".to_string(),
                description: "A test tool".to_string(),
                version: "1.0.0".to_string(),
                category: ToolCategory::FileSystem,
                capabilities: vec![ToolCapability::ReadOnly],
                supported_operators: vec!["S".to_string()],
                requires_sandbox: false,
            }
        }

        async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
            Ok(OperatorResult {
                operator: operator.clone(),
                success: true,
                data: Some(serde_json::json!({"content": "test data"})),
                error: None,
                execution_time_ms: 0,
                content_hash: None,
            })
        }

        fn can_execute(&self, operator: &OperatorType) -> bool {
            matches!(operator, OperatorType::Seek(_))
        }
    }

    #[tokio::test]
    async fn test_executor_execute() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(TestTool));

        let executor = ToolExecutor::new(registry);
        let operator = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        });
        let context = ExecutionContext::new("test-task");

        let record = executor.execute(&operator, &context).await.unwrap();

        assert!(record.succeeded());
        assert_eq!(record.tool_id, "test-tool");
        assert!(record.content_hash().is_some());
    }

    #[tokio::test]
    async fn test_executor_no_tool() {
        let registry = Arc::new(ToolRegistry::new());
        let executor = ToolExecutor::new(registry);

        let operator = OperatorType::Seek(SeekOp::ReadFile {
            path: "test.txt".to_string(),
        });
        let context = ExecutionContext::new("test-task");

        let result = executor.execute(&operator, &context).await;
        assert!(matches!(result, Err(ExecutorError::NoToolAvailable(_))));
    }

    #[tokio::test]
    async fn test_executor_sequence() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(TestTool));

        let executor = ToolExecutor::new(registry);
        let operators = vec![
            OperatorType::Seek(SeekOp::ReadFile {
                path: "a.txt".to_string(),
            }),
            OperatorType::Seek(SeekOp::ReadFile {
                path: "b.txt".to_string(),
            }),
        ];
        let context = ExecutionContext::new("test-task");

        let results = executor.execute_sequence(&operators, &context).await;

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
