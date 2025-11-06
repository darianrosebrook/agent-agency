# V3 Build Errors - Priority Report & Worker Distribution

**Generated:** 2025-01-28  
**Total Errors:** 91  
**Affected Crate:** `data-interfaces-adapters`  
**Affected Binaries:** 3

---

## Summary

| Binary | Error Count | Priority | Estimated Time |
|--------|-------------|----------|----------------|
| `agent-agency-cli` | 46 | HIGH | 4-6 hours |
| `agent-agency-advanced-cli` | 35 | HIGH | 3-4 hours |
| `agent-agency-api-server` | 10 | MEDIUM | 1-2 hours |
| **TOTAL** | **91** | - | **8-12 hours** |

---

## Error Breakdown by Binary

### 1. agent-agency-cli (46 errors)

**Priority: HIGH** - Core CLI interface

#### Error Categories:
- **Missing Imports (3):** `schemars` unresolved import
- **Missing Derive Macros (4):** `Subcommand` derive macro not found
- **Missing Attributes (11):** `arg` attribute not found
- **Borrow Checker (3):** Borrow of moved value `response`
- **Missing Methods (25):** Various methods not found (e.g., `as_array`, `as_f64`, `as_str` on `Option`)

#### Sample Errors:
```
error[E0432]: unresolved import `schemars` (3 occurrences)
error: cannot find derive macro `Subcommand` in this scope (4 occurrences)
error: cannot find attribute `arg` in this scope (11 occurrences)
error[E0382]: borrow of moved value: `response` (3 occurrences)
error[E0599]: no method named `as_array` found for enum `Option` (multiple)
error[E0277]: trait bound issues with `FromArgMatches` and `Subcommand`
```

#### Fix Strategy:
1. Add `schemars` dependency to `Cargo.toml` (if needed) or remove unused imports
2. Ensure `clap` features include `derive` for `Subcommand`
3. Fix borrow checker issues by cloning or restructuring code
4. Replace `Option::as_array/as_f64/as_str` with proper pattern matching or `if let`

---

### 2. agent-agency-advanced-cli (35 errors)

**Priority: HIGH** - Advanced CLI features

#### Error Categories:
- **Missing File (1):** `dashboard.html` not found
- **Missing Imports (1):** `schemars` unresolved import
- **Missing Fields (12):** Struct fields missing (e.g., `AutonomousExecutorConfig`, `AllowList`, `Budgets`, `SelfPromptingConfig`)
- **Missing Methods (7):** Methods not found (e.g., `send`, `with_config`, `description`, `execute_task`)
- **Type Issues (3):** Cannot find type/value (e.g., `Scope`, `task_id`)
- **Function Signature Mismatch (1):** Wrong number of arguments
- **Comparison Issues (1):** Binary operation `==` cannot be applied to `SafetyMode`

#### Sample Errors:
```
error: couldn't read `dashboard.html`: No such file or directory
error[E0432]: unresolved import `schemars`
error[E0560]: struct `AutonomousExecutorConfig` has no field named `enable_arbiter_adjudication`
error[E0560]: struct `AllowList` has no field named `globs`
error[E0560]: struct `Budgets` has no field named `max_files` / `max_loc`
error[E0560]: struct `SelfPromptingConfig` has no field named `enable_evaluation` / `enable_rollback` / etc.
error[E0599]: no method named `send` found for struct `UnboundedReceiver`
error[E0599]: no function or associated item named `with_config` found for struct `SelfPromptingLoop`
error[E0369]: binary operation `==` cannot be applied to type `SafetyMode`
```

#### Fix Strategy:
1. Create missing `dashboard.html` file or update path
2. Add missing fields to structs or update code to match current struct definitions
3. Fix method calls to match current API (e.g., `UnboundedReceiver::send` → proper channel API)
4. Update `SafetyMode` comparison to use proper trait implementation
5. Fix function call signatures to match current API

---

### 3. agent-agency-api-server (10 errors)

**Priority: MEDIUM** - API server interface

#### Error Categories:
- **Missing Imports (3):** Unresolved imports (e.g., `RestApi`, `audited_orchestrator`, `schemars`)
- **Private Modules (2):** Module `progress_tracker` is private
- **Missing Dependency (1):** Unresolved crate `toml`
- **Missing Fields (4):** Struct `DatabaseConfig` missing fields (`min_connections`, `idle_timeout_seconds`, etc.)

#### Sample Errors:
```
error[E0432]: unresolved import `data_infrastructure::api::server::RestApi`
error[E0432]: unresolved import `agent_orchestration::audited_orchestrator`
error[E0432]: unresolved import `schemars`
error[E0603]: module `progress_tracker` is private (2 occurrences)
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `toml`
error[E0560]: struct `DatabaseConfig` has no field named `min_connections` / `idle_timeout_seconds` / etc.
```

#### Fix Strategy:
1. Update import paths to match current module structure
2. Make `progress_tracker` module public or use public API
3. Add `toml` dependency to `Cargo.toml` or remove usage
4. Update `DatabaseConfig` initialization to match current struct fields

---

## Worker Distribution

### Worker 1: agent-agency-cli (46 errors)
**Estimated Time:** 4-6 hours  
**Focus:** Core CLI functionality

**Tasks:**
1. Fix missing `schemars` import (5 min)
2. Fix `clap` derive macros and attributes (30 min)
3. Fix borrow checker issues with `response` (1 hour)
4. Replace `Option` helper methods with proper pattern matching (2-3 hours)
5. Fix trait bound issues with `FromArgMatches` and `Subcommand` (1 hour)

---

### Worker 2: agent-agency-advanced-cli (35 errors)
**Estimated Time:** 3-4 hours  
**Focus:** Advanced CLI features

**Tasks:**
1. Create or fix `dashboard.html` path (15 min)
2. Fix missing struct fields (12 occurrences) - update struct definitions or code (2 hours)
3. Fix missing methods (`send`, `with_config`, `description`, `execute_task`) (1 hour)
4. Fix type issues (`Scope`, `task_id`) (30 min)
5. Fix function signature and comparison issues (30 min)

---

### Worker 3: agent-agency-api-server (10 errors)
**Estimated Time:** 1-2 hours  
**Focus:** API server interface

**Tasks:**
1. Fix unresolved imports (`RestApi`, `audited_orchestrator`, `schemars`) (30 min)
2. Fix private module access (`progress_tracker`) (30 min)
3. Add `toml` dependency or remove usage (15 min)
4. Fix `DatabaseConfig` struct fields (30 min)

---

## Error Code Distribution

| Error Code | Count | Description |
|------------|-------|-------------|
| E0432 | 7 | Unresolved import |
| E0599 | 32 | Method or function not found |
| E0560 | 16 | Struct field missing |
| E0382 | 3 | Borrow of moved value |
| E0603 | 2 | Module is private |
| E0277 | 6 | Trait bound not satisfied |
| E0433 | 1 | Unresolved module/crate |
| E0425 | 1 | Cannot find value |
| E0422 | 1 | Cannot find type |
| E0061 | 1 | Wrong number of arguments |
| E0369 | 1 | Binary operation cannot be applied |
| Other | 1 | Missing file |
| **TOTAL** | **91** | |

---

## Priority Fix Order

### Phase 1: Quick Wins (30 minutes)
1. Add `schemars` dependency if needed
2. Add `toml` dependency if needed
3. Fix missing file path for `dashboard.html`

### Phase 2: Import & Module Issues (1 hour)
1. Fix unresolved imports (7 occurrences)
2. Fix private module access (2 occurrences)

### Phase 3: Struct Field Updates (2-3 hours)
1. Update struct initializations to match current definitions (16 occurrences)
2. Update API calls to match current signatures

### Phase 4: Method & Trait Issues (3-4 hours)
1. Fix missing methods (32 occurrences)
2. Fix trait bound issues (6 occurrences)
3. Fix borrow checker issues (3 occurrences)

### Phase 5: Type & API Updates (2-3 hours)
1. Fix type mismatches
2. Fix function signatures
3. Fix comparison operations

---

## Notes

- Most errors are due to API changes in dependent crates
- Many struct fields have been renamed or removed
- Several methods have been renamed or moved
- Import paths need to be updated to match current module structure

**Recommended Approach:**
1. Start with Worker 3 (fewest errors, quickest wins)
2. Then Worker 2 (medium complexity)
3. Finally Worker 1 (most errors, but patterns will be established)
