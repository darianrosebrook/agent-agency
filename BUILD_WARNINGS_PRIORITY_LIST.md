# Build Warnings Priority List - v3 Cleanup

**Generated:** $(date)  
**Total Warnings:** 209 warnings across 4 crates  
**Status:** All build errors resolved ✅  
**Note:** Improvements reserved for v4 - focus on quick fixes only

---

## Summary by Crate

| Crate | Warnings | Priority | Estimated Effort |
|-------|----------|----------|------------------|
| `agent-orchestration` | 191 | P0-P1 | 4-6 hours |
| `agent-constitutional-council` | 8 | P1 | 1 hour |
| `data-interfaces-adapters` | 7 | P1 | 30 min |
| `data-infrastructure` | 3 | P2 | 15 min |

---

## Priority Classification

- **P0 (Critical)**: Ambiguous glob re-exports (can cause compilation issues in future Rust versions)
- **P1 (Important)**: Unused variables/fields that indicate incomplete implementations or dead code
- **P2 (Nice to Have)**: Unused code that may be used in v4, or intentionally unused for future features

---

## Worker Assignments

### Worker 1: Ambiguous Glob Re-exports (P0) - `agent-orchestration`
**Priority:** P0 - Critical  
**Estimated Time:** 1-2 hours  
**Crate:** `agent-orchestration`  
**File:** `iterations/v3/agent-orchestration/src/planning/mod.rs`

**Issues:**
- 7 ambiguous glob re-exports causing naming conflicts
- Types re-exported multiple times: `WorkerPerformance`, `AuditEvent`, `PlanGenerationStrategy`, `PlanningConstraints`, `CostLimits`, `PlanningContext`, `TodoIntegration`

**Fix Strategy:**
1. Identify duplicate re-exports in `mod.rs` lines 56-65
2. Remove redundant `pub use` statements
3. Use explicit imports instead of glob re-exports where possible
4. Verify no breaking changes to public API

**Files to Modify:**
- `iterations/v3/agent-orchestration/src/planning/mod.rs`

**Acceptance Criteria:**
- [ ] Zero ambiguous glob re-export warnings
- [ ] All tests pass
- [ ] Public API unchanged (verify with `cargo doc`)

---

### Worker 2: Unused Variables - Planning Module (P1) - `agent-orchestration`
**Priority:** P1 - Important  
**Estimated Time:** 1-2 hours  
**Crate:** `agent-orchestration`  
**Files:** Multiple files in `planning/` subdirectory

**Issues:**
- 15+ unused variables in planning-related code
- Indicates incomplete implementations or dead code paths

**Specific Warnings:**
1. `test_type` - `quality_gates.rs:213`
2. `plan_id` - `storage.rs:1656` (and multiple other locations)
3. `uuid` - `parallel_coordinator.rs:620`
4. `title` - `todo_integration.rs:83`
5. `plan` - `todo_integration.rs:491`
6. `milestone_id` - `todo_integration.rs:496, 501` (multiple)
7. `gate` - `todo_integration.rs:511`
8. `todo_integration_inner` - `factory.rs:216`
9. `working_spec` - `legacy_plan_adapter.rs:36`
10. `budget_max_files` - `planning_engine_impl.rs:772`
11. `budget_max_loc` - `planning_engine_impl.rs:773`
12. `audit_manager` - `plan_executor.rs:636`
13. `state` - `plan_executor.rs:916`

**Fix Strategy:**
1. Review each unused variable to determine if:
   - Should be used (complete implementation)
   - Should be prefixed with `_` (intentionally unused for now)
   - Should be removed (dead code)
2. Prefix with `_` if reserved for v4
3. Remove if truly dead code

**Files to Modify:**
- `iterations/v3/agent-orchestration/src/planning/quality_gates.rs`
- `iterations/v3/agent-orchestration/src/planning/storage.rs`
- `iterations/v3/agent-orchestration/src/planning/parallel_coordinator.rs`
- `iterations/v3/agent-orchestration/src/planning/todo_integration.rs`
- `iterations/v3/agent-orchestration/src/planning/factory.rs`
- `iterations/v3/agent-orchestration/src/planning/legacy_plan_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs`
- `iterations/v3/agent-orchestration/src/planning/plan_executor.rs`

**Acceptance Criteria:**
- [ ] All unused variable warnings resolved
- [ ] Code compiles without warnings
- [ ] No functionality broken (verify with tests)

---

### Worker 3: Unused Variables - Core Orchestration (P1) - `agent-orchestration`
**Priority:** P1 - Important  
**Estimated Time:** 1 hour  
**Crate:** `agent-orchestration`  
**Files:** Core orchestration files

**Issues:**
- 10+ unused variables in core orchestration logic
- May indicate incomplete features or dead code

**Specific Warnings:**
1. `risk_tier` - `council.rs:1264`
2. `critical_issues` - `decision_making.rs:406`
3. `task_context` - `evidence_enrichment.rs:233`
4. `final_verdict` - `refinement_loop.rs:199` (value assigned but never read)
5. `last_error` - `refinement_loop.rs:248, 272` (multiple, value assigned but never read)
6. `pleading_result` - `caws_adjudication_cycle.rs:168`
7. `deliberation_result` - `caws_adjudication_cycle.rs:176`
8. `publication_result` - `caws_adjudication_cycle.rs:184`
9. `claim_extractor` - `caws_adjudication_cycle.rs:448`
10. `alpha` - `rubric_engineering.rs:584`
11. `avg_evidence` - `rubric_engineering.rs:598`
12. `avg_budget` - `rubric_engineering.rs:602`
13. `avg_gate` - `rubric_engineering.rs:606`
14. `avg_provenance` - `rubric_engineering.rs:610`
15. `agent_history` - `curriculum_learning.rs:661`
16. `tests_added` - `adapter.rs:273`
17. `deterministic` - `adapter.rs:274`
18. `output` - `adapter.rs:526, 574` (multiple)

**Fix Strategy:**
1. Review context of each unused variable
2. Prefix with `_` if reserved for v4 features
3. Remove assignments if values are never read
4. Consider if variables should be used (incomplete implementation)

**Files to Modify:**
- `iterations/v3/agent-orchestration/src/council.rs`
- `iterations/v3/agent-orchestration/src/decision_making.rs`
- `iterations/v3/agent-orchestration/src/evidence_enrichment.rs`
- `iterations/v3/agent-orchestration/src/planning/refinement_loop.rs`
- `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`
- `iterations/v3/agent-orchestration/src/planning/rubric_engineering.rs`
- `iterations/v3/agent-orchestration/src/planning/curriculum_learning.rs`
- `iterations/v3/agent-orchestration/src/adapter.rs`

**Acceptance Criteria:**
- [ ] All unused variable warnings resolved
- [ ] No dead code introduced
- [ ] Tests pass

---

### Worker 4: Unused Code - Data Infrastructure (P2) - `data-infrastructure`
**Priority:** P2 - Nice to Have  
**Estimated Time:** 15-30 minutes  
**Crate:** `data-infrastructure`  
**File:** `iterations/v3/data-infrastructure/src/embedding/provider.rs`

**Issues:**
- Unused fields and functions in CLIP embedding provider
- Likely reserved for future v4 features

**Specific Warnings:**
1. Field `model` never read - `provider.rs:91`
2. Field `model_path` never read - `provider.rs:947`
3. Functions `load_clip_model` and `load_clip_model_from_path` never used - `provider.rs:1052`

**Fix Strategy:**
1. Add `#[allow(dead_code)]` if reserved for v4
2. Or remove if truly unused
3. Document if intentionally unused for future features

**Files to Modify:**
- `iterations/v3/data-infrastructure/src/embedding/provider.rs`

**Acceptance Criteria:**
- [ ] Warnings resolved or documented
- [ ] No breaking changes
- [ ] Code compiles cleanly

---

### Worker 5: Unused Variables - Data Interfaces (P1) - `data-interfaces-adapters`
**Priority:** P1 - Important  
**Estimated Time:** 30 minutes  
**Crate:** `data-interfaces-adapters`  
**File:** `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`

**Issues:**
- 7 warnings in API server binary
- Some may be auto-fixable with `cargo fix`

**Fix Strategy:**
1. Run `cargo fix --bin "agent-agency-api-server"` to auto-fix suggestions
2. Manually fix remaining warnings
3. Review each unused variable context

**Files to Modify:**
- `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`

**Acceptance Criteria:**
- [ ] All warnings resolved
- [ ] API server compiles cleanly
- [ ] No functionality broken

---

### Worker 6: Unused Variables - Constitutional Council (P1) - `agent-constitutional-council`
**Priority:** P1 - Important  
**Estimated Time:** 1 hour  
**Crate:** `agent-constitutional-council`  
**Files:** Multiple files

**Issues:**
- 8 warnings in constitutional council crate
- Unused variables and potentially dead code

**Fix Strategy:**
1. Review each warning individually
2. Prefix with `_` if reserved for v4
3. Remove if dead code
4. Complete implementation if variable should be used

**Files to Modify:**
- (Files to be identified during investigation)

**Acceptance Criteria:**
- [ ] All warnings resolved
- [ ] Tests pass
- [ ] No breaking changes

---

### Worker 7: Unused Variables - System Federated ML (P1) - `system-federated-ml`
**Priority:** P1 - Important  
**Estimated Time:** 15 minutes  
**Crate:** `system-federated-ml`  
**File:** `iterations/v3/system-federated-ml/src/evidence_collection_tools.rs`

**Issues:**
- 1 unused variable warning
- Simple fix

**Specific Warning:**
1. `context` - `evidence_collection_tools.rs:51`

**Fix Strategy:**
1. Prefix with `_` if intentionally unused
2. Or use the variable if it should be used

**Files to Modify:**
- `iterations/v3/system-federated-ml/src/evidence_collection_tools.rs`

**Acceptance Criteria:**
- [ ] Warning resolved
- [ ] Code compiles cleanly

---

## Quick Reference: Warning Types

### Ambiguous Glob Re-exports
- **Impact:** Can cause compilation failures in future Rust versions
- **Fix:** Remove duplicate `pub use` statements or use explicit imports

### Unused Variables
- **Impact:** Code quality, may indicate incomplete implementations
- **Fix:** Prefix with `_` if intentionally unused, or use the variable

### Unused Fields
- **Impact:** Dead code, may indicate incomplete struct design
- **Fix:** Add `#[allow(dead_code)]` if reserved for v4, or remove

### Unused Functions
- **Impact:** Dead code, may indicate incomplete features
- **Fix:** Add `#[allow(dead_code)]` if reserved for v4, or remove

### Unused Assignments
- **Impact:** Unnecessary computation, performance concern
- **Fix:** Remove assignment if value is never read

---

## Verification Commands

After each worker completes their fixes:

```bash
# Check specific crate
cargo check -p <crate-name>

# Check entire workspace
cargo check --workspace

# Count remaining warnings
cargo check --workspace 2>&1 | grep -E "^warning:" | grep -v "Swift\|CoreML\|ANE\|generated" | wc -l

# Run tests
cargo test --workspace
```

---

## Notes for Workers

1. **Focus on Quick Fixes**: Since improvements are reserved for v4, prioritize:
   - Prefixing unused variables with `_`
   - Removing truly dead code
   - Fixing ambiguous re-exports

2. **Avoid Major Refactoring**: Don't redesign code, just fix warnings

3. **Preserve Functionality**: Ensure all tests pass after fixes

4. **Document Intentional Unused Code**: If code is reserved for v4, add comments explaining why

5. **Test After Each Fix**: Run `cargo check` and `cargo test` after each change

---

## Progress Tracking

- [ ] Worker 1: Ambiguous glob re-exports
- [ ] Worker 2: Unused variables - Planning module
- [ ] Worker 3: Unused variables - Core orchestration
- [ ] Worker 4: Unused code - Data infrastructure
- [ ] Worker 5: Unused variables - Data interfaces
- [ ] Worker 6: Unused variables - Constitutional council
- [ ] Worker 7: Unused variables - System federated ML

**Target:** Zero warnings (excluding Swift/CoreML build warnings which are informational)

---

## Estimated Total Time

**Total Estimated Time:** 6-9 hours across 7 workers  
**Parallel Execution:** Can be done in parallel by different workers  
**Sequential Execution:** ~1 day if done sequentially


