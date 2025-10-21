//! Prompt Frame & Tool Schema
//!
//! This module provides the foundation for deterministic, reproducible prompt generation
//! and tool-call validation. It enables replay, cross-model consistency, and bandit learning.

pub mod frame;
pub mod tool_schema;

pub use frame::{PromptFrame, EvidenceBundle, Budgets};
pub use tool_schema::{PatchAction, ChangeKind, FileChange, ToolCallValidator, ToolSchemaError};

/// Simple adaptive prompting strategy stub
pub struct AdaptivePromptingStrategy;

impl AdaptivePromptingStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdaptivePromptingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompting strategy trait
pub trait PromptingStrategy {
    // Stub trait for now
}
