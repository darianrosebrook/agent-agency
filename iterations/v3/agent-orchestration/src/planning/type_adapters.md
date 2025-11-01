# Type Adapters Documentation

This module provides conversion traits and implementations for migrating between local orchestration types and contracts types.

## Purpose

During the migration from local type definitions to shared types in `agent-agency-contracts`, these adapters provide a bridge that allows gradual migration without breaking existing code.

## Available Conversions

### TaskScope
- **Local**: `crate::types::TaskScope`
- **Contracts**: `agent_agency_contracts::types::planning::TaskScope`
- **Usage**: Convert task scope definitions between local and contracts format

### BlastRadius
- **Local**: `agent_agency_contracts::types::planning::BlastRadius` (already from contracts)
- **Contracts**: Same type - identity conversion
- **Usage**: Direct pass-through for blast radius analysis

### ChangeBudget
- **Local**: `crate::council_types::ChangeBudget` (simplified: max_files, max_loc)
- **Contracts**: `agent_agency_contracts::planning_io::ChangeBudget` (extended with migrations, breaking changes, etc.)
- **Usage**: Convert between simplified and full change budget representations

### AcceptanceCriterion
- **Local**: `crate::types::AcceptanceCriterion` (id, given, when, then)
- **Contracts**: `agent_agency_contracts::AcceptanceCriterion` (includes optional priority)
- **Usage**: Convert acceptance criteria with optional priority field

### WorkingSpec
- **Local**: `crate::types::WorkingSpec` (simplified structure)
- **Contracts**: `agent_agency_contracts::WorkingSpec` (comprehensive CAWS structure)
- **Usage**: Convert between simplified and full working spec representations

## Usage Examples

```rust
use crate::planning::type_adapters::{ToContracts, FromContracts};

// Convert local WorkingSpec to contracts
let local_spec = WorkingSpec { /* ... */ };
let contracts_spec = local_spec.to_contracts();

// Convert back from contracts
let back_to_local = WorkingSpec::from_contracts(contracts_spec);

// Convert TaskScope
let local_scope = TaskScope { /* ... */ };
let contracts_scope = local_scope.to_contracts();
```

## Migration Path

These adapters should be considered temporary. The goal is to:

1. Gradually migrate all code to use contracts types directly
2. Remove local type definitions
3. Remove these adapters once migration is complete

## Testing

Unit tests in this module verify round-trip conversions:
- `test_task_scope_conversion` - Verifies TaskScope conversions
- `test_acceptance_criterion_conversion` - Verifies AcceptanceCriterion conversions
- `test_change_budget_conversion` - Verifies ChangeBudget conversions

