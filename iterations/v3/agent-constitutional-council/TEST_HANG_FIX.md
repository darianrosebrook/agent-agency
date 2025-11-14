# Test Hang Fix Implementation

## Problem

Tests for `agent-constitutional-council` were hanging because:
1. Tests use `MockJudgeEngine` (no CoreML needed)
2. Dependency chain pulls in `coreml-rs` → requires Swift runtime
3. Swift runtime library (`libswift_Concurrency.dylib`) not found → test hangs

## Solution Implemented

### 1. Made CoreML Optional (Behind Feature Flag)

**`system-acceleration/Cargo.toml`**:
- `coreml-rs` is now optional: `coreml-rs = { version = "0.1", optional = true }`
- CoreML enabled by default: `default = ["coreml"]` (for production)
- Feature flag: `coreml = ["dep:coreml-rs"]`

**Why this works**:
- Production builds get CoreML by default (ANE acceleration preserved)
- Tests can disable CoreML to avoid Swift runtime requirements
- Explicit control over when CoreML is needed

### 2. Production Builds Explicitly Enable CoreML

**`agent-orchestration/Cargo.toml`**:
- CoreML enabled by default: `default = ["research", "coreml"]`
- Feature flag: `coreml = ["system-acceleration/coreml"]`
- `system-acceleration` uses `default-features = false` to allow control

**Why this works**:
- Production builds get CoreML by default (ANE acceleration preserved)
- Tests can disable CoreML via `--no-default-features`
- Clear separation: production needs CoreML, tests can skip it

### 3. Improved Swift Runtime Path Resolution

**`system-acceleration/build.rs`**:
- Swift linking only happens when `coreml` feature is enabled
- Added Swift 6.x paths for newer Xcode versions
- Added `@executable_path` rpaths for test binaries
- CoreML framework linking is conditional

**Why this works**:
- Tests without CoreML don't try to link Swift libraries
- If CoreML is enabled, better path resolution helps find Swift runtime
- Test binaries can find Swift libraries in more locations

## Running Tests

### Option 1: Disable CoreML for Tests (Recommended)

```bash
# Run tests without CoreML (no Swift runtime needed)
# This disables CoreML in agent-orchestration → system-acceleration
cargo test --package agent-constitutional-council --no-default-features
```

### Option 2: Keep CoreML Enabled (If Testing CoreML Integration)

```bash
# Run tests with CoreML (requires Swift runtime)
# This should work now with improved path resolution
cargo test --package agent-constitutional-council
```

## Production Builds

Production builds automatically get CoreML:

```bash
# Production build (CoreML enabled by default)
cargo build --release

# Explicitly enable CoreML (redundant but clear)
cargo build --release --features coreml
```

## What Tests Actually Need CoreML?

**Answer: None of the current tests**

All tests in `agent-constitutional-council` use `MockJudgeEngine`, which doesn't need CoreML. The tests verify:
- Council initialization
- Judge type enums
- Verdict label enums
- Basic functionality with mock engines

**Future CoreML Integration Tests**:
If you add tests that use real `CoreMLEngine`, those tests should:
1. Be in a separate test module (e.g., `tests/coreml_integration.rs`)
2. Be gated behind a feature flag (e.g., `#[cfg(feature = "coreml-tests")]`)
3. Require actual CoreML models to be present

## Verification

After this fix:
- ✅ Tests can run without Swift runtime (using `--no-default-features`)
- ✅ Production builds get CoreML by default (ANE acceleration preserved)
- ✅ Clear separation between test and production dependencies
- ✅ Better Swift runtime path resolution if CoreML is enabled

## Author

@darianrosebrook

