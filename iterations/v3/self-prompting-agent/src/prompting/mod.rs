//! Prompt Frame & Tool Schema
//!
//! This module provides the foundation for deterministic, reproducible prompt generation
//! and tool-call validation. It enables replay, cross-model consistency, and bandit learning.

pub mod frame;
pub mod tool_schema;

pub use frame::{PromptFrame, EvidenceBundle, Budgets};
pub use tool_schema::{PatchAction, ChangeKind, FileChange, ToolCallValidator};
