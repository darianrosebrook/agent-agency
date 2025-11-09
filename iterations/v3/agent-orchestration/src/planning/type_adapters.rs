//! Type adapters for migration from local types to contracts types
//!
//! This module provides conversion traits and implementations to gradually
//! migrate from local type definitions to shared types in agent-agency-contracts.
//! These adapters should be removed once all code has been updated to use contracts types.
//!
//! @author @darianrosebrook

use agent_agency_contracts::types::planning::{TaskScope as ContractsTaskScope, BlastRadius};
use agent_agency_contracts::planning_io::ChangeBudget as ContractsChangeBudget;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// Local types from orchestration
use crate::types::TaskScope;

// WorkingSpec and AcceptanceCriterion have been removed from crate::types
// Use agent_agency_contracts types directly instead

// Deprecated ChangeBudget for adapter conversions (use contracts directly)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

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
            enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict, // Default
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
// AcceptanceCriterion Conversions - REMOVED
// ============================================================================
// AcceptanceCriterion has been removed from crate::types
// Use agent_agency_contracts::AcceptanceCriterion directly

// ============================================================================
// WorkingSpec Conversions - REMOVED
// ============================================================================
// WorkingSpec has been removed from crate::types
// Use agent_agency_contracts::WorkingSpec directly

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

    // Tests for WorkingSpec and AcceptanceCriterion removed - types no longer exist
}
