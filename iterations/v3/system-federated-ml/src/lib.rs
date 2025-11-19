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
