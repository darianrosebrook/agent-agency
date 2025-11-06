# V3 Build Errors - Worker Assignments

**Generated**: Automatically from compilation error analysis  
**Total Errors**: 94  
**Workers**: 3  
**Architecture**: Contracts-First (see `.cursor/plans/contracts-first-architecture-migration-26cce472.plan copy.md`)

---

## Contracts-First Architecture Principles

Before fixing any errors, workers must follow these principles:

1. **Check contracts first**: If a type is shared or used across crates, it should be in `agent-agency-contracts`
2. **Use ports for dependencies**: Instead of direct crate dependencies, use trait ports defined in contracts
3. **Remove duplicates**: Local type definitions should be replaced with contracts types
4. **Explicit imports**: Use explicit module paths, not wildcard imports (`use agent_agency_contracts::types::...`, not `use agent_agency_contracts::prelude::*`)
5. **No async_trait in contracts**: Contracts use `BoxFuture`, not `async_trait` macro

---

## Summary

| Worker | Errors | Estimated Time | Priority |
|--------|--------|----------------|----------|
| Worker 1 | ~30 errors | 3-4 hours | HIGH |
| Worker 2 | ~32 errors | 3-4 hours | HIGH |
| Worker 3 | ~32 errors | 3-4 hours | HIGH/MEDIUM |

---

## WORKER 1 ASSIGNMENT

### Task 1: agent-orchestration - Quick Fixes (Categories 1, 2, 4)
**Priority**: HIGH  
**Estimated Time**: 30-45 minutes  
**Errors**: ~15 errors

**Category 1: Duplicate Type Definitions**
- Location: `src/multimodal_orchestration.rs:76-80`
- **Contracts-First Check**: 
  - Verify if `ConsensusCoordinator`, `KnowledgeSeeker`, `OrchestratorConfig` should be in contracts
  - If shared across crates → move to `agent-agency-contracts/src/types/orchestration/`
  - If local-only → remove duplicates (lines 76-80)
- Types: `ConsensusCoordinator`, `KnowledgeSeeker`, `OrchestratorConfig`
- Reference: Lines 36-40 have the correct definitions
- **Action**: Check `agent-agency-contracts` first, then decide if local or contracts

**Category 2: Duplicate Import Statements**
- Location: `src/multimodal_orchestration.rs:67-72`
- Fix: Remove duplicate imports (lines 67-72)
- Reference: Lines 27-32 have the original imports

**Category 4: Syntax Error - Orphaned Doc Comment**
- Location: `src/adapter.rs:575`
- Fix: Remove orphaned doc comment at line 575
- Quick fix: Delete the comment line

### Task 2: agent-orchestration - Type Mismatches (Category 7)
**Priority**: HIGH  
**Estimated Time**: 2 hours  
**Errors**: ~15 errors

**Category 7: Type Mismatches**
- Locations: `src/lib.rs:325-385`, `src/multimodal_orchestration.rs:718,734,735`
- **Contracts-First Check**:
  1. Verify all types come from contracts where possible
  2. Use contracts `TaskScope` instead of local `TaskScope`
  3. Use contracts `RiskTier` (already imported from contracts)
- **Fixes needed**:
  1. Wrap `scope_out` in `Some()` (line 358)
     - Use `agent_agency_contracts::types::planning::TaskScope` if available
  2. Convert integer to `RiskTier` enum (line 734, 381)
     - Use `agent_agency_contracts::RiskTier::Tier1/Tier2/Tier3` (NOT local definition)
  3. Convert `Vec<String>` to `Option<String>` for `acceptance` field (line 385, 735)
     - Check if contracts has `AcceptanceCriterion` type
- Reference: `src/types.rs:195` for local `TaskScope` (consider migrating to contracts)
- Reference: `agent-agency-contracts/src/task_request.rs:114` for `RiskTier` enum
- **Action**: Prefer contracts types over local types

### Task 3: agent-orchestration - Enum Variants (Category 10)
**Priority**: HIGH  
**Estimated Time**: 1 hour  
**Errors**: ~10 errors

**Category 10: Missing Enum Variants**
- Locations: `src/autonomous_executor.rs:213-216`, `src/lib.rs:373,382`, `src/council.rs`
- **Contracts-First Check**:
  1. **Remove local `RiskTier`**: Already in contracts, use `agent_agency_contracts::RiskTier`
  2. **TaskPriority**: Check if `TaskPriority::Medium` exists in contracts, if not use `Normal`
  3. **FinalDecision**: Check if in contracts, use contracts version
- **Fixes**:
  1. Remove local `RiskTier` redefinition (line 213-216)
     - Use `agent_agency_contracts::RiskTier::Tier1/Tier2/Tier3`
     - Import: `use agent_agency_contracts::RiskTier;`
  2. Replace `TaskPriority::Medium` with `TaskPriority::Normal`
     - Check `agent-agency-contracts/src/types/planning.rs` for available variants
  3. Fix `FinalDecision` pattern matching (use `reason` field, not `rationale`)
     - Check if `FinalDecision` is in contracts
     - If in contracts, use contracts version exclusively
- Reference: `src/decision_making.rs:142` for local `FinalDecision` (consider migrating to contracts)
- Reference: `src/workflow.rs:174-203` for correct pattern matching
- **Action**: Remove all local enum definitions that duplicate contracts types

**Total for Worker 1**: ~40 errors, 3.5-4 hours

---

## WORKER 2 ASSIGNMENT

### Task 1: agent-orchestration - Dependency & Module Issues (Categories 3, 5)
**Priority**: HIGH  
**Estimated Time**: 1-1.5 hours  
**Errors**: ~20 errors

**Category 3: Missing `agent_data_processing` Dependency**
- Location: `Cargo.toml:81` (commented out)
- **Contracts-First Approach**:
  1. **Check if types are in contracts**: Verify if `DataInput`, `DataSource`, `ProcessingId` etc. are in `agent-agency-contracts`
  2. **If in contracts**: Remove direct dependency, use contracts types
  3. **If not in contracts**: Move shared types to contracts first, then use contracts types
  4. **If local-only**: Create port trait in contracts, implement in `agent-data-processing`
- **Recommended Fix**: 
  - Create port traits in `agent-agency-contracts/src/types/data_processing/ports.rs`
  - Implement ports in `agent-data-processing` (similar to `EmbeddingProvider` pattern)
  - Remove direct dependency, use `Arc<dyn DataProcessingPort>` instead
- Reference: `agent-data-processing/src/lib.rs:43-80` re-exports all needed types
- Locations: `src/multimodal_orchestration.rs:19,58,415,878,879`

**Category 5: Missing Module `orchestrator_integration`**
- Location: `src/planning/mod.rs`
- Fix: Add `pub mod orchestrator_integration;` to `planning/mod.rs`
- Reference: `src/planning/orchestrator_integration.rs` exists (722 lines)
- Re-export: `pub use orchestrator_integration::OrchestratorPlanningIntegration;`

### Task 2: agent-orchestration - Function Signatures (Category 8)
**Priority**: HIGH  
**Estimated Time**: 2-3 hours  
**Errors**: ~15 errors

**Category 8: Wrong Function Signatures**
- Location: `src/lib.rs:266,273`
- Fixes:
  1. `Council::new()` - Remove `.await`, add 4 parameters:
     - `config.council_config`
     - `vec![]` (available_judges)
     - `verdict_aggregator` (create via `create_verdict_aggregator()`)
     - `decision_engine` (create via `create_decision_engine()`)
  2. `AutonomousExecutor::new()` - Add 10 parameters (see signature at `src/autonomous_executor.rs:865-898`)
- Factory functions:
  - `src/verdict_aggregation.rs:1326-1327` - `create_verdict_aggregator()`
  - `src/decision_making.rs:753-755` - `create_decision_engine()`
- Reference: `src/council.rs:1474-1500` for complete example
- Reference: `src/autonomous_executor.rs:2019-2032` for complete example

### Task 3: agent-orchestration - Trait Implementation Issues (Category 11)
**Priority**: HIGH  
**Estimated Time**: 1 hour  
**Errors**: ~10 errors

**Category 11: Trait Implementation Issues**
- Location: `src/multimodal_orchestration.rs`
- Error: `E0117` - Implementing traits for external types (orphan rule violation)
- Fix: Remove trait implementations for external types OR use newtype wrappers
- Pattern:
  ```rust
  struct MyType(ExternalType);
  impl MyTrait for MyType { ... }
  ```

**Total for Worker 2**: ~45 errors, 4-5.5 hours

---

## WORKER 3 ASSIGNMENT

### Task 1: agent-orchestration - Missing Types (Category 6)
**Priority**: HIGH  
**Estimated Time**: 1 hour  
**Errors**: ~10 errors

**Category 6: Missing Types in `types` Module**
- Location: `src/types.rs`
- **Contracts-First Check**:
  1. **Check contracts first**: Search `agent-agency-contracts` for `ExecutionMode` and `TaskType`
  2. **If found in contracts**: Re-export from contracts, don't define locally
  3. **If not found but shared**: Add to `agent-agency-contracts/src/types/orchestration/` first
  4. **If local-only**: Add to `src/types.rs` with clear comment that it's orchestration-specific
- **Fixes**:
  1. Check `agent-agency-contracts/src/types/prelude.rs` for `ExecutionMode`
  2. If missing, add `ExecutionMode` to contracts:
     - File: `agent-agency-contracts/src/types/orchestration/mod.rs` (create if needed)
     - Variants: `Strict`, `Auto`, `DryRun`
  3. Check contracts for `TaskType`:
     - If missing and shared → add to contracts
     - If local-only → add to `src/types.rs` with `#[allow(dead_code)]` if unused
- Reference: `src/autonomous_executor.rs:205` has local definition
- Locations: `src/lib.rs:379` uses `crate::types::TaskType`
- **Action**: Always check contracts before creating local types

### Task 2: agent-orchestration - Missing Struct Fields (Category 9)
**Priority**: HIGH  
**Estimated Time**: 1.5-2 hours  
**Errors**: ~15 errors

**Category 9: Missing Struct Fields**
- Locations: `src/lib.rs:325,326,328,406,409`
- Fixes:
  1. Update `ExecutionArtifacts` initialization:
     - Remove: `output_files`, `diff_stats`
     - Use: `execution_id`, `worker_id`, `status`, `output`, `error`
     - Reference: `src/types.rs:66`
  2. Update `TaskExecutionResult` initialization:
     - Remove: `status` field
     - Use: `working_spec`, `artifacts`, `quality_report`
     - Reference: `src/types.rs:55`
  3. Update `CriticalIssue` initialization:
     - Remove: `issue_type`, `impact`
     - Use: `severity`, `category`, `description`, `evidence`
     - Use: `IssueSeverity` from `judge_backup::verdicts`, NOT `RiskSeverity`
     - Reference: `src/judge_backup/verdicts.rs:116`
     - Reference: `src/verdict_aggregation.rs:413-419` for correct pattern
  4. Implement `Default` for `DiffStats` (if needed):
     - Reference: `src/types.rs:264`

### Task 3: system-quality-security (All 5 errors)
**Priority**: MEDIUM  
**Estimated Time**: 2-3 hours  
**Errors**: 5 errors

**Error 1: Borrow Checker Issue**
- Location: `src/data_encryption.rs:402`
- Fix: Extract `algorithm` and `rotation_days` (Copy types) before dropping lock
- Reference: `src/keystore.rs:277-314` for similar pattern

**Error 2: Missing `rand_distr::Laplace` (2 occurrences)**
- Locations: `src/privacy_anonymization.rs:207,471`
- Fix: Replace with manual Laplace implementation
- Reference: `system-federated-ml/src/differential_privacy.rs:95-116` for implementation

**Error 3: Type Annotations for `BoundKey` (2 occurrences)**
- Locations: `src/data_encryption.rs:290,370`
- Fix: Add explicit type annotations:
  - `BoundKey<SealingKey<AES_256_GCM>>` for encryption
  - `BoundKey<OpeningKey<AES_256_GCM>>` for decryption
- Add imports: `use ring::aead::{SealingKey, OpeningKey};`

### Task 4: workspace-dependencies (1 error)
**Priority**: CRITICAL  
**Estimated Time**: 30 minutes  
**Errors**: 1 error

**Fix: Add candle-core to workspace.dependencies**
- Location: Root `Cargo.toml` (or `iterations/v3/Cargo.toml`)
- Issue: `data-infrastructure/Cargo.toml` inherits `candle-core` from workspace, but it's not defined
- Fix: Add to `[workspace.dependencies]`:
  ```toml
  candle-core = { version = "0.3", features = ["metal"] }
  ```
- Reference: `cargo-check-errors.json` for error details

**Total for Worker 3**: ~31 errors, 4.5-6 hours

---

## Recommended Work Order (All Workers)

### Phase 0: Contracts-First Verification (All Workers)
**Before starting fixes, each worker must:**
1. Check `agent-agency-contracts/src/types/` for existing type definitions
2. Verify if shared types should be in contracts before creating local types
3. Review contracts-first plan: `.cursor/plans/contracts-first-architecture-migration-26cce472.plan copy.md`
4. Use explicit imports from contracts: `use agent_agency_contracts::types::...`

### Phase 1: Quick Wins (All Workers)
1. **Worker 1**: Category 4 (syntax error) - 5 min
2. **Worker 1**: Categories 1 & 2 (duplicates) - 30 min
   - **Contracts check**: Verify types aren't in contracts before removing duplicates
3. **Worker 3**: workspace-dependencies - 30 min

### Phase 2: Dependency & Module Resolution (Workers 2 & 3)
4. **Worker 2**: Category 3 (missing dependency) - 30 min
5. **Worker 2**: Category 5 (missing module) - 15 min
6. **Worker 3**: Category 6 (missing types) - 1 hour

### Phase 3: Type & Signature Fixes (All Workers)
7. **Worker 1**: Category 7 (type mismatches) - 2 hours
8. **Worker 2**: Category 8 (function signatures) - 2-3 hours
9. **Worker 3**: Category 9 (struct fields) - 1.5-2 hours

### Phase 4: Enum & Trait Fixes (Workers 1 & 2)
10. **Worker 1**: Category 10 (enum variants) - 1 hour
11. **Worker 2**: Category 11 (trait issues) - 1 hour

### Phase 5: Security Crate (Worker 3)
12. **Worker 3**: system-quality-security (all 5 errors) - 2-3 hours

---

## Error Documentation References

- **agent-orchestration**: `iterations/v3/agent-orchestration/COMPILATION_ERRORS.md` (88 errors, 11 categories)
- **system-quality-security**: `iterations/v3/system-quality-security/COMPILATION_ERRORS.md` (5 errors)
- **workspace-dependencies**: `cargo-check-errors.json` (1 error)

---

## Quick Reference: Key Files

### agent-orchestration
- `src/multimodal_orchestration.rs` - Duplicates, imports, trait issues
- `src/lib.rs` - Function calls, type mismatches, struct initializations
- `src/types.rs` - Missing types
- `src/adapter.rs:575` - Syntax error
- `src/planning/mod.rs` - Missing module export
- `Cargo.toml:81` - Missing dependency

### system-quality-security
- `src/data_encryption.rs` - Type annotations, borrow checker
- `src/privacy_anonymization.rs` - Laplace distribution

### Workspace
- Root `Cargo.toml` - Add `candle-core` to `workspace.dependencies`

---

## Success Criteria

After all fixes:
- [ ] `cargo check` passes in `iterations/v3/`
- [ ] All 94 errors resolved
- [ ] No new errors introduced
- [ ] All tests pass (where applicable)

### Contracts-First Verification Criteria
- [ ] No duplicate type definitions between local and contracts types
- [ ] All shared types are in `agent-agency-contracts`
- [ ] Direct crate dependencies replaced with port traits where appropriate
- [ ] Explicit imports used (no wildcard `use agent_agency_contracts::prelude::*`)
- [ ] Local types documented as orchestration-specific if not in contracts
- [ ] Type adapters removed if migration complete (check `src/planning/type_adapters.rs`)

## Contracts-First Reference

For detailed guidance on contracts-first architecture:
- **Plan**: `.cursor/plans/contracts-first-architecture-migration-26cce472.plan copy.md`
- **Contracts crate**: `iterations/v3/agent-agency-contracts/src/types/`
- **Type adapters**: `iterations/v3/agent-orchestration/src/planning/type_adapters.rs` (for migration)
- **Contracts prelude**: `iterations/v3/agent-agency-contracts/src/types/prelude.rs`

