# Rust Compilation Error Fix Priority Plan

**Date:** $(date +%Y-%m-%d)  
**Total Errors:** 120 across 2 crates  
**Analysis:** Error patterns identified, fixable patterns prioritized

---

## Executive Summary

- **agent-orchestration**: 109 errors (highest priority)
- **data-infrastructure**: 11 errors (mostly trait bounds)

### Error Categories

| Category | Count | Fixability | Priority |
|----------|-------|------------|----------|
| Type Mismatch | 30 | High | P1 |
| Method Not Found | 17 | Medium | P1 |
| Trait Bound | 22 | Medium | P2 |
| Ownership | 5 | High | P1 |
| Other | 46 | Low-Medium | P3 |

---

## Priority 1: High-Impact Fixable Patterns

### Batch 1: String/Path Conversion Errors (Surgical - Automated)

**Pattern:** `String` used where `Path`/`PathBuf` expected (and vice versa)

**Errors:**
- `agent-orchestration/src/planning/scope_guard.rs:229` - `String.is_absolute()` → use `PathBuf::from(path).is_absolute()`
- `agent-orchestration/src/planning/scope_guard.rs:230` - `String.to_string_lossy()` → convert to `PathBuf` first
- `agent-orchestration/src/planning/council_review.rs:914-988` - Multiple `String` to `PathBuf` conversions

**Fix Strategy:**
1. Find all `String` variables used with path methods
2. Replace with `PathBuf::from()` or `Path::new()`
3. Use `to_string_lossy()` on `Path`/`PathBuf`, not `String`

**Estimated Time:** 15 minutes  
**Fix Count:** ~10 errors

### Batch 2: Unwrap_or on usize (Surgical - Automated)

**Pattern:** Calling `unwrap_or()` on `usize` which doesn't have Option methods

**Errors:**
- `agent-orchestration/src/adapter.rs:281,284,288,291` - `max_files.unwrap_or()` and `max_loc.unwrap_or()`

**Root Cause:** `ChangeBudget` fields are `usize` but code expects `Option<u32>`

**Fix Strategy:**
1. Check `ChangeBudget` definition in contracts
2. If `Option<u32>`, update to match
3. If `usize`, remove `unwrap_or()` and use direct value

**Estimated Time:** 10 minutes  
**Fix Count:** 4 errors

### Batch 3: Missing Some() Wrappers (Surgical - Automated)

**Pattern:** Assigning `String` to `Option<String>` field without `Some()`

**Errors:**
- `agent-orchestration/src/autonomous_executor.rs:1615` - `refinement_reason` expects `Option<String>`

**Fix Strategy:**
```rust
// Before
refinement_reason = refinement_reason.clone()

// After
refinement_reason = Some(refinement_reason.clone())
```

**Estimated Time:** 5 minutes  
**Fix Count:** 1 error

### Batch 4: Type Annotation Fixes (Surgical - Semi-Automated)

**Pattern:** Missing type annotations in generic contexts

**Errors:**
- `data-infrastructure/src/audit.rs:227` - `row.get("user_id")` needs type annotation

**Fix Strategy:**
```rust
// Before
"user_id": row.get("user_id")

// After
"user_id": row.get::<String, _>("user_id")  // or appropriate type
```

**Estimated Time:** 10 minutes  
**Fix Count:** 1 error

### Batch 5: Result Handling in Match Arms (Surgical - Manual)

**Pattern:** Match arm returns `Result<T>` but arm expects `T`

**Errors:**
- `agent-orchestration/src/autonomous_executor.rs:1462` - Match arm type mismatch

**Fix Strategy:**
```rust
// Before
match ... {
    Ok(verdict) => verdict,
    Err(e) => return Err(e),
}

// After - add `?` operator or proper error handling
```

**Estimated Time:** 15 minutes  
**Fix Count:** 1 error

---

## Priority 2: API/Type Contract Mismatches

### Batch 6: Missing Struct Fields (Manual Review Required)

**Pattern:** Code references fields that don't exist on structs

**Errors:**
- `agent-orchestration/src/autonomous_executor.rs:1304` - `milestone_results` field missing
- `agent-orchestration/src/autonomous_executor.rs:1768` - `description` field missing on `Milestone`
- `agent-orchestration/src/planning/orchestrator_integration.rs:439` - `timestamp` field missing

**Fix Strategy:**
1. Check struct definitions in `agent-agency-contracts`
2. Update code to use correct field names
3. Or add missing fields to structs (if intentional)

**Estimated Time:** 30 minutes  
**Fix Count:** 3 errors

### Batch 7: Trait Method Signature Mismatches (Manual Review)

**Pattern:** Implemented trait methods have wrong signatures

**Errors:**
- `agent-orchestration/src/planning/factory.rs:30,35,38` - `SessionId` type mismatch in trait impl

**Root Cause:** Using `agent_agency_contracts::SessionId` vs `ports::council_coordinator::SessionId`

**Fix Strategy:**
1. Check trait definition for correct type paths
2. Update impl signatures to match trait
3. May need type aliases or imports

**Estimated Time:** 20 minutes  
**Fix Count:** 3 errors

### Batch 8: Method Not Found - Council Session (Manual Review)

**Pattern:** Methods called but not implemented or in wrong trait

**Errors:**
- `agent-orchestration/src/adapter.rs:230` - `review_task()` not found on `CouncilSession`
- `agent-orchestration/src/adapter.rs:230` - `to_orchestrated_task()` not found

**Fix Strategy:**
1. Check if methods should be on trait vs struct
2. Implement missing methods
3. Update call sites if method moved

**Estimated Time:** 30 minutes  
**Fix Count:** 2 errors

### Batch 9: Scope Type Mismatches (Manual Review)

**Pattern:** `ScopeRestrictions` vs `TaskScope` type confusion

**Errors:**
- `agent-orchestration/src/adapter.rs:296` - Expects `TaskScope`, got `ScopeRestrictions`
- `agent-orchestration/src/planning/council_monitor.rs:188,293,352` - `ScopeRestrictions::default()` not found

**Fix Strategy:**
1. Determine correct type to use
2. Add `Default` impl if needed
3. Update all usages consistently

**Estimated Time:** 25 minutes  
**Fix Count:** 4 errors

---

## Priority 3: Trait Bound Issues

### Batch 10: JsonSchema Trait Bounds (Manual Review)

**Pattern:** Types missing `JsonSchema` derive for API documentation

**Errors:**
- `data-infrastructure/src/api/types.rs:81` - `QualityReport` needs `JsonSchema`
- `data-infrastructure/src/vector_store.rs:25` - `Pool<Postgres>` needs `JsonSchema`
- `data-infrastructure/src/simple_client.rs:19` - `DatabaseClient` needs `JsonSchema`
- `data-infrastructure/src/rate_limiter.rs:67` - `RwLock<HashMap<...>>` needs `JsonSchema`
- Multiple error types need `JsonSchema`

**Fix Strategy:**
1. For custom types: Add `#[derive(JsonSchema)]`
2. For foreign types: Exclude from schema generation with `#[schemars(skip)]`
3. For complex types: Create wrapper types for schema generation

**Estimated Time:** 45 minutes  
**Fix Count:** 10+ errors

### Batch 11: Serde Trait Bounds (Surgical - Automated)

**Pattern:** Types missing `Serialize`/`Deserialize` derives

**Errors:**
- `agent-orchestration/src/planning/plan_executor.rs:117` - `WorkerHealth` and `WorkerPerformance` need derives

**Fix Strategy:**
```rust
#[derive(Serialize, Deserialize)]
pub struct WorkerHealth { ... }

#[derive(Serialize, Deserialize)]
pub struct WorkerPerformance { ... }
```

**Estimated Time:** 5 minutes  
**Fix Count:** 2 errors

---

## Priority 4: Ownership and Mutability

### Batch 12: Arc Mutability Issues (Manual Review)

**Pattern:** Trying to mutably borrow data inside `Arc`

**Errors:**
- `agent-orchestration/src/planning/plan_executor.rs:368,730` - `Arc` mutability issues with `todo_integration`

**Fix Strategy:**
1. Use `Arc<RwLock<>>` or `Arc<Mutex<>>` for shared mutable state
2. Or restructure to avoid mutation through Arc

**Estimated Time:** 20 minutes  
**Fix Count:** 2 errors

### Batch 13: Type Mismatch - Mutex/RwLock (Manual Review)

**Pattern:** Wrong mutex type or async/sync mismatch

**Errors:**
- `agent-orchestration/src/planning/orchestrator_integration.rs:274` - `Mutex` vs `RwLock` or sync/async mismatch

**Fix Strategy:**
1. Check if needs `tokio::sync::Mutex` vs `std::sync::Mutex`
2. Check if needs `RwLock` instead of `Mutex`
3. Ensure async/await compatibility

**Estimated Time:** 15 minutes  
**Fix Count:** 1 error

---

## Priority 5: Other Issues

### Batch 14: MoSCoW Priority Type (Surgical - Automated)

**Pattern:** String used instead of enum type

**Errors:**
- `agent-orchestration/src/autonomous_executor.rs:2369` - `"Should"` should be `MoSCoWPriority::Should`

**Fix Strategy:**
```rust
// Before
priority: Some("Should".to_string())

// After
priority: Some(MoSCoWPriority::Should)
```

**Estimated Time:** 5 minutes  
**Fix Count:** 1 error

### Batch 15: Duration::from_std (Manual Review)

**Pattern:** Non-existent method `from_std()` on Duration

**Errors:**
- `agent-orchestration/src/planning/scope_guard.rs:363` - `Duration::from_std()` not found

**Fix Strategy:**
1. Check what conversion is needed
2. Use `Duration::from_secs()`, `from_millis()`, etc. as appropriate
3. Or convert from `std::time::Duration` if different

**Estimated Time:** 10 minutes  
**Fix Count:** 1 error

### Batch 16: Missing Methods on Types (Manual Review)

**Pattern:** Methods called that don't exist on type

**Errors:**
- `agent-orchestration/src/planning/parallel_coordinator.rs:168` - `ExecutionContext.as_mut()` not found
- `agent-orchestration/src/planning/factory.rs:137` - `PlanGenerator::new()` wrong argument count
- `agent-orchestration/src/planning/todo_integration.rs:386` - `get_planning_telemetry()` not found
- `agent-orchestration/src/planning/council_review.rs:803` - `get_audit_trail_entries()` not found
- `agent-orchestration/src/planning/council_review.rs:1019` - `QualityGates.as_ref()` not found

**Fix Strategy:**
1. Check if methods exist with different names
2. Check if methods moved to traits
3. Implement missing methods or update call sites

**Estimated Time:** 60 minutes  
**Fix Count:** 5+ errors

### Batch 17: Ambiguous Numeric Types (Surgical - Automated)

**Pattern:** Numeric type inference failures

**Errors:**
- `agent-orchestration/src/planning/council_review.rs:1044` - Ambiguous float type

**Fix Strategy:**
```rust
// Before
constitutional_score: constitutional_score.max(0.0)

// After
constitutional_score: constitutional_score.max(0.0f64)  // or f32
```

**Estimated Time:** 5 minutes  
**Fix Count:** 1 error

---

## Implementation Plan

### Phase 1: Quick Wins (30-45 minutes)
1. ✅ Batch 1: String/Path conversions (15 min)
2. ✅ Batch 2: Unwrap_or on usize (10 min)
3. ✅ Batch 3: Missing Some() wrappers (5 min)
4. ✅ Batch 11: Serde derives (5 min)
5. ✅ Batch 14: MoSCoW Priority (5 min)
6. ✅ Batch 17: Ambiguous numeric (5 min)

**Target:** ~20 errors fixed

### Phase 2: Type Fixes (1-1.5 hours)
1. ✅ Batch 4: Type annotations (10 min)
2. ✅ Batch 5: Result handling (15 min)
3. ✅ Batch 6: Missing struct fields (30 min)
4. ✅ Batch 7: Trait method signatures (20 min)
5. ✅ Batch 13: Mutex type fixes (15 min)

**Target:** ~15 errors fixed

### Phase 3: API Contracts (1.5-2 hours)
1. ✅ Batch 8: Council session methods (30 min)
2. ✅ Batch 9: Scope type mismatches (25 min)
3. ✅ Batch 16: Missing methods (60 min)

**Target:** ~15 errors fixed

### Phase 4: Trait Bounds (1 hour)
1. ✅ Batch 10: JsonSchema bounds (45 min)
2. ✅ Batch 12: Arc mutability (20 min)

**Target:** ~12 errors fixed

### Phase 5: Remaining Issues (30 minutes)
1. ✅ Batch 15: Duration conversion (10 min)
2. ✅ Any remaining errors (20 min)

**Target:** ~5 errors fixed

---

## Automated Fix Scripts

### Script 1: Path Conversion Fixer
```bash
# Fix String.is_absolute() and to_string_lossy() issues
# Find: (\w+)\.is_absolute\(\)
# Replace with: PathBuf::from(&$1).is_absolute()

# Find: (\w+)\.to_string_lossy\(\)
# Replace with: PathBuf::from(&$1).to_string_lossy()
```

### Script 2: Unwrap_or Remover
```bash
# Fix unwrap_or on usize
# Find: \.unwrap_or\(
# Replace based on context - may need manual review
```

### Script 3: Type Annotation Adder
```bash
# Add type annotations to row.get() calls
# Find: row\.get\("([^"]+)"\)
# Replace with: row.get::<String, _>("$1")  # Adjust type as needed
```

---

## Progress Tracking

- [ ] Phase 1: Quick Wins (~20 errors)
- [ ] Phase 2: Type Fixes (~15 errors)
- [ ] Phase 3: API Contracts (~15 errors)
- [ ] Phase 4: Trait Bounds (~12 errors)
- [ ] Phase 5: Remaining (~5 errors)

**Total Progress:** 0/120 errors fixed

---

## Notes

- Many errors are interconnected - fixing one may resolve others
- Type mismatches often cascade - fix root cause first
- Trait bound issues may require contract changes
- Some errors need architectural decisions (e.g., mutability patterns)

