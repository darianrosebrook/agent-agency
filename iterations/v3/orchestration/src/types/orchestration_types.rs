//! Core types for the Orchestration system
//!
//! This module contains all the core data structures used by the orchestration system
//! including configuration, task scopes, budgets, and execution results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task scope definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    /// Files and directories included in this task scope
    pub in_scope: Vec<String>,
    /// Files and directories explicitly excluded from this task scope
    pub out_scope: Vec<String>,
}

/// Change budget for orchestration constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBudget {
    /// Maximum number of files that can be changed
    pub max_files: u32,
    /// Maximum lines of code that can be changed
    pub max_loc: u32,
}

/// Blast radius for orchestration impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadius {
    /// Modules that will be affected by the orchestration
    pub modules: Vec<String>,
    /// Whether data migration is required
    pub data_migration: bool,
    /// External dependencies that will be affected
    pub external_deps: Vec<String>,
}

/// Memory-informed orchestration decision
#[derive(Debug, Clone)]
pub(crate) struct MemoryInformedDecision {
    /// Whether parallel execution is preferred based on historical success
    pub prefers_parallel: bool,
    /// Suggested worker IDs based on past performance
    pub suggested_workers: Vec<String>,
    /// Expected success rate for the preferred strategy
    pub expected_success_rate: f32,
    /// Confidence level in the decision (0.0 to 1.0)
    pub confidence: f32,
}

/// Result of task execution orchestration
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// The final working specification after orchestration
    pub working_spec: agent_agency_contracts::working_spec::WorkingSpec,
    /// Execution artifacts produced during orchestration
    pub artifacts: crate::planning::types::ExecutionArtifacts,
    /// Quality report from orchestration
    pub quality_report: Option<crate::quality::QualityReport>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Maximum time allowed for orchestration (in seconds)
    pub max_orchestration_time_seconds: u64,
    /// Whether to enable parallel execution
    pub enable_parallel_execution: bool,
    /// Whether to enable memory-informed decisions
    pub enable_memory_decisions: bool,
    /// Whether to enable ARM optimization
    pub enable_arm_optimization: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: agent_agency_resilience::CircuitBreakerConfig,
    /// Retry configuration
    pub retry_config: agent_agency_resilience::RetryConfig,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_orchestration_time_seconds: 300, // 5 minutes
            enable_parallel_execution: true,
            enable_memory_decisions: true,
            enable_arm_optimization: cfg!(target_arch = "aarch64"),
            circuit_breaker_config: agent_agency_resilience::CircuitBreakerConfig::default(),
            retry_config: agent_agency_resilience::RetryConfig::default(),
        }
    }
}

/// Apple Silicon Model Registry Configuration (stub)
#[derive(Debug, Clone)]
pub struct AppleModelRegistryConfig {
    /// Path to model registry
    pub registry_path: Option<std::path::PathBuf>,
    /// Performance tier preference
    pub preferred_tier: agent_agency_apple_silicon::Tier,
}

impl Default for AppleModelRegistryConfig {
    fn default() -> Self {
        Self {
            registry_path: None,
            preferred_tier: agent_agency_apple_silicon::Tier::Balanced,
        }
    }
}

/// Apple Silicon System Sensors (stub)
#[derive(Debug, Clone)]
pub struct SystemSensors;

impl SystemSensors {
    pub fn detect() -> Self {
        Self
    }
}

/// Apple Silicon Model Registry (stub)
#[derive(Debug, Clone)]
pub struct AppleModelRegistry;

impl AppleModelRegistry {
    pub fn from_path(_path: &std::path::Path) -> Option<Self> {
        Some(Self)
    }

    pub fn from_config(_config: AppleModelRegistryConfig) -> Self {
        Self
    }
}
