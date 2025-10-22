<!-- 67bf117d-25d3-4406-a04b-e26b127d7302 c3932011-2e3e-4cf7-9f9d-bc728d6b4b69 -->
# Fix Workspace Compilation - Quick Baseline

## Phase 1: Establish Working Baseline (1a - Quick fix)

**Goal**: Get `cargo check` passing by temporarily disabling broken crates.

### Step 1.1: Identify and Disable Broken Crates

Currently failing to compile:

- `claim-extraction` - 307 errors
- `apple-silicon` - 161 errors  
- `production` - 30 errors
- `planning-agent` - 15 errors
- `mcp-integration` - 17 errors
- `source-integrity` - 4 errors

**Action**: Comment out these 6 crates in `iterations/v3/Cargo.toml` with tracking comments:

```toml
# TEMPORARILY DISABLED FOR BASELINE - RE-ENABLE PRIORITY ORDER:
# "claim-extraction",  # Priority 1: 307 errors - needed by model-benchmarking
# "apple-silicon",  # Priority 2: 161 errors - torch integration in progress
# "production",  # Priority 3: 30 errors
# "mcp-integration",  # Priority 4: 17 errors
# "planning-agent",  # Priority 5: 15 errors  
# "source-integrity",  # Priority 6: 4 errors (easiest to fix)
```

### Step 1.2: Remove Disabled Crate Dependencies

Check and comment out any dependencies on these crates in:

- `iterations/v3/*/Cargo.toml` files that reference them
- Feature flags that include them

### Step 1.3: Verify Baseline

Run `cargo check` to confirm workspace compiles with remaining enabled crates.

## Phase 2: Fix Apple Silicon Torch Integration (2a - Continue conditional compilation)

**Goal**: Complete the torch feature flag work you started in `apple-silicon/src/memory/manager.rs`.

### Step 2.1: Wrap All Torch Usage

Files needing `#[cfg(feature = "with_torch")]` guards:

1. **`apple-silicon/src/memory/manager.rs`** (partially done):

   - Wrap remaining `tch::` calls
   - Add non-torch fallback implementations for all torch functions

2. **`apple-silicon/src/quantization.rs`**:

   - Wrap `quantize_pytorch_model()` function
   - Wrap PyTorch format detection

3. **`apple-silicon/src/types.rs`**:

   - Make `ModelFormat::PyTorch` conditional or add runtime check

### Step 2.2: Fix Unsafe Function Calls

Address 161 compilation errors in apple-silicon:

- Wrap unsafe Metal calls in `unsafe` blocks
- Fix type mismatches from torch/non-torch code paths
- Add missing imports when torch is disabled

### Step 2.3: Test Apple Silicon

```bash
# Test without torch
cargo check -p agent-agency-apple-silicon --no-default-features

# Test with torch  
cargo check -p agent-agency-apple-silicon --features with_torch
```

## Phase 3: Systematic Re-enablement (3a - Comprehensive fix)

**Goal**: Re-enable each disabled crate in priority order, fixing all compilation errors.

### Step 3.1: Fix source-integrity (4 errors - easiest)

1. Check error types with `cargo check -p source-integrity 2>&1 | head -50`
2. Fix compilation errors (likely missing imports or type mismatches)
3. Re-enable in workspace Cargo.toml
4. Verify with `cargo check`

### Step 3.2: Fix planning-agent (15 errors)

1. Analyze errors: `cargo check -p agent-agency-planning-agent 2>&1 | grep "error\["`
2. Common fixes needed:

   - Import path corrections
   - Type signature fixes
   - Missing trait implementations

3. Re-enable and verify

### Step 3.3: Fix mcp-integration (17 errors)

1. Check if errors are torch-related or structural
2. Fix import paths and type issues
3. Re-enable and verify

### Step 3.4: Fix production (30 errors)

1. Analyze error patterns
2. Fix lifetime issues, missing imports
3. Re-enable and verify

### Step 3.5: Fix apple-silicon (161 errors - after Phase 2 work)

1. Apply Phase 2 fixes
2. Test both feature configurations
3. Re-enable and verify

### Step 3.6: Fix claim-extraction (307 errors - most complex)

1. Analyze error categories (likely enum variants, type mismatches)
2. Fix in logical groups (imports → types → functions)
3. May require multiple iterations
4. Re-enable and verify

### Step 3.7: Final Verification

```bash
cargo check --workspace
cargo test --workspace --no-fail-fast
```

## Tracking Document

Create `iterations/v3/DISABLED_CRATES_TRACKER.md`:

```markdown
# Temporarily Disabled Crates Tracker

## Current Status: [DATE]

### Disabled for Baseline
- [ ] claim-extraction (307 errors) - Priority 1
- [ ] apple-silicon (161 errors) - Priority 2  
- [ ] production (30 errors) - Priority 3
- [ ] mcp-integration (17 errors) - Priority 4
- [ ] planning-agent (15 errors) - Priority 5
- [ ] source-integrity (4 errors) - Priority 6

### Previously Disabled (Still Disabled)
- [ ] self-prompting-agent (177 errors + TODOs)
- [ ] integration-tests (unknown errors)
- [ ] api-server (117 errors)

### Re-enabled Successfully
- [x] council
- [x] resilience  
- [x] observability
[... list all currently enabled crates ...]

## Next Actions
1. Fix source-integrity (easiest)
2. Work up priority list
3. Track error reduction progress
```

## Success Metrics

**Phase 1 Complete**: `cargo check` passes with 6 crates temporarily disabled

**Phase 2 Complete**: apple-silicon compiles with and without torch feature

**Phase 3 Complete**: All 6 disabled crates re-enabled and workspace compiles

## Estimated Timeline

- Phase 1: 15 minutes
- Phase 2: 1-2 hours (torch integration is complex)
- Phase 3: 2-4 hours (depends on error types)

Total: 3-7 hours for complete fix

### To-dos

- [ ] Disable 6 broken crates in Cargo.toml with priority tracking comments
- [ ] Remove/comment out dependencies on disabled crates in other Cargo.toml files
- [ ] Run cargo check to verify workspace compiles
- [ ] Complete torch guards in apple-silicon/src/memory/manager.rs
- [ ] Add torch guards to apple-silicon/src/quantization.rs
- [ ] Fix 161 compilation errors in apple-silicon (unsafe blocks, types)
- [ ] Test apple-silicon with and without torch feature
- [ ] Fix 4 errors in source-integrity and re-enable
- [ ] Fix 15 errors in planning-agent and re-enable
- [ ] Fix 17 errors in mcp-integration and re-enable
- [ ] Fix 30 errors in production and re-enable
- [ ] Apply Phase 2 fixes and re-enable apple-silicon
- [ ] Fix 307 errors in claim-extraction and re-enable
- [ ] Create DISABLED_CRATES_TRACKER.md with current status
- [ ] Run cargo check and cargo test on full workspace