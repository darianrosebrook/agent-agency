//! Tool Trait and Types
//!
//! Core abstractions for tools that execute operators.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use v4_types::operators::OperatorResult;
use v4_types::OperatorType;

/// Tool identifier
pub type ToolId = String;

/// Tool capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCapability {
    /// Tool only reads data
    ReadOnly,
    /// Tool can write data
    WriteData,
    /// Tool can execute code
    ExecuteCode,
    /// Tool accesses network
    NetworkAccess,
    /// Tool accesses filesystem
    FileSystemAccess,
    /// Tool requires elevated privileges
    ElevatedPrivileges,
}

/// Tool category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// File system operations
    FileSystem,
    /// Code search and analysis
    CodeAnalysis,
    /// Memory and knowledge operations
    Memory,
    /// Web and network operations
    Web,
    /// Git operations
    Git,
    /// Control flow operations
    Control,
    /// Schema and validation operations
    Validation,
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Unique identifier
    pub id: ToolId,
    /// Human-readable name
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// Version string
    pub version: String,
    /// Category
    pub category: ToolCategory,
    /// Capabilities
    pub capabilities: Vec<ToolCapability>,
    /// Supported operator classes (S, M, P, K, C)
    pub supported_operators: Vec<String>,
    /// Whether this tool requires sandbox execution
    pub requires_sandbox: bool,
}

/// Tool error
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid operator for tool: {0}")]
    InvalidOperator(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::IoError(err.to_string())
    }
}

/// Tool trait - all tools must implement this
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool ID
    fn id(&self) -> &str;

    /// Get tool metadata
    fn metadata(&self) -> ToolMetadata;

    /// Execute an operator
    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError>;

    /// Check if this tool can execute a given operator
    fn can_execute(&self, operator: &OperatorType) -> bool;

    /// Validate operator parameters before execution
    fn validate(&self, operator: &OperatorType) -> Result<(), ToolError> {
        if !self.can_execute(operator) {
            return Err(ToolError::InvalidOperator(format!(
                "Tool {} cannot execute {:?}",
                self.id(),
                operator
            )));
        }
        Ok(())
    }
}

/// Execution context passed to tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Task ID
    pub task_id: String,
    /// Working directory
    pub working_dir: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether running in sandbox
    pub sandboxed: bool,
    /// Blocked paths (for sandbox)
    pub blocked_paths: Vec<String>,
    /// Maximum output size in bytes
    pub max_output_bytes: usize,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            task_id: String::new(),
            working_dir: ".".to_string(),
            timeout_ms: 30000,
            sandboxed: false,
            blocked_paths: Vec::new(),
            max_output_bytes: 1024 * 1024, // 1MB
        }
    }
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            ..Default::default()
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = dir.into();
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Enable sandbox mode
    pub fn with_sandbox(mut self) -> Self {
        self.sandboxed = true;
        self
    }

    /// Add blocked paths
    pub fn with_blocked_paths(mut self, paths: Vec<String>) -> Self {
        self.blocked_paths = paths;
        self
    }

    /// Check if a path is blocked
    pub fn is_path_blocked(&self, path: &str) -> bool {
        self.blocked_paths.iter().any(|blocked| {
            path.starts_with(blocked) || path.contains(blocked)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_default() {
        let ctx = ExecutionContext::default();
        assert_eq!(ctx.working_dir, ".");
        assert_eq!(ctx.timeout_ms, 30000);
        assert!(!ctx.sandboxed);
    }

    #[test]
    fn test_execution_context_builder() {
        let ctx = ExecutionContext::new("task-1")
            .with_working_dir("/tmp")
            .with_timeout(5000)
            .with_sandbox()
            .with_blocked_paths(vec!["/etc".to_string(), "/root".to_string()]);

        assert_eq!(ctx.task_id, "task-1");
        assert_eq!(ctx.working_dir, "/tmp");
        assert_eq!(ctx.timeout_ms, 5000);
        assert!(ctx.sandboxed);
        assert!(ctx.is_path_blocked("/etc/passwd"));
        assert!(ctx.is_path_blocked("/root/.ssh"));
        assert!(!ctx.is_path_blocked("/tmp/file"));
    }

    #[test]
    fn test_tool_capability() {
        let caps = vec![ToolCapability::ReadOnly, ToolCapability::FileSystemAccess];
        assert!(caps.contains(&ToolCapability::ReadOnly));
        assert!(!caps.contains(&ToolCapability::WriteData));
    }
}
