# CoreML Architectural Fix Plan - Upstream-First Approach

**Date:** 2025-01-XX  
**Status:** 🚧 **IN PROGRESS** - Implementing runtime-loading architecture

---

## Executive Summary

Following upstream-first architectural guidance: **Feature flags are necessary but not sufficient**. The root cause is **static linkage to Swift/CoreML** pulling in platform runtime requirements even when CoreML is "disabled". 

**Solution**: Implement a **provider boundary with runtime-loaded CoreML plugin** instead of static linking.

---

## Root Cause Analysis

### The Real Problem

1. **Feature unification**: Cargo's feature resolver can enable CoreML across the workspace even when you think it's disabled
2. **Static linkage**: Any Swift/CoreML artifact in the link set causes tests to try loading platform runtimes
3. **Transitive dependencies**: Non-CoreML crates depending on CoreML-aware crates drag the feature in

### Why Feature Flags Alone Fail

- Feature flags control **compile-time** inclusion
- But **link-time** dependencies still pull in Swift runtime
- Tests hang because Swift libraries are referenced even if code paths aren't executed

---

## Architectural Solution

### Phase 1: Provider Boundary (Already Exists ✅)

The `JudgeEngine` trait in `agent-agency-contracts` is our provider boundary:

```rust
#[async_trait]
pub trait JudgeEngine: Send + Sync + std::fmt::Debug {
    async fn complete(&self, req: EngineRequest) -> Result<EngineResponse, EngineError>;
    fn capabilities(&self) -> EngineCaps;
}
```

**Status**: ✅ Already implemented - this is good!

### Phase 2: Runtime-Loaded CoreML Plugin (To Implement)

**Goal**: Make CoreML a **plugin**, not a **link-time dependency**.

#### Option A: Objective-C/C Shim (Recommended for immediate fix)

- Write minimal ObjC/C wrapper around CoreML (no Swift → no Swift runtime)
- Link only on macOS when feature enabled
- Zero Swift runtime risk

#### Option B: Swift dylib as Runtime Plugin (Best long-term)

- Build Swift `.dylib` exposing C ABI
- Use `libloading` to `dlopen` **only if**:
  - `target_os = macos`
  - Feature enabled
  - Environment/availability check passes
- **Zero CoreML/Swift load unless explicitly requested**

**Recommendation**: Start with Option A (ObjC shim) for immediate fix, migrate to Option B later.

### Phase 3: Cargo Hygiene (Partially Done ✅)

- ✅ `resolver = "2"` set in workspace
- ⚠️ Need to verify all crates respect it
- ⚠️ Need to ensure platform-specific deps only

### Phase 4: Runtime Selection with Fallback

```rust
pub fn select_backend() -> Box<dyn JudgeEngine> {
    // 1) Explicit override wins
    if std::env::var("INFER_BACKEND").as_deref() == Ok("coreml") {
        if let Some(b) = try_coreml() { return b; }
        eprintln!("coreml requested but unavailable; falling back");
    }

    // 2) Autodetect on macOS when enabled
    #[cfg(all(target_os = "macos", feature = "coreml"))]
    if let Some(b) = try_coreml() { return b; }

    // 3) Default CPU/ORT
    Box::new(OrtBackend::new().expect("ORT backend must be available"))
}
```

---

## Implementation Steps

### Step 1: Fix Immediate Blockers (ORT Compilation Errors)

**File**: `iterations/v3/data-infrastructure/src/embedding/provider.rs`

**Issues**:
- `commit_from_file` doesn't exist → Use `with_model_from_file` + `commit`
- `ort::Error` doesn't implement `StdError` → Wrap in `anyhow::Error`
- `Value::from_array` type issues → Convert `Array2` to `Vec` first

**Fix**: Create `ort_compat.rs` adapter module

### Step 2: Create CoreML Plugin Crate

**New crate**: `inference-coreml-plugin`

**Structure**:
```
inference-coreml-plugin/
├── Cargo.toml          # macOS-only, optional, no Swift deps
├── src/
│   ├── lib.rs          # Runtime loading logic
│   └── objc_shim.rs    # ObjC/C wrapper (or Swift dylib loader)
└── build.rs            # Build ObjC shim (no Swift)
```

**Key**: This crate **never links Swift statically**. It either:
- Uses ObjC/C directly (Option A)
- Uses `libloading` to load Swift dylib at runtime (Option B)

### Step 3: Update Dependency Graph

**Current**:
```
agent-constitutional-council
  └── agent-orchestration
      └── system-acceleration (static CoreML link)
```

**Target**:
```
agent-constitutional-council
  └── agent-orchestration
      └── inference-loader (runtime selection)
          ├── inference-core (ORT/CPU backend)
          └── inference-coreml-plugin (optional, runtime-loaded)
```

### Step 4: Test Strategy

**Unit tests** (no CoreML):
- Never depend on `inference-coreml-plugin`
- Use `MockJudgeEngine` (already exists)
- Run: `cargo test --workspace --no-default-features`

**CoreML integration tests**:
- Live in `inference-coreml-plugin/tests/`
- Gated: `#[cfg_attr(not(all(target_os="macos", feature="coreml")), ignore)]`
- Hard timeouts: Spawn process, kill after N seconds

**CI matrix**:
- Linux: `cargo test --workspace --no-default-features` ✅ Never hangs
- macOS (with Xcode): `cargo test --workspace --features coreml` ✅ Tests CoreML path

---

## Acceptance Criteria

- [ ] `cargo test --workspace --no-default-features` passes on Linux and macOS (no hangs)
- [ ] `cargo tree -e features` shows no CoreML/Swift crates unless `--features coreml` + macOS
- [ ] On macOS with `--features coreml`, `INFER_BACKEND=coreml` runs CoreML tests
- [ ] Production binary doesn't crash on Linux/Windows (no CoreML symbols linked)
- [ ] ANE metrics emitted when available; clear "using ORT/CPU" log otherwise

---

## Files to Create/Modify

### New Files
1. `iterations/v3/inference-coreml-plugin/Cargo.toml`
2. `iterations/v3/inference-coreml-plugin/src/lib.rs`
3. `iterations/v3/inference-coreml-plugin/src/objc_shim.rs` (or `dylib_loader.rs`)
4. `iterations/v3/inference-loader/Cargo.toml` (runtime selection)
5. `iterations/v3/inference-loader/src/lib.rs`

### Modified Files
1. `iterations/v3/data-infrastructure/src/embedding/provider.rs` - Fix ORT API
2. `iterations/v3/data-infrastructure/src/embedding/ort_compat.rs` - New adapter
3. `iterations/v3/system-acceleration/Cargo.toml` - Remove static CoreML link
4. `iterations/v3/system-acceleration/build.rs` - Remove Swift linking
5. All `Cargo.toml` files - Ensure `resolver = "2"`, platform-specific deps

---

## Migration Path

### Phase 1: Immediate (Today)
1. Fix ORT compilation errors (unblock builds)
2. Create `ort_compat.rs` adapter
3. Verify tests can run without CoreML

### Phase 2: Short-term (This Week)
4. Create `inference-coreml-plugin` crate
5. Implement ObjC shim (Option A) or dylib loader (Option B)
6. Update `system-acceleration` to use plugin instead of static link
7. Update dependency graph

### Phase 3: Long-term (This Month)
8. Migrate to full runtime loading architecture
9. Add capability detection and telemetry
10. Update CI/CD with proper test matrix

---

## Risk Mitigation

### If We Don't Implement This

- **Development blocked**: Tests hang indefinitely
- **Production risk**: Runtime crashes if Swift unavailable
- **Technical debt**: Problem gets worse over time

### If We Implement This

- **Development unblocked**: Tests run without CoreML
- **Production safe**: No Swift runtime required unless explicitly enabled
- **ANE acceleration preserved**: Still available when needed
- **Clean architecture**: Proper separation of concerns

---

## Author

@darianrosebrook  
**Priority**: 🔴 **P0 - CRITICAL**  
**Estimated Time**: 
- Phase 1 (immediate): 2-3 hours
- Phase 2 (short-term): 4-6 hours  
- Phase 3 (long-term): 8-12 hours







