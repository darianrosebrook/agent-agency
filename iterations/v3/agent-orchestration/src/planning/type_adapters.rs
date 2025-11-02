//! Type adapters for migration from local types to contracts types
//!
//! This module provides conversion traits and implementations to gradually
//! migrate from local type definitions to shared types in agent-agency-contracts.
//! These adapters should be removed once all code has been updated to use contracts types.
//!
//! @author @darianrosebrook

use agent_agency_contracts::types::planning::{TaskScope as ContractsTaskScope, BlastRadius};
use agent_agency_contracts::planning_io::ChangeBudget as ContractsChangeBudget;
use agent_agency_contracts::WorkingSpec as ContractsWorkingSpec;
use agent_agency_contracts::AcceptanceCriterion as ContractsAcceptanceCriterion;

// Local types from orchestration
use crate::types::{TaskScope, WorkingSpec, AcceptanceCriterion};
use crate::council_types::ChangeBudget;

/// Conversion trait from local types to contracts types
pub trait ToContracts<T> {
    fn to_contracts(&self) -> T;
}

/// Conversion trait from contracts types to local types
pub trait FromContracts<T> {
    fn from_contracts(contracts: T) -> Self;
}

// ============================================================================
// TaskScope Conversions
// ============================================================================

impl ToContracts<ContractsTaskScope> for TaskScope {
    fn to_contracts(&self) -> ContractsTaskScope {
        ContractsTaskScope {
            in_scope: self.in_scope.clone(),
            out_scope: self.out_scope.clone(),
        }
    }
}

impl FromContracts<ContractsTaskScope> for TaskScope {
    fn from_contracts(contracts: ContractsTaskScope) -> Self {
        TaskScope {
            in_scope: contracts.in_scope,
            out_scope: contracts.out_scope,
        }
    }
}

// ============================================================================
// BlastRadius Conversions
// ============================================================================

impl ToContracts<BlastRadius> for BlastRadius {
    fn to_contracts(&self) -> BlastRadius {
        BlastRadius {
            modules: self.modules.clone(),
            data_migration: self.data_migration,
            external_deps: self.external_deps.clone(),
        }
    }
}

impl FromContracts<BlastRadius> for BlastRadius {
    fn from_contracts(contracts: BlastRadius) -> Self {
        BlastRadius {
            modules: contracts.modules,
            data_migration: contracts.data_migration,
            external_deps: contracts.external_deps,
        }
    }
}

// ============================================================================
// ChangeBudget Conversions
// ============================================================================

impl ToContracts<ContractsChangeBudget> for ChangeBudget {
    fn to_contracts(&self) -> ContractsChangeBudget {
        ContractsChangeBudget {
            max_files: self.max_files as usize,
            max_loc: self.max_loc as usize,
            max_migrations: 0, // Local type doesn't have this field
            allow_breaking_changes: false, // Local type doesn't have this field
            allow_new_dependencies: false, // Local type doesn't have this field
            enforcement_mode: agent_agency_contracts::planning_io::EnforcementMode::Strict, // Default
        }
    }
}

impl FromContracts<ContractsChangeBudget> for ChangeBudget {
    fn from_contracts(contracts: ContractsChangeBudget) -> Self {
        ChangeBudget {
            max_files: contracts.max_files as u32,
            max_loc: contracts.max_loc as u32, // Convert usize to u32
        }
    }
}

// ============================================================================
// AcceptanceCriterion Conversions
// ============================================================================

impl ToContracts<ContractsAcceptanceCriterion> for AcceptanceCriterion {
    fn to_contracts(&self) -> ContractsAcceptanceCriterion {
        ContractsAcceptanceCriterion {
            id: self.id.clone(),
            given: self.given.clone(),
            when: self.when.clone(),
            then: self.then.clone(),
            priority: None, // Local type doesn't have priority field
        }
    }
}

impl FromContracts<ContractsAcceptanceCriterion> for AcceptanceCriterion {
    fn from_contracts(contracts: ContractsAcceptanceCriterion) -> Self {
        AcceptanceCriterion {
            id: contracts.id,
            given: contracts.given,
            when: contracts.when,
            then: contracts.then,
        }
    }
}

// ============================================================================
// WorkingSpec Conversions
// ============================================================================

impl ToContracts<ContractsWorkingSpec> for WorkingSpec {
    fn to_contracts(&self) -> ContractsWorkingSpec {
        // Convert local WorkingSpec to contracts WorkingSpec
        use agent_agency_contracts::working_spec::{
            WorkingSpecConstraints, WorkingSpecContext, WorkingSpecMetadata,
            BudgetLimits, ScopeRestrictions, TestPlan, RollbackPlan,
        };
        use chrono::Utc;

        // Convert acceptance criteria
        let acceptance_criteria = self.acceptance_criteria
            .iter()
            .map(|c| ContractsAcceptanceCriterion {
                id: c.id.clone(),
                given: c.given.clone(),
                when: c.when.clone(),
                then: c.then.clone(),
                priority: None, // Local type doesn't have priority
            })
            .collect();

        // Build constraints from local fields
        let constraints = WorkingSpecConstraints {
            max_duration_minutes: None,
            max_iterations: None,
            budget_limits: Some(BudgetLimits {
                max_files: Some(self.change_budget.max_files as u32),
                max_loc: Some(self.change_budget.max_loc as u32),
            }),
            scope_restrictions: Some(ScopeRestrictions {
                allowed_paths: self.scope.in_scope.clone(),
                blocked_paths: self.scope.out_scope.clone(),
            }),
        };

        // Build context
        let context = WorkingSpecContext {
            workspace_root: ".".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: std::collections::HashMap::new(),
            environment: agent_agency_contracts::task_request::Environment::Development,
        };

        // Build metadata
        let metadata = WorkingSpecMetadata {
            created_at: Utc::now(),
            created_by: None,
            last_modified: Some(Utc::now()),
            version: None,
            tags: vec![],
        };

        ContractsWorkingSpec {
            version: "1.0.0".to_string(),
            id: self.id.clone(),
            title: self.title.clone(),
            description: format!("Working specification for: {}", self.title),
            goals: vec![self.title.clone()],
            risk_tier: self.risk_tier as u32,
            constraints,
            acceptance_criteria,
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan {
                strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
                automated_steps: vec![],
                manual_steps: vec![],
                data_impact: agent_agency_contracts::working_spec::DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(5),
            },
            context,
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: Some(metadata),
            milestones: vec![],
            change_budget: ContractsChangeBudget {
                max_files: self.change_budget.max_files as usize,
                max_loc: self.change_budget.max_loc as usize,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::EnforcementMode::Strict,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl FromContracts<ContractsWorkingSpec> for WorkingSpec {
    fn from_contracts(contracts: ContractsWorkingSpec) -> Self {
        // Convert contracts WorkingSpec to local WorkingSpec
        let change_budget = if let Some(budget_limits) = &contracts.constraints.budget_limits {
            ChangeBudget {
                max_files: budget_limits.max_files.unwrap_or(25),
                max_loc: budget_limits.max_loc.unwrap_or(1000),
            }
        } else {
            ChangeBudget {
                max_files: contracts.change_budget.max_files as u32,
                max_loc: contracts.change_budget.max_loc as u32,
            }
        };

        let scope = if let Some(scope_restrictions) = &contracts.constraints.scope_restrictions {
            TaskScope {
                in_scope: scope_restrictions.allowed_paths.clone(),
                out_scope: scope_restrictions.blocked_paths.clone(),
            }
        } else if !contracts.scope.is_empty() {
            TaskScope {
                in_scope: contracts.scope[0].allowed_paths.clone(),
                out_scope: contracts.scope[0].blocked_paths.clone(),
            }
        } else {
            TaskScope {
                in_scope: vec![],
                out_scope: vec![],
            }
        };

        // Determine mode from description or default
        let mode = if contracts.description.contains("refactor") {
            "refactor".to_string()
        } else if contracts.description.contains("fix") {
            "fix".to_string()
        } else if contracts.description.contains("doc") {
            "doc".to_string()
        } else {
            "feature".to_string()
        };

        WorkingSpec {
            id: contracts.id,
            title: contracts.title,
            risk_tier: contracts.risk_tier as u8,
            mode,
            change_budget,
            blast_radius: BlastRadius {
                modules: vec![], // Contracts WorkingSpec doesn't store blast_radius directly
                data_migration: false,
                external_deps: vec![],
            },
            scope,
            acceptance_criteria: contracts.acceptance_criteria
                .into_iter()
                .map(|c| AcceptanceCriterion {
                    id: c.id,
                    given: c.given,
                    when: c.when,
                    then: c.then,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_scope_conversion() {
        let local = TaskScope {
            in_scope: vec!["src/".to_string()],
            out_scope: vec!["tests/".to_string()],
        };

        let contracts = local.to_contracts();
        let back = TaskScope::from_contracts(contracts);

        assert_eq!(local.in_scope, back.in_scope);
        assert_eq!(local.out_scope, back.out_scope);
    }

    #[test]
    fn test_acceptance_criterion_conversion() {
        let local = AcceptanceCriterion {
            id: "A1".to_string(),
            given: "User is logged in".to_string(),
            when: "User clicks button".to_string(),
            then: "Action is performed".to_string(),
        };

        let contracts = local.to_contracts();
        let back = AcceptanceCriterion::from_contracts(contracts);

        assert_eq!(local.id, back.id);
        assert_eq!(local.given, back.given);
        assert_eq!(local.when, back.when);
        assert_eq!(local.then, back.then);
    }

    #[test]
    fn test_change_budget_conversion() {
        let local = ChangeBudget {
            max_files: 25,
            max_loc: 1000,
        };

        let contracts = local.to_contracts();
        let back = ChangeBudget::from_contracts(contracts);

        assert_eq!(local.max_files, back.max_files);
        assert_eq!(local.max_loc, back.max_loc);
    }
}
