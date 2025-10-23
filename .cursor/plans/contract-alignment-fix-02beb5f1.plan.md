<!-- 02beb5f1-3a7b-4d70-a49d-288b09e81e68 338f326d-9da4-4d4f-a6b2-dbfa87267b41 -->
# Contract Alignment Fix Plan

## Overview

Fix 142 compilation errors in `agent-agency-workers` by aligning with `agent-agency-contracts` API, adding Default implementations, convenience methods for common patterns, and correcting method calls.

## Phase 1: Add Default Implementations to Contracts

### 1.1 ExecutionArtifacts Default

**File**: `iterations/v3/agent-agency-contracts/src/execution_artifacts.rs`

Add `Default` implementation after the struct definition (around line 43):

```rust
impl Default for ExecutionArtifacts {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            task_id: Uuid::nil(),
            working_spec_id: String::new(),
            iteration: 0,
            code_changes: CodeChanges::default(),
            tests: TestArtifacts::default(),
            coverage: CoverageResults::default(),
            linting: LintingResults::default(),
            provenance: Provenance::default(),
            metadata: None,
        }
    }
}
```

### 1.2 Add Default for Nested Structs

Add `Default` implementations for:

- `CodeChanges`
- `TestArtifacts` 
- `CoverageResults`
- `LintingResults`
- `Provenance`

Each should provide sensible empty/zero defaults.

## Phase 2: Add Convenience Methods to WorkingSpec

**File**: `iterations/v3/agent-agency-contracts/src/working_spec.rs`

Add implementation block after struct definitions (around line 57):

```rust
impl WorkingSpec {
    /// Get maximum files allowed from budget constraints
    pub fn max_files(&self) -> Option<u32> {
        self.constraints.budget_limits.as_ref()
            .and_then(|b| b.max_files)
    }

    /// Get maximum LOC allowed from budget constraints
    pub fn max_loc(&self) -> Option<u32> {
        self.constraints.budget_limits.as_ref()
            .and_then(|b| b.max_loc)
    }

    /// Get allowed paths from scope restrictions
    pub fn allowed_paths(&self) -> Vec<String> {
        self.constraints.scope_restrictions.as_ref()
            .map(|s| s.allowed_paths.clone())
            .unwrap_or_default()
    }

    /// Get blocked paths from scope restrictions
    pub fn blocked_paths(&self) -> Vec<String> {
        self.constraints.scope_restrictions.as_ref()
            .map(|s| s.blocked_paths.clone())
            .unwrap_or_default()
    }
}
```

## Phase 3: Fix ArbiterOrchestrator Integration

**File**: `iterations/v3/workers/src/autonomous_executor.rs`

### 3.1 Update Import (line ~29)

Replace stub comment with actual import:

```rust
use agent_agency_orchestration::arbiter::{ArbiterOrchestrator, ArbiterVerdict, VerdictStatus, WorkerOutput};
```

### 3.2 Fix adjudicate_task Call (line ~408)

The method signature is correct - no changes needed. Just ensure the import is present.

## Phase 4: Fix CAWS Validator Integration

**File**: `iterations/v3/workers/src/autonomous_executor.rs`

### 4.1 Replace validate_task_progress Call (line ~841)

Change from:

```rust
self.validator.validate_task_progress(&task_spec, phase_name).await
```

To:

```rust
self.validator.validate_task_execution(
    &working_spec,
    phase_name,
    &RuntimeTaskDescriptor::default() // or construct from task_spec
).await
```

## Phase 5: Fix ExecutionEvent Field Mismatches

**File**: `iterations/v3/workers/src/autonomous_executor.rs`

### 5.1 Remove Invalid Fields from ExecutionFailed

Around line 493-500, the `ExecutionFailed` variant doesn't have `timestamp` field. Remove it or use the correct fields from `agent-agency-contracts/src/execution_events.rs`.

### 5.2 Fix ExecutionPhaseCompleted Fields

Around line 840+, remove `success` and `timestamp` fields that don't exist in the contract.

## Phase 6: Fix WorkingSpec Field Access

**File**: `iterations/v3/workers/src/autonomous_executor.rs`

Replace direct field access with convenience methods:

```rust
// OLD: working_spec.scope_in
// NEW: working_spec.allowed_paths()

// OLD: working_spec.change_budget_max_files  
// NEW: working_spec.max_files()

// OLD: working_spec.change_budget_max_loc
// NEW: working_spec.max_loc()
```

## Phase 7: Fix ExecutionArtifacts Field Access

**File**: `iterations/v3/workers/src/autonomous_executor.rs`

Update all references to removed fields:

- `artifacts.id` → `artifacts.task_id`
- `artifacts.test_results` → `artifacts.tests`
- Access nested statistics properly through the new structure

## Phase 8: Add Missing Cargo Dependencies

**File**: `iterations/v3/workers/Cargo.toml`

Ensure dependency on orchestration crate:

```toml
agent-agency-orchestration = { path = "../orchestration" }
```

## Verification Steps

After all changes:

1. Run `cargo check --package agent-agency-workers`
2. Verify zero compilation errors
3. Run `cargo check --workspace` to ensure no regressions
4. Run `cargo clippy --package agent-agency-workers` for warnings

## Expected Outcome

- Zero compilation errors in workers crate
- Clean contract alignment between workers and contracts
- Maintainable nested structure access via convenience methods
- Proper integration with orchestration and CAWS validator