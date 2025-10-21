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

/// Telemetry collector for agent performance tracking
#[derive(Debug, Clone)]
pub struct AgentTelemetryCollector {
    agent_id: String,
    metrics: std::collections::HashMap<String, serde_json::Value>,
}

impl AgentTelemetryCollector {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            metrics: std::collections::HashMap::new(),
        }
    }

    pub fn record_metric(&mut self, key: &str, value: serde_json::Value) {
        self.metrics.insert(key.to_string(), value);
    }

    pub fn get_metric(&self, key: &str) -> Option<&serde_json::Value> {
        self.metrics.get(key)
    }
}
