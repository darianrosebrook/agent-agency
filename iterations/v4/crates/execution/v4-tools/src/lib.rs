//! V4 Tools
//!
//! Tool registry and execution framework for Agent Agency V4.
//!
//! ## Overview
//!
//! This crate provides:
//! - **Tool trait**: Interface for implementing tools
//! - **ToolRegistry**: Central registry for tool discovery
//! - **ToolExecutor**: Coordinated tool execution with auditing
//! - **Built-in tools**: Standard tools for file, memory, and search operations
//!
//! ## Architecture
//!
//! ```text
//! Operator (from v4-types)
//!     │
//!     ▼
//! ToolExecutor
//!     │
//!     ├──► ToolRegistry ──► Find matching tool
//!     │
//!     ▼
//! Tool.execute()
//!     │
//!     ▼
//! ExecutionRecord (with audit hash)
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_tools::{ToolRegistry, ToolExecutor, ExecutionContext};
//! use v4_tools::builtin::register_builtin_tools;
//! use v4_types::operators::SeekOp;
//! use v4_types::OperatorType;
//!
//! // Create registry and register tools
//! let registry = Arc::new(ToolRegistry::new());
//! register_builtin_tools(&registry);
//!
//! // Create executor
//! let executor = ToolExecutor::new(registry);
//!
//! // Execute an operator
//! let operator = OperatorType::Seek(SeekOp::ReadFile {
//!     path: "src/lib.rs".to_string(),
//! });
//! let context = ExecutionContext::new("task-1");
//!
//! let record = executor.execute(&operator, &context).await?;
//! println!("Success: {}, Hash: {:?}", record.succeeded(), record.content_hash());
//! ```

pub mod builtin;
pub mod executor;
pub mod registry;
pub mod tool;

// Re-export main types
pub use builtin::{
    register_builtin_tools, CodeSearchTool, DirectoryListTool, FileEditTool, FilePatchTool,
    FileReadTool, FileWriteTool, MemoryQueryTool, ShellExecTool, ShellSandboxProfile,
    TestRunnerTool,
};
pub use executor::{ExecutionRecord, ExecutorConfig, ExecutorError, ToolExecutor};
pub use registry::{RegistrySnapshot, ToolRegistry};
pub use tool::{
    ExecutionContext, Tool, ToolCapability, ToolCategory, ToolError, ToolId, ToolMetadata,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use v4_types::operators::SeekOp;
    use v4_types::OperatorType;

    #[test]
    fn test_registry_with_builtin_tools() {
        let registry = ToolRegistry::new();
        register_builtin_tools(&registry);

        assert_eq!(registry.count(), 9);

        // Find tools by category
        let fs_tools = registry.find_by_category(ToolCategory::FileSystem);
        assert_eq!(fs_tools.len(), 5); // FileRead, FileWrite, FileEdit, DirList, FilePatch

        // Find tools by capability
        let readonly_tools = registry.find_by_capability(ToolCapability::ReadOnly);
        assert_eq!(readonly_tools.len(), 4); // FileRead, DirList, CodeSearch, MemoryQuery
    }

    #[tokio::test]
    async fn test_executor_with_builtin_tools() {
        let registry = Arc::new(ToolRegistry::new());
        register_builtin_tools(&registry);

        let executor = ToolExecutor::new(registry);

        // Test with a search operation (doesn't require actual files)
        let operator = OperatorType::Seek(SeekOp::SearchCode {
            pattern: "test".to_string(),
            path: Some(".".to_string()),
        });
        let context = ExecutionContext::new("test-task");

        let record = executor.execute(&operator, &context).await.unwrap();
        assert!(record.succeeded());
    }

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::new("task-123")
            .with_working_dir("/tmp")
            .with_timeout(5000)
            .with_sandbox()
            .with_blocked_paths(vec!["/etc".to_string()]);

        assert_eq!(ctx.task_id, "task-123");
        assert!(ctx.sandboxed);
        assert!(ctx.is_path_blocked("/etc/passwd"));
    }

    #[test]
    fn test_tool_metadata_serialization() {
        let registry = ToolRegistry::new();
        register_builtin_tools(&registry);

        let snapshot = registry.snapshot();
        let json = serde_json::to_string(&snapshot).unwrap();

        assert!(json.contains("builtin:file-read"));
        assert!(json.contains("builtin:dir-list"));
    }

    #[test]
    fn test_find_for_operator() {
        let registry = ToolRegistry::new();
        register_builtin_tools(&registry);

        // Seek operators should find tools
        let seek_tools = registry.find_for_operator("S");
        assert!(!seek_tools.is_empty());

        // Memorize operators should find write/edit/patch tools
        let memorize_tools = registry.find_for_operator("M");
        assert_eq!(memorize_tools.len(), 3); // FileWrite, FileEdit, FilePatch

        // Control operators should find shell-exec and test-runner
        let control_tools = registry.find_for_operator("C");
        assert_eq!(control_tools.len(), 2);
    }
}
