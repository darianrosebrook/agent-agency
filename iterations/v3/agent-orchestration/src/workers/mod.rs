//! Worker Integration Module
//!
//! Bridges agent-orchestration with agent-workers for task execution.
//! Provides unified interface for worker execution while delegating to agent-workers crate.

pub mod execution_bridge;

pub use execution_bridge::WorkerExecutionBridge;



