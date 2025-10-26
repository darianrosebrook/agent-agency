//! Types and helper functions for orchestration functions
//!
//! This module contains types and utility functions used throughout
//! the orchestration process, extracted from the monolithic orchestrate_functions.rs.

use serde::{Deserialize, Serialize};
use agent_agency_apple_silicon::{
    AllocationPlanner, AllocationRequest, AllocationPlan, DeviceKind, DeviceSensors,
};
use std::collections::HashMap;

/// Stub for SimplePlanner - temporary implementation
#[derive(Debug, Clone)]
pub struct SimplePlanner;

impl SimplePlanner {
    pub fn new(_sensors: SystemSensors, _registry: AppleModelRegistry) -> Self {
        Self
    }
}

/// System sensors for allocation planning
pub type SystemSensors = DeviceSensors;

/// Apple model registry for allocation planning
pub type AppleModelRegistry = HashMap<String, DeviceKind>;

/// Map internal risk tier to council risk tier
pub fn map_risk_tier(tier: u8) -> agent_agency_council::models::RiskTier {
    use agent_agency_council::models::RiskTier;
    match tier {
        1 => RiskTier::Low,
        2 => RiskTier::Medium,
        3 => RiskTier::High,
        _ => RiskTier::High,
    }
}

/// Convert TaskDescriptor to Council TaskSpec
pub fn to_task_spec(desc: &crate::caws_runtime::TaskDescriptor) -> agent_agency_council::models::TaskSpec {
    use agent_agency_council::models::{TaskSpec, TaskScope, Environment};

    TaskSpec {
        id: uuid::Uuid::new_v4(),
        title: desc.description.clone(),
        description: desc.description.clone(),
        risk_tier: map_risk_tier(desc.risk_tier.unwrap_or(2)),
        scope: TaskScope {
            files_affected: desc.scope_in.clone(),
            max_files: desc.budget.as_ref().map(|b| b.max_files as u32),
            max_loc: desc.budget.as_ref().map(|b| b.max_loc as u32),
            domains: desc.domains.clone(),
        },
        acceptance_criteria: desc.acceptance_criteria.iter().map(|ac| {
            agent_agency_council::models::AcceptanceCriterion {
                description: ac.description.clone(),
                priority: match ac.priority {
                    1 => agent_agency_contracts::working_spec::CriterionPriority::Critical,
                    2 => agent_agency_contracts::working_spec::CriterionPriority::High,
                    3 => agent_agency_contracts::working_spec::CriterionPriority::Medium,
                    _ => agent_agency_contracts::working_spec::CriterionPriority::Low,
                },
            }
        }).collect(),
        context: Some(agent_agency_council::models::TaskContext {
            workspace_root: "/tmp".to_string(), // TODO: Get from actual workspace
            git_branch: "main".to_string(), // TODO: Get from actual git context
            recent_changes: vec![], // TODO: Get from actual git history
            dependencies: HashMap::new(), // TODO: Get from actual dependencies
            environment: Environment {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
                node_version: None,
                python_version: None,
            },
        }),
    }
}

/// Record ARM allocation plan for provenance
pub fn record_arm_plan(desc: &crate::caws_runtime::TaskDescriptor) {
    // TODO: Implement ARM plan recording for provenance
    tracing::debug!("Recording ARM allocation plan for task {}", desc.task_id);
}
