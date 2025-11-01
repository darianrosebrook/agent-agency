<!-- f1e8a904-d9d1-45aa-bc1d-dbcded018eb9 facf2fe7-d2f5-4660-9735-12bc3d3a763b -->
# Fix Compilation Errors Plan

## Overview

Fix ~600 compilation errors blocking `agent-orchestration` crate from building. Errors fall into 6 main categories: missing imports, private type access, type mismatches, missing struct fields, missing trait methods, and enum variant mismatches.

## Error Categories

### Category 1: Missing Prelude Import (E0432)

**Problem**: `council_types.rs` imports `agent_agency_contracts::prelude` but should be `agent_agency_contracts::types::prelude`

**Fix**:

- Update `iterations/v3/agent-orchestration/src/council_types.rs:6` to use `agent_agency_contracts::types::prelude::*`

### Category 2: Private Type Access (E0603)

**Problem**: `OrchestratorConfig` and `ProcessingPriority` are private when imported from contracts

**Files Affected**:

- `iterations/v3/agent-orchestration/src/adapter.rs` (lines 545-552)
- `iterations/v3/agent-orchestration/src/multimodal_orchestration.rs` (line 97)
- `iterations/v3/agent-orchestration/src/planning/data_processing_adapter.rs` (lines 65-70)

**Fix Strategy**:

- `OrchestratorConfig` is already defined locally in `iterations/v3/agent-orchestration/src/types.rs:105` - ensure all imports use local version
- `ProcessingPriority` is defined in `agent-agency-contracts/src/types/prelude.rs:57` but may be private - check if it needs to be `pub` or use local definition

**Actions**:

1. Verify `ProcessingPriority` visibility in `agent-agency-contracts/src/types/data_processing.rs`
2. If private, either make it `pub` in contracts OR create local alias in `agent-orchestration/src/multimodal_orchestration.rs`
3. Update all imports to use correct source

### Category 3: Type Mismatches (E0308, E0026)

**Problem 1**: `MilestoneMetrics` should be `Option<MilestoneMetrics>`

**Location**: `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs:328`

**Fix**: Wrap `MilestoneMetrics` in `Some()`:

```rust
// BEFORE
metrics: MilestoneMetrics { ... }

// AFTER
metrics: Some(MilestoneMetrics { ... })
```

**Problem 2**: `FinalDecision` enum variants don't have `rationale` field

**Location**: Multiple files accessing `decision.rationale`

**Fix**: Check `agent-agency-contracts/src/final_verdict.rs` to see actual structure. If rationale is at top level, access via `final_verdict.rationale` instead of `decision.rationale`.

### Category 4: Missing Struct Fields (E0609)

**Problems**:

- `PlanExecutionResult` missing `evidence_artifacts`, `quality_verifications`, `milestone_results`, `completed_at`
- `ExecutionPlan` missing `id` field

**Files Affected**:

- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs`

**Fix Strategy**:

1. Check `agent-agency-contracts/src/planning_io.rs` for actual `PlanExecutionResult` structure
2. Update local usages to match contract structure OR add missing fields if they're required
3. For `ExecutionPlan.id`, check if it's on `contract_plan` (contracts version) or needs to be added to local wrapper

**Actions**:

1. Read `agent-agency-contracts/src/planning_io.rs` to understand actual structure
2. Update field access in `planning_engine_impl.rs` to match contracts
3. If fields are truly missing from contracts, add them with proper defaults

### Category 5: Missing Trait Methods (E0599)

**Problems**:

- `PlanningEngine::generate_plan()` method not found
- `PlanningStorage::store_plan()`, `store_execution_result()`, `get_plan_for_task()`, `get_execution_result()` not found

**Files Affected**:

- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs`

**Fix Strategy**:

1. Check `agent-agency-contracts/src/ports/planning_engine.rs` for actual trait definition
2. Check if methods have different names or signatures
3. Update implementations to match trait contracts

**Actions**:

1. Read `agent-agency-contracts/src/ports/planning_engine.rs` to see actual trait methods
2. Read `agent-agency-contracts/src/ports/memory_system.rs` for `PlanningStorage` if it exists
3. Update method calls to match trait definitions

### Category 6: Generic Arguments and Lifetime Issues (E0107, E0195)

**Problems**:

- Enum takes 2 generic arguments but 1 supplied
- Method lifetime parameters don't match trait declaration

**Fix Strategy**:

1. Check error messages to identify specific enums/methods
2. Update generic arguments to match trait definitions
3. Fix lifetime annotations to match trait declarations

## Implementation Order

1. **Fix Category 1** (prelude import) - Quick fix, unblocks other errors
2. **Fix Category 2** (private types) - Resolves access issues
3. **Fix Category 3** (type mismatches) - Fixes obvious errors
4. **Fix Category 4** (missing fields) - Requires contract verification
5. **Fix Category 5** (missing methods) - Requires trait verification
6. **Fix Category 6** (generics/lifetimes) - Requires careful type matching

## Verification

After each category fix:

- Run `cargo check --package agent-orchestration` to verify error count reduction
- Target: 0 errors remaining

## Files to Modify

Primary files:

- `iterations/v3/agent-orchestration/src/council_types.rs`
- `iterations/v3/agent-orchestration/src/adapter.rs`
- `iterations/v3/agent-orchestration/src/multimodal_orchestration.rs`
- `iterations/v3/agent-orchestration/src/planning/data_processing_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs`

Reference files (read-only, verify structure):

- `iterations/v3/agent-agency-contracts/src/types/prelude.rs`
- `iterations/v3/agent-agency-contracts/src/planning_io.rs`
- `iterations/v3/agent-agency-contracts/src/final_verdict.rs`
- `iterations/v3/agent-agency-contracts/src/ports/planning_engine.rs`