//! Orchestration module for Agent Agency V3
//!
//! This module has been decomposed from a monolithic 1756 LOC file into focused modules:
//! - `types/`: Core types and configuration
//! - `worker_registry/`: Worker management and registry
//! - `orchestrator_core/`: Main Orchestrator implementation
//! - `orchestrate_functions.rs`: Legacy utility functions

use std::collections::HashMap;
use uuid::Uuid;

use crate::adapter::build_short_circuit_verdict;
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
// NEW: Runtime-validator integration
use caws_runtime_validator::integration::{
    OrchestrationIntegration, DefaultOrchestrationIntegration,
    OrchestrationValidationResult, ExecutionDecision, ExecutionMode,
    Violation as RuntimeViolation, ViolationCode as RuntimeViolationCode,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::planning::types::{ExecutionArtifacts, TestResults, CoverageReport, MutationReport, LintReport, TypeCheckReport, ProvenanceRecord};
use parallel_workers::types::{Artifact, ArtifactType, ExecutionMetrics};
use crate::planning::agent::{CriterionPriority, RollbackRisk};
use crate::tracking::ProgressTracker;
use crate::types::{TaskScope, ChangeBudget, BlastRadius, OrchestratorConfig, TaskExecutionResult};
use crate::worker_registry::{WorkerRegistry, StaticWorkerRegistry};
use crate::orchestrator_core::Orchestrator;
use agent_agency_apple_silicon::{
    AllocationPlanner, AllocationRequest, AllocationPlan, DeviceKind, DeviceSensors,
};

use agent_agency_contracts::working_spec::{
    WorkingSpecMetadata, AcceptanceCriterion, NonFunctionalRequirements, RollbackPlan,
};
use agent_agency_council::{ConsensusCoordinator, ProvenanceEmitter};
use agent_agency_council::models::{
    AcceptanceCriterion as CouncilAcceptanceCriterion, Environment as CouncilEnvironment,
    RiskTier as CouncilRiskTier, SelfAssessment as CouncilSelfAssessment,
    TaskContext as CouncilTaskContext, TaskScope as CouncilTaskScope, TaskSpec as CouncilTaskSpec,
    WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_council::types::{CawsWaiver, ConsensusResult, FinalVerdict};
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig, retry, RetryConfig};
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

// Re-export main components
pub use types::*;
pub use worker_registry::*;
pub use orchestrator_core::*;

// Include legacy functions
include!("orchestrate_functions.rs");

// Helper functions that remain in the main module

fn map_risk_tier(tier: u8) -> CouncilRiskTier {
    match tier {
        1 => CouncilRiskTier::Low,
        2 => CouncilRiskTier::Medium,
        3 => CouncilRiskTier::High,
        _ => CouncilRiskTier::High,
    }
}

pub fn to_task_spec(desc: &TaskDescriptor) -> CouncilTaskSpec {
    CouncilTaskSpec {
        id: uuid::Uuid::new_v4(),
        title: format!("task-{}", desc.task_id),
        description: "Orchestrated task".to_string(),
        risk_tier: map_risk_tier(desc.risk_tier),
        scope: CouncilTaskScope {
            files_affected: desc.scope_in.clone(),
            max_files: None,
            max_loc: None,
            domains: vec![],
        },
        acceptance_criteria: desc
            .acceptance
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|criterion| CouncilAcceptanceCriterion {
                id: uuid::Uuid::new_v4().to_string(),
                description: criterion,
            })
            .collect(),
        context: CouncilTaskContext {
            workspace_root: ".".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: HashMap::new(),
            environment: CouncilEnvironment::Development,
        },
        worker_output: CouncilWorkerOutput {
            content: String::new(),
            files_modified: vec![],
            rationale: String::new(),
            self_assessment: CouncilSelfAssessment {
                caws_compliance: 0.0,
                quality_score: 0.0,
                confidence: 0.0,
                concerns: vec![],
                improvements: vec![],
                estimated_effort: None,
            },
            metadata: HashMap::new(),
        },
        caws_spec: None,
    }
}

fn record_arm_plan(desc: &TaskDescriptor) {
    let tier = match desc.risk_tier {
        1 => agent_agency_apple_silicon::Tier::HighEfficiency,
        2 => agent_agency_apple_silicon::Tier::Balanced,
        _ => agent_agency_apple_silicon::Tier::HighPerformance,
    };

    // ARM planning logic would go here
    let _tier = tier; // Suppress unused variable warning
}