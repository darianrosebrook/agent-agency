# Placeholder Detection & CI Gate Strategy

**Purpose:** Prevent incomplete implementations from shipping while allowing TODOs as documentation/notes.

**Current State:**
- You have 360+ TODOs across the codebase (mostly documentation-style)
- Council already detects them and penalizes scores
- Python analyzer (`todo_analyzer.py`) distinguishes explicit vs hidden patterns
- Goal: Fail CI only for critical placeholders; allow documentary TODOs

---

## Classification System

### CRITICAL (FAIL CI immediately)

These are actual incomplete implementations that should never reach production:

```rust
// PLACEHOLDER: Implementation stub - will be replaced
fn process_image(&self, data: &[u8]) -> Result<String> {
    Ok("stub_output".to_string())
}

// PLACEHOLDER: TODO - integrate actual Vision Framework
async fn extract_text_from_image(&self, path: &Path) -> Result<Vec<String>> {
    // Simulated response
    Ok(vec!["text1".to_string()])
}

// TODO: Implement actual database integration instead of simulation
fn save_task(&self, task: &Task) -> Result<()> {
    // In-memory only
    Ok(())
}

// FIXME: This is broken and needs real implementation
fn compute_model_performance(&self) -> f64 {
    42.0  // Magic number placeholder
}

// unimplemented!() or todo!() macros
fn validate_schema(&self) -> Result<()> {
    unimplemented!("Implement schema validation")
}

throw new Error("TODO: Feature not implemented");
return null; // TODO: Implement

assert False, "TODO: Fill this in"
```

**Pattern:** Actual code that won't work or returns fake/stub data.

**CI Action:** FAIL with message:
```
ERROR: Incomplete implementations found (must be removed before merge):
  - file.rs:42: PLACEHOLDER: Integration stub
  - file.rs:100: unimplemented!()
  - file.ts:55: // TODO: Feature not implemented

Fix by either:
1. Implement the feature
2. Move to P1 (acceptable if within planned scope)
3. Remove dead code
```

---

### ACCEPTED (allow in CI, tracked)

These are documentary TODOs that don't block functionality:

```rust
/// TODO: Implement actual ONNX protobuf parsing with onnx-proto crate
/// - [ ] Replace heuristic string matching with proper protobuf parsing
/// - [ ] Add onnx-proto crate dependency for full ONNX format support
///
/// For now, using simplified pattern matching that covers 95% of models.
fn parse_onnx_metadata(&self, model_data: &[u8]) -> Result<IoSchema> {
    // Simplified but working implementation
    let magic = String::from_utf8_lossy(&model_data[0..4]);
    if magic == "ONNX" {
        // ... actual parsing logic ...
    }
    Ok(IoSchema { /* filled in */ })
}

// TODO: Replace simple average fusion with sophisticated result fusion algorithms
// Requirements for completion:
// - Support for weighted fusion based on model confidence
// - Dynamic weighting based on task complexity
// - Caching for frequently occurring patterns
//
// Current simple average works for MVP but will be optimized in next phase.
fn fuse_results(&self, results: &[SearchResult]) -> SearchResult {
    // Current working implementation
}

// TODO: Implement database query execution and result analysis
// - [ ] Execute actual SQL queries against performance database
// - [ ] Parse query results into structured format
// NOTE: Currently using mock data that mirrors expected schema
async fn query_performance_db(&self, table: &str) -> Result<Vec<Row>> {
    mock_data_for_table(table)  // Will replace with real DB
}

// TODO: Migrate from in-memory storage to persistent Redis
// For now, testing with in-memory cache to validate logic.
// Will move to production Redis when load-tested.
fn get_cached_value(&self, key: &str) -> Option<String> {
    self.in_memory_cache.get(key)
}

# TODO: This will be replaced with production OpenTelemetry integration
# Using basic logging for now to validate traces format
def record_trace(trace_data):
    logger.info(f"Trace: {trace_data}")
```

**Pattern:** Working code with clear future improvement notes; includes context/requirements; won't crash production.

**CI Action:** WARN (or allow, depending on phase):
```
INFO: 42 documentary TODOs found in production code (allowed for now)
├── 15 in orchestration/ (optimization candidates)
├── 12 in database/ (schema improvements)
├── 10 in workers/ (feature expansion)
└── 5 in api-layer/ (backwards-compat upgrades)

Monitoring: These will be tracked in metrics; review each phase.
```

---

### DEFERRED (tracked, not in src/)

These are speculative or aspirational ideas that don't belong in production:

```
docs/archive/implementation-tracking/
├── FUTURE_FEATURES.md
├── ML_PIPELINE_ROADMAP.md
└── PERFORMANCE_OPTIMIZATIONS.md

scripts/experiments/
├── quantization_testing.py  (exploratory, not production)
├── clip_integration_poc.py  (proof of concept)
└── ml_inference_bench.rs    (benchmark setup)
```

**CI Action:** Skip (these aren't in src/, so they never trigger the gate).

---

## Implementation Strategy

### 1. Update Council Scorer

Differentiate penalty by TODO category:

```rust
// iterations/v3/council/src/advanced_arbitration.rs

// CRITICAL - causes immediate rejection
TodoCategory::Explicit | TodoCategory::PlaceholderCode | TodoCategory::CodeStub => {
    score *= 0.2; // 80% penalty
}
TodoCategory::IncompleteImplementation => {
    score *= 0.3; // 70% penalty
}

// ACCEPTED - lighter penalty for documentary
TodoCategory::FutureImprovement => {
    score *= 0.9; // 10% penalty
}
TodoCategory::TemporarySolution => {
    score *= 0.8; // 20% penalty
}

// Explicit override: if score drops below 40%, fail regardless
if final_score < 0.4 {
    return ReviewDecision::REJECT("Council score below threshold");
}
```

### 2. CI/CD Gate (`.github/workflows/lint.yml`)

```yaml
name: Code Quality Gates

on: [push, pull_request]

jobs:
  critical-placeholder-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Scan for critical TODOs & placeholders
        run: |
          CRITICAL_PATTERNS=(
            "// PLACEHOLDER:"
            "// FIXME:"
            "unimplemented!()"
            "todo!()"
            'throw new Error("TODO'
            "raise NotImplementedError"
            "assert False.*TODO"
          )
          
          FOUND=0
          for pattern in "${CRITICAL_PATTERNS[@]}"; do
            if grep -r "$pattern" \
              iterations/v3/apps/*/src/ \
              iterations/v3/workers/src/ \
              iterations/v3/database/src/ \
              iterations/v3/orchestration/src/ \
              2>/dev/null; then
              FOUND=$((FOUND + 1))
            fi
          done
          
          if [ $FOUND -gt 0 ]; then
            echo "ERROR: Found $FOUND critical placeholder(s)"
            exit 1
          fi
          echo "✅ No critical placeholders"
  
  documentary-todo-summary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Count documentary TODOs (for metrics only)
        run: |
          # Count TODOs that include context/requirements
          GOOD_TODOS=$(grep -r "// TODO:.*\(Requirements\|- \[\|For now\|will replace\)" \
            iterations/v3/ 2>/dev/null | wc -l)
          
          # Count bare TODOs (might need review)
          BARE_TODOS=$(grep -r "// TODO: $" \
            iterations/v3/ 2>/dev/null | wc -l)
          
          echo "TODO Inventory:"
          echo "- Well-documented: $GOOD_TODOS"
          echo "- Bare TODOs: $BARE_TODOS"
          
          if [ "$BARE_TODOS" -gt 20 ]; then
            echo "⚠️  WARNING: High bare TODO count; consider adding context"
          fi
```

### 3. Council Scoring Update

```rust
// iterations/v3/council/src/todo_analyzer.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TodoCategory {
    // CRITICAL - never acceptable
    Explicit,                    // "// TODO: ..."
    PlaceholderCode,            // "// PLACEHOLDER: ..."
    CodeStub,                   // "// TODO: Implementation stub"
    IncompleteImplementation,   // "// TODO: Replace with real impl"
    
    // ACCEPTED - documentary, not blocking
    TemporarySolution,          // "// TODO: Replace temporary solution"
    FutureImprovement,          // "// TODO: Optimize when load-tested"
    HardcodedValue,             // "// TODO: Make configurable"
    
    // Unknown
    Unknown,
}

impl TodoCategory {
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            TodoCategory::Explicit
                | TodoCategory::PlaceholderCode
                | TodoCategory::CodeStub
                | TodoCategory::IncompleteImplementation
        )
    }
    
    pub fn penalty_multiplier(&self) -> f32 {
        if self.is_critical() {
            0.2  // 80% penalty → likely fails overall score
        } else {
            0.85 // 15% penalty → acceptable in documented form
        }
    }
}
```

### 4. Enforcement Phase (Gradual)

**Phase 1 (Weeks 1-2):** Report only
```
INFO: Critical placeholders that will be enforced next week:
- iterations/v3/workers/src/executor.rs:53: unimplemented!()
- iterations/v3/database/src/query.rs:100: // PLACEHOLDER: SQL parsing

→ Fix by end of week or create tracked waiver
```

**Phase 2 (Week 3+):** Hard fail
```
ERROR: Critical placeholder found; cannot merge
- iterations/v3/workers/src/executor.rs:53: unimplemented!()

Options:
1. Fix it now
2. Move to docs/archive/ (if exploratory)
3. Request waiver (requires approver + rationale)
```

---

## Good TODO Examples (Allowed)

### Clear requirements & timeline:
```rust
/// TODO: Implement actual Core ML output extraction and post-processing
/// Requirements:
/// - [ ] Parse MLMultiArray for different data types
/// - [ ] Handle batched outputs correctly
/// - [ ] Support variable-length sequences
///
/// Timeline: Q2 2025 (when Core ML integration is prioritized)
fn format_coreml_output(&self, output: &MLMultiArray) -> Result<OutputTensor> {
    // Current: Simplified single-sample path works for MVP
    Ok(OutputTensor { /* ... */ })
}
```

### Known limitation with workaround:
```rust
// TODO: Replace simplified seasonal pattern detection with proper statistical analysis
// For now, using basic frequency counting which covers 80% of use cases.
// Will integrate FFT-based spectral analysis when performance is critical.

fn detect_seasonal_patterns(&self, data: &[f64]) -> Vec<Period> {
    // Working implementation that's good enough for current load
}
```

### Performance optimization placeholder:
```rust
// TODO: Implement model caching with LRU eviction and persistence
// Currently loading model from disk each time (works but slow).
// Will add in-memory cache when benchmarks show bottleneck.

async fn load_model(&self, path: &Path) -> Result<Model> {
    tokio::fs::read(path).await?  // Works, just not cached
}
```

---

## Bad TODO Examples (Must Fix)

### Incomplete stub with no explanation:
```rust
// TODO: Implement feature
fn process_data(&self) -> Result<Vec<String>> {
    Ok(vec![])  // Empty!
}
```
**Why bad:** No context, will fail in production.  
**Fix:** Either implement or mark as `// PLACEHOLDER:` and remove from scope.

### Silent failure:
```rust
// TODO: Add error handling
async fn save_to_database(&self, data: &Data) -> Result<()> {
    // Silently does nothing
    Ok(())
}
```
**Why bad:** Won't error, will lose data silently.  
**Fix:** Add actual implementation or throw error immediately.

### Broken on purpose:
```rust
async fn validate_schema(&self) -> bool {
    // TODO: Implement proper validation
    panic!("Not implemented");  // Runtime crash!
}
```
**Why bad:** Crashes in production.  
**Fix:** Implement validation or return error result.

---

## Migration Path (Existing TODOs)

### Step 1: Audit (Week 1)
```bash
rg "// TODO:|// PLACEHOLDER:|unimplemented!|todo!" \
  iterations/v3/src/ \
  iterations/v3/apps/*/src/ \
  | sort -u > /tmp/all_todos.txt

# Categorize by hand (or use python analyzer)
```

### Step 2: Triage (Week 2)
- Move "aspirational" to `docs/archive/`
- Add requirements to documentary TODOs
- Flag critical ones for immediate fix

### Step 3: Enforce (Week 3+)
- Enable CI gate
- Council scoring reflects categories
- Track metrics on resolution rate

---

## Metrics to Track

1. **Critical placeholder count:** Should trend to 0
2. **Documentary TODO count:** Stable (expected in complex systems)
3. **Council score impact:** By category and milestone
4. **Time to TODO resolution:** How long before items are addressed

---

## FAQ

**Q: Can I have a TODO in production code?**  
A: Only if it's well-documented (context, requirements, timeline) and the code works anyway. Critical placeholders (stubs, unimplemented) must be removed or moved to `docs/archive/`.

**Q: What's the difference between // TODO and // PLACEHOLDER?**  
A: `TODO` = future work on working code. `PLACEHOLDER` = incomplete stub that doesn't work. Use PLACEHOLDER for anything that would crash or silently fail.

**Q: How do I defer work to P1 without TODOs?**  
A: Move exploratory code to `docs/archive/experiments/` or create a separate `iterations/phase-2/` tree. Keep src/ clean.

**Q: Can I lower my council score to ship a TODO?**  
A: No. Critical TODOs fail CI, period. Documentary TODOs lower score by ~15% (acceptable if overall score > 50%).

**Q: When should I use waiver system?**  
A: When you have a critical TODO but can justify it (e.g., "Core ML integration deferred to Q2, using CPU fallback for now with clear latency impact"). Requires approver signature.

