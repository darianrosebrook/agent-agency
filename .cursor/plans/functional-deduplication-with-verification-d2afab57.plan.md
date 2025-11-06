<!-- d2afab57-d47f-41f0-a96e-8caa1fc6b9ed ebcb69f6-0602-4e6b-b42b-3b7c3515e706 -->
# Functional Deduplication Plan with Verification

## Objective

Eliminate 60-70% of functional duplication in v3 crates while maintaining backward compatibility and ensuring no critical bugs are introduced.

## Verification Strategy

- Pre-consolidation baseline: Run duplication checker and capture metrics
- Post-consolidation verification: Ensure duplication metrics decrease
- Functional verification: All existing tests pass, no regressions
- Compilation verification: All crates compile without errors
- Integration verification: Dependent crates still work

## Phase 1: Critical Duplications (Breaking Changes)

### 1.1 Eliminate Learning Orchestrator Duplication

**Files:** `iterations/v3/agent-research/src/learning_algorithms/orchestrator.rs` ↔ `iterations/v3/agent-research/src/orchestrator.rs`

**Action:**

- Compare both files line-by-line to identify any unique logic
- If `learning_algorithms/orchestrator.rs` has unique code, merge into `orchestrator.rs`
- Delete `learning_algorithms/orchestrator.rs`
- Update `learning_algorithms.rs` to remove orchestrator re-export
- Update any imports using `learning_algorithms::orchestrator` to use `crate::orchestrator`

**Verification:**

- [ ] `cargo check --package agent-research` passes
- [ ] `cargo test --package agent-research` passes
- [ ] Duplication checker shows orchestrator cluster size reduced from 2 to 1
- [ ] No compilation errors in dependent crates

### 1.2 Consolidate Evidence Collectors Base Implementation

**Files:** `iterations/v3/agent-research/src/evidence/constitutional.rs`, `documentation.rs`, `performance.rs`, `security.rs`

**Action:**

- Create `iterations/v3/agent-research/src/evidence/base.rs` with:
  - `EvidenceCollector` trait (if not exists)
  - `BaseEvidenceCollector` struct with common fields and methods
  - Common `new()` and `with_config()` implementations
- Refactor each collector to extend base:
  - Keep only specialized `collect_evidence()` logic
  - Remove duplicate constructor and config code

**Verification:**

- [ ] All 4 evidence collectors compile
- [ ] `cargo test --package agent-research --lib evidence` passes
- [ ] Duplication checker shows evidence collector pairs reduced from 6 to 0
- [ ] Public API unchanged (collectors still exportable from mod.rs)

### 1.3 Unify Security Error Types

**Files:** `iterations/v3/system-quality-security/src/data_encryption.rs`, `keystore.rs`, `sandbox.rs`, `secret_manager.rs`

**Action:**

- Create `iterations/v3/system-quality-security/src/errors.rs` with:
  - Common error enum variants: `KeyNotFound`, `AccessDenied`, `InvalidKey`, etc.
  - Generic error types that can be specialized
- Replace duplicate error enums in each module with:
  - Type aliases to common errors, OR
  - Specialized error enums that extend common base

**Verification:**

- [ ] `cargo check --package system-quality-security` passes
- [ ] `cargo test --package system-quality-security` passes
- [ ] Duplication checker shows security error pairs reduced from 6 to 0
- [ ] Error messages remain descriptive and specific

## Phase 2: Structural Consolidations

### 2.1 Extract Judge Base Implementation

**Files:** `iterations/v3/agent-constitutional-council/src/judges/integration_validator.rs`, `quality_evaluator.rs`, `technical_auditor.rs`

**Action:**

- Create `iterations/v3/agent-constitutional-council/src/judges/base.rs` with:
  - `BaseJudge<T>` generic struct with common `new(engine)` implementation
  - Common rubric initialization patterns
- Refactor each judge to use base:
  - Keep only judge-specific rubric definitions
  - Remove duplicate constructor code

**Verification:**

- [ ] All 3 judges compile and implement `Judge` trait correctly
- [ ] `cargo test --package agent-constitutional-council` passes
- [ ] Duplication checker shows judge constructor pairs reduced from 3 to 0
- [ ] Judge creation via `Judges::new()` still works

### 2.2 Extract Specialized Worker Base

**Files:** `iterations/v3/agent-workers/src/specialized_workers.rs` (multiple implementations)

**Action:**

- Create `iterations/v3/agent-workers/src/workers/base.rs` with:
  - `BaseSpecializedWorker` trait with common execution pattern
  - Common logging, error handling, task parsing utilities
- Refactor each specialized worker to:
  - Extend base implementation
  - Keep only worker-specific execution logic

**Verification:**

- [ ] All specialized workers compile
- [ ] `cargo test --package agent-workers` passes
- [ ] Duplication checker shows worker template pairs reduced from 6 to 0
- [ ] Worker execution behavior unchanged

## Phase 3: API Type Consolidation

### 3.1 Eliminate Duplicate Waiver Types

**Files:** `iterations/v3/data-infrastructure/src/api/types.rs` ↔ `models.rs`

**Action:**

- Remove `WaiverResponse` from `api/types.rs`
- Update API handlers to use `Waiver` from `models.rs` directly
- If transformation needed, create `api/transform.rs` with conversion functions

**Verification:**

- [ ] `cargo check --package data-infrastructure` passes
- [ ] `cargo test --package data-infrastructure` passes
- [ ] Duplication checker shows waiver type pair eliminated
- [ ] API endpoints still return correct JSON structure

## Verification Checklist (After Each Phase)

### Compilation Verification

```bash
# Run for each affected crate
cargo check --package <crate-name>
cargo build --package <crate-name>
```

### Test Verification

```bash
# Run full test suite
cargo test --package <crate-name>
# Run integration tests if available
cargo test --package <crate-name> --test '*'
```

### Duplication Verification

```bash
# Run duplication checker and verify metrics improved
node scripts/quality-gates/check-functional-duplication.mjs ci | grep -E "Pair|Cluster" | wc -l
# Compare before/after counts
```

### Integration Verification

```bash
# Build entire workspace
cargo build --workspace
# Run workspace tests
cargo test --workspace
```

### Regression Verification

- [ ] All existing unit tests pass
- [ ] All existing integration tests pass
- [ ] No new compiler warnings introduced
- [ ] Public API surface unchanged (verify with `cargo doc`)
- [ ] Dependent crates compile without modification

## Success Criteria

### Quantitative Metrics

- Duplication pairs reduced by 60-70% (from ~100 to ~30-40)
- Duplication clusters reduced by 50% (from 8 to 4)
- Zero compilation errors across workspace
- Zero test failures across workspace
- Duplication checker shows no new violations

### Qualitative Metrics

- Code follows SOLID principles (SRP, DRY)
- Base implementations are reusable and well-tested
- Public APIs remain backward compatible
- Documentation updated for new base types

## Rollback Procedure

If verification fails at any phase:

1. Revert changes using git: `git revert <commit-hash>`
2. Restore deleted files from git history if needed
3. Re-run verification to confirm rollback successful
4. Document failure reason before attempting again

## Risk Mitigation

- **Risk:** Breaking changes affect dependent crates
  - **Mitigation:** Maintain public API compatibility, use type aliases where needed

- **Risk:** Tests fail after consolidation
  - **Mitigation:** Run tests after each file change, not just at end

- **Risk:** Performance regression from abstraction
  - **Mitigation:** Benchmark critical paths before/after (if applicable)

- **Risk:** Lost functionality during merge
  - **Mitigation:** Line-by-line comparison before deletion, preserve unique logic

### To-dos

- [ ] Phase 1.1: Eliminate learning orchestrator duplication - compare files, merge unique logic, delete duplicate, update imports
- [ ] Phase 1.2: Consolidate evidence collectors - create base.rs, refactor 4 collectors to extend base
- [ ] Phase 1.3: Unify security error types - create errors.rs, replace duplicate enums with type aliases
- [ ] Phase 2.1: Extract judge base implementation - create base.rs, refactor 3 judges to use base
- [ ] Phase 2.2: Extract specialized worker base - create base.rs, refactor workers to extend base
- [ ] Phase 3.1: Eliminate duplicate waiver types - remove WaiverResponse, use Waiver directly
- [ ] Verification: Run cargo check/build on all affected crates, ensure zero compilation errors
- [ ] Verification: Run cargo test on all affected crates, ensure zero test failures
- [ ] Verification: Run duplication checker, verify 60-70% reduction in duplication pairs
- [ ] Verification: Build and test entire workspace, verify dependent crates unaffected