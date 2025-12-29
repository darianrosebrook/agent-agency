# Mutation Testing Strategy

## Current Problem

- **1991 mutants** tested across entire crate
- **Many MISSED mutants** (surviving = tests don't catch them)
- **Slow execution** (~3-5 seconds per mutant = 1.5-3 hours total)
- **No tests** for `arbiter_pipeline.rs` (explains many missed mutants)

## Root Cause

Mutation testing is running on code without adequate test coverage. Many modules have no tests at all.

## Immediate Solution: Scoped Mutation Testing

### Option 1: Test Only Modules with Tests (Recommended)

Run mutation testing only on modules that have tests:

```bash
# Test only modules with existing tests
cargo mutants --workspace \
  --file '**/encryption.rs' \
  --file '**/security.rs' \
  --timeout 300 \
  --no-shuffle \
  --baseline run
```

### Option 2: Exclude Untested Modules

Exclude modules without tests from mutation testing:

```bash
# Exclude untested modules
cargo mutants --workspace \
  --exclude-file '**/arbiter_pipeline.rs' \
  --exclude-file '**/conflict_resolution_tools.rs' \
  --timeout 300 \
  --no-shuffle \
  --baseline run
```

### Option 3: Per-Module Incremental Testing

Test one module at a time with focused tests:

```bash
# Test single module
cargo mutants --workspace \
  --file '**/encryption.rs' \
  --timeout 60 \
  --no-shuffle \
  --baseline run
```

## Long-Term Strategy

### Phase 1: Add Tests for Critical Modules (Current Priority)

1. **`arbiter_pipeline.rs`** - Core decision logic (highest priority)
   - Test `DecisionStageAdapter::process` with all stages
   - Test `ArbiterPipelineOptimizer` decision methods
   - Test risk tier classification logic
   - Test worker pool selection

2. **`conflict_resolution_tools.rs`** - Conflict resolution
   - Test conflict resolution methods
   - Test match arm coverage

### Phase 2: Incremental Mutation Testing

Once tests exist:
1. Run mutation testing on one module at a time
2. Fix tests to kill surviving mutants
3. Move to next module

### Phase 3: CI Integration

Only run mutation testing on:
- Modules with ≥80% branch coverage
- Critical paths (auth, billing, data processing)
- Before releases (not on every commit)

## Quick Fix: Disable Mutation Testing for Untested Modules

Add to `Cargo.toml` or mutation config:

```toml
# Only run mutation testing on tested modules
[package.metadata.mutants]
exclude = [
    "**/arbiter_pipeline.rs",
    "**/conflict_resolution_tools.rs",
    # Add other untested modules
]
```

## Mutation Testing Best Practices

1. **Test First**: Add tests before running mutation testing
2. **Scope Narrowly**: Test one module/file at a time
3. **Focus on Critical Paths**: Prioritize auth, billing, data processing
4. **Use Timeouts**: Prevent hanging on slow tests
5. **Baseline First**: Ensure unmutated code passes

## Recommended Workflow

```bash
# 1. Add tests for arbiter_pipeline.rs
cargo test --lib arbiter_pipeline

# 2. Run mutation testing on just that module
cargo mutants --workspace \
  --file '**/arbiter_pipeline.rs' \
  --timeout 60 \
  --no-shuffle \
  --baseline run

# 3. Fix tests to kill surviving mutants
# 4. Repeat for next module
```

## Current Status

- ✅ `encryption.rs` - Has tests, mutation testing should work
- ❌ `arbiter_pipeline.rs` - No tests, skip mutation testing
- ❌ `conflict_resolution_tools.rs` - No tests, skip mutation testing
- ❓ Other modules - Check test coverage first





















