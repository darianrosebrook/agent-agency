//! Agent Workers - MCP-Based Task Execution System
//!
//! A unified worker orchestration system that executes tasks using MCP tools
//! rather than hardcoded implementations. Consolidates workers/, parallel-workers/,
//! and worker/ into a single, coherent MCP-based architecture.
//!
//! ## Architecture
//!
//! Workers discover and execute tools from the MCP registry:
//! - **Tool Discovery**: Automatic MCP tool discovery and registration
//! - **MCP Execution**: Tool-based execution instead of hardcoded logic
//! - **Quality Gates**: CAWS compliance and validation throughout
//!
//! ## Key Components
//!
//! - **MCPWorkerPool**: Main worker orchestration with MCP tool integration
//! - **ToolExecutor**: Executes MCP tools with proper error handling
//! - **MCPToolRegistry**: Registry for available MCP tools

#![allow(warnings)]
#![allow(dead_code)]

pub mod core;
pub mod execution;
pub mod mcp_integration;
pub mod types;

// Re-export main types
pub use core::{MCPWorkerPool, WorkerPoolConfig, WorkerHandle};
pub use execution::{ToolExecutor, ExecutionResult};
pub use mcp_integration::MCPToolRegistry;
pub use types::*;

// Factory functions
pub use core::create_worker_pool;

/// Create a new MCP-based worker pool with default configuration
pub fn new_worker_pool() -> MCPWorkerPool {
    MCPWorkerPool::new(WorkerPoolConfig::default())
}

/// Create a worker pool with custom MCP tool registry
pub fn new_worker_pool_with_tools(tools: MCPToolRegistry) -> MCPWorkerPool {
    MCPWorkerPool::with_tools(WorkerPoolConfig::default(), tools)
}
