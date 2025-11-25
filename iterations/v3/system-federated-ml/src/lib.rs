//! Complete Tool Calling Ecosystem - MCP Integration with CAWS Tooling
//!
//! Implements comprehensive tooling ecosystem for reasoning, conflict resolution,
//! and evidence collection through MCP-based CAWS tool discovery and execution.
//!
//! ## Tool Categories
//!
//! 1. **Policy Enforcement Tools**: CAWS validation, waiver auditing, budget verification
//! 2. **Evidence Collection Tools**: Claim extraction, fact verification, source validation
//! 3. **Governance Tools**: Audit logging, provenance tracking, compliance reporting
//! 4. **Quality Gate Tools**: Code analysis, test execution, performance validation
//! 5. **Conflict Resolution Tools**: Debate orchestration, consensus building, evidence synthesis
//! 6. **Workflow Tools**: Task decomposition, progress tracking, resource allocation
//!
//! ## Federated ML Components
//!
//! - **Aggregation**: Secure multi-party computation for model updates
//! - **Coordinator**: Federation round management and participant orchestration
//! - **Encryption**: Homomorphic encryption (Paillier) for privacy-preserving computation
//! - **Bandit Policy**: Contextual bandit algorithms for parameter optimization

// Core tool ecosystem modules
pub mod arbiter_pipeline;
pub mod claim_extraction;
pub mod conflict_resolution_tools;
pub mod evidence_collection_tools;
pub mod evidence_types;
pub mod executor;
pub mod fact_verification;
pub mod multi_modal_verification;
pub mod parallel_integration;
pub mod policy_enforcement;
pub mod protocol;
pub mod schema_registry;
pub mod source_validation;
pub mod tool_chain_planner;
pub mod tool_coordinator;
pub mod tool_discovery;
pub mod tool_execution;
pub mod tool_registry;
pub mod validation;
pub mod model_updates;
pub mod security;
pub mod encryption;

// Federated ML modules
pub mod aggregation;
pub mod coordinator;
pub mod differential_privacy;
pub mod participant;

// LLM Parameter Optimization modules
pub mod bandit_policy;
pub mod bandit_stubs;
pub mod bayesian_optimizer;
pub mod counterfactual_log;
pub mod parameter_optimizer;
pub mod parameter_dashboard;
pub mod performance_monitor;
pub mod reward;
pub mod rollout;

// CAWS Integration modules
pub mod caws_integration {
    //! CAWS Integration - re-exports from runtime_caws_integration
    pub use crate::runtime_caws_integration::*;
}
pub mod runtime_caws_integration;
pub mod planning_agent_integration;
pub mod quality_gate_validator;
pub mod quality_guardrails;

// Advanced optimization modules
#[cfg(feature = "chunked_execution")]
pub mod chunked_execution;
pub mod chunked_executor;
#[cfg(feature = "precision_engineering")]
pub mod precision_engineering;
// streaming_pipeline requires common_pipeline crate - disabled until dependency is added
// pub mod streaming_pipeline;
#[cfg(feature = "thermal_scheduler")]
pub mod thermal_scheduler;
pub mod tool_bandits;

// Example and tuning modules
pub mod kokoro_tuning;
pub mod llm_parameter_feedback_example;

pub use arbiter_pipeline::{
    ArbiterPipelineOptimizer, DecisionInput, DecisionPipelineConfig, DecisionResult,
};
pub use conflict_resolution_tools::{ConflictResolutionTool, ConsensusBuilder, DebateOrchestrator};
pub use evidence_collection_tools::EvidenceCollectionTool; // FactVerificationTool, SourceValidationTool - not implemented yet
pub use executor::{ChainExecutor, ExecutionResult};
pub use multi_modal_verification::MultimodalVerificationTool;
pub use parallel_integration::ParallelToolCoordinator;
pub use policy_enforcement::PolicyEnforcementTools;
// pub use governance_tools::{GovernanceTool, AuditLogger, ProvenanceTracker}; // Module not implemented yet
// pub use quality_gate_tools::{QualityGateTool, CodeAnalysisTool, PerformanceValidator}; // Module not implemented yet
// pub use reasoning_tools::{ReasoningTool, LogicValidator, InferenceEngine}; // Module not implemented yet

// Stub implementations for missing tool types are handled by PolicyEnforcementTools
pub use tool_chain_planner::{
    ChainResult, PlanningConstraints, PlanningContext, ToolChain as TypedToolChain,
    ToolChainPlanner,
};
pub use tool_coordinator::{ToolChain, ToolCoordinator, ToolExecutionResult};
pub use tool_discovery::{ToolCapability, ToolDiscoveryEngine}; // ToolMetadata - private
pub use tool_execution::{ToolExecutor, ToolInvocation, ToolResult};
pub use tool_registry::{RegisteredTool, ToolRegistration, ToolRegistry};
// pub use workflow_tools::{WorkflowTool, TaskDecomposer, ProgressTracker}; // Module not implemented yet
// pub use crate::tool_orchestrator::ToolOrchestrator; // Module not implemented yet

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Policy validation result
#[derive(Debug, Clone, JsonSchema)]
pub enum PolicyValidationResult {
    /// Task is allowed
    Allowed,
    /// Task requires waiver
    RequiresWaiver(String),
    /// Task is blocked by policy
    Blocked(String),
}

/// Compliance status for optimization results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceStatus {
    /// CAWS compliance score (0.0-1.0)
    pub caws_compliance: f64,
    /// Quality threshold compliance (0.0-1.0)
    pub quality_threshold: f64,
    /// Trade-off score between performance and quality (0.0-1.0)
    pub trade_off_score: f64,
    /// Last compliance check timestamp
    #[schemars(with = "String")]
    pub last_checked: chrono::DateTime<chrono::Utc>,
}
