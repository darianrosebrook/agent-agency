# Test Hang Investigation Report

## Problem Summary

Tests for `agent-constitutional-council` were hanging and not exiting correctly. Investigation revealed the root cause.

## Root Cause

The test processes were hanging due to a **missing Swift runtime library** (`libswift_Concurrency.dylib`). This occurs because:

1. **Dependency Chain**:
   - `agent-constitutional-council` → `agent-orchestration`
   - `agent-orchestration` → `system-acceleration`  
   - `system-acceleration` → `coreml-rs` (always included on macOS)

2. **Swift Runtime Requirement**:
   - `coreml-rs` requires Swift runtime libraries
   - Even though tests use `MockJudgeEngine` (no actual CoreML), the dependency is still linked
   - The test binary tries to load `@rpath/libswift_Concurrency.dylib` at startup
   - Library cannot be found in expected paths, causing the process to hang

3. **Evidence from Logs**:
   ```
   dyld[81950]: Library not loaded: @rpath/libswift_Concurrency.dylib
   Referenced from: <...> agent_constitutional_council-92d25608c2278f32
   ```

## Affected Tests

Based on the output file `/tmp/cargo_test_output.txt`, the hang occurred when running:
- `agent-constitutional-council` unit tests
- Specifically at the point where the test binary tried to load Swift libraries

## Solutions

### Option 1: Make CoreML Dependency Optional (Recommended)

Make `coreml-rs` optional behind a feature flag in `system-acceleration`:

```toml
# In system-acceleration/Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
# Change from:
coreml-rs = "0.1"
# To:
coreml-rs = { version = "0.1", optional = true }

[features]
default = []
coreml = ["dep:coreml-rs"]  # Gate CoreML behind feature flag
```

Then update `agent-orchestration` to only enable CoreML when needed:
```toml
system-acceleration = { path = "../system-acceleration", default-features = false }
```

### Option 2: Fix Swift Runtime Path Resolution

Update `system-acceleration/build.rs` to ensure Swift runtime paths are correctly set for test binaries:

```rust
// Add test-specific rpath configuration
#[cfg(test)]
{
    // Additional paths for test binaries
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks");
}
```

### Option 3: Conditional Compilation for Tests

Use `#[cfg(not(test))]` to exclude CoreML dependencies during test builds, but this is complex and may break test coverage.

## Immediate Workaround

For now, tests can be run with:
```bash
# Set DYLD_LIBRARY_PATH to help find Swift libraries
export DYLD_LIBRARY_PATH="/usr/lib/swift:/System/Library/Frameworks:$DYLD_LIBRARY_PATH"
cargo test --package agent-constitutional-council
```

However, this is a workaround and doesn't solve the root cause.

## Recommendation

**Option 1 is recommended** because:
1. Tests don't need CoreML functionality
2. Reduces build time for test-only builds
3. Makes dependencies explicit and optional
4. Follows Rust best practices for optional features

## Next Steps

1. Make `coreml-rs` optional in `system-acceleration/Cargo.toml`
2. Update `agent-orchestration` to conditionally enable CoreML
3. Verify tests run without hanging
4. Ensure production builds still have CoreML available when needed

## Author

@darianrosebrook








