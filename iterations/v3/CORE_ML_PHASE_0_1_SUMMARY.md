# Core ML Implementation – Phase 0-1 Summary

**Author:** @darianrosebrook  
**Date:** October 18, 2025  
**Status:** Phase 0-1 Complete, Phase 2-3 Pending

---

## Phase 0 Completion: Rust Inference Contracts

### Files Created/Modified

**`apple-silicon/src/inference.rs`** (300+ lines)
- `InferenceEngine` trait: prepare, infer, capabilities
- `PreparedModel` trait: cache_key, io_schema, sla_estimate
- `ModelArtifact` enum: Authoring, Compiled with versioned cache keys
- `ComputeUnits` enum: All, CpuOnly, CpuAndGpu, CpuAndNe (C FFI mapping)
- `CapabilityReport`: device class, ANE op coverage %, compute unit dispatch telemetry
- `DType` enum: F32, F16, I32, I8, U8
- `TensorSpec`, `IoSchema`, `PrepareOptions`, `TensorMap` types

**Key Design Invariants:**
- All tensors are row-major, dtype-explicit
- No ObjC/Swift types cross Rust public boundary
- Cache keys include OS build + Core ML version for reproducibility

### Candle Backend Reference Implementation

**`apple-silicon/src/candle_backend.rs`** (170+ lines)
- `CandleBackend` struct implementing `InferenceEngine`
- `CandleModel` struct implementing `PreparedModel`
- Safe Rust wrapper with automatic Drop handling
- Mock implementations (production code would load .safetensors via candle-core)
- 3 unit tests passing:
  - `test_candle_backend_creation` — Send+Sync traits verified
  - `test_candle_backend_capabilities` — Device class, ANE coverage, SLA
  - `test_candle_model_traits` — PreparedModel interface contract

### Module Integration

**`apple-silicon/src/lib.rs`** (updated)
- Export `inference` module
- Export `CandleBackend` from candle_backend
- Export key types: `InferenceEngine`, `CapabilityReport`, `ComputeUnits`, `DType`, etc.

---

## Phase 1 Completion: Swift C-ABI Bridge Foundation

### Swift Package Structure

**`coreml-bridge/Package.swift`** (~20 lines)
- Swift tools version 5.9+
- macOS 11+ deployment target
- Link CoreML and Foundation frameworks
- Produces static library `libCoreMLBridge.a`

### Swift C-ABI Functions

**`coreml-bridge/Sources/CoreMLBridge/CoreMLBridge.swift`** (230+ lines)

**Implemented Functions:**

1. **`coreml_compile_model`**
   - Input: model path, compute units (0=All, 1=CPU, 2=CPU+GPU, 3=CPU+ANE)
   - Output: compiled .mlmodelc directory path
   - Returns: 0 on success, 1 on failure
   - Wraps in autoreleasepool

2. **`coreml_load_model`**
   - Input: compiled directory, compute units
   - Output: opaque MLModel handle (via Unmanaged)
   - Returns: 0 on success, 1 on failure
   - Uses MLModelConfiguration for compute unit selection

3. **`coreml_model_schema`**
   - Input: model handle
   - Output: JSON describing inputs/outputs (names, dtypes, shapes)
   - Returns: 0 on success, 1 on failure
   - Introspects MLModelDescription

4. **`coreml_predict`**
   - Input: model handle, inputs JSON, timeout (ms)
   - Output: outputs JSON
   - Returns: 0 on success, 1 on failure
   - Runs MLModel.prediction via MLDictionaryFeatureProvider

5. **`coreml_free_model`**
   - Releases Unmanaged handle via Unmanaged::release()

6. **`coreml_free_cstr`**
   - Frees C strings allocated by bridge

**Safety Guarantees:**
- ✅ All exceptions caught and converted to error codes + strings
- ✅ Each call wrapped in `@autoreleasepool {}`
- ✅ Compute units enum mapping (Swift MLComputeUnits)
- ✅ Opaque pointers for ARC object lifetime management
- ✅ No ObjC types cross FFI boundary

### Build Status

```bash
$ cd coreml-bridge && swift build --configuration release
✅ Compiles successfully (warnings only, no errors)
✅ Produces libCoreMLBridge.a static library
```

---

## What's Next: Phase 2-3 Plan

### Phase 2: Rust Wrapper & Integration (~2-4 weeks)

**Tasks:**
- Create `apple-silicon/src/core_ml_bridge.rs`:
  - `extern "C"` declarations for all Swift functions
  - `CoreMLModel` safe wrapper with Drop guard
  - `with_autorelease_pool()` helper for every FFI call
  - Error translation C → Rust

- Create `apple-silicon/src/core_ml_backend.rs`:
  - Impl `InferenceEngine` for Core ML backend
  - Route `prepare()` → compile+load
  - Route `infer()` → predict with timeouts

- Create `apple-silicon/src/telemetry.rs`:
  - Counters: compile/infer success, p50/p95/p99
  - Circuit breaker: disable if <95% success or p99 > SLA
  - Device dispatch tracking (requested vs actual)

- Update `apple-silicon/build.rs`:
  - Invoke `swift build --configuration release`
  - Link `libCoreMLBridge.a` + CoreML framework
  - Handle MACOSX_DEPLOYMENT_TARGET

- Gate A Validation: 10k compile+load cycles leak-free (Instruments)

### Phase 3: Compression Lab & ANE Feasibility (~3-6 weeks)

**Tasks:**
- Create `scripts/models/` with FastViT T8 download + conversion
- Build quantization lab: FP16, INT8, 4-bit palettization variants
- Add ResNet-50, DETR, BERT-SQuAD models for comprehensive testing
- Validate ANE operator coverage with telemetry
- Gate C: ≥30% speedup vs CPU on primary model

### Phase 4: Hardening & Device Matrix (~3-5 weeks)

**Tasks:**
- MLMultiArray buffer pools per shape/dtype
- Static batching support
- Nightly device matrix (M1/M2/M3, macOS current & current-1)
- Gate D: 1-hour soak stable, documented SLOs

---

## Test Status

✅ Phase 0 Unit Tests: 3/3 passing
```
test candle_backend::tests::test_candle_backend_creation ... ok
test candle_backend::tests::test_candle_backend_capabilities ... ok
test candle_backend::tests::test_candle_model_traits ... ok
```

✅ Swift Bridge: Compiles without errors (warnings only)

---

## Abstraction Seams Summary

**Public Rust API** (stable, future-proof):
- `prepare(artifact, options) → PreparedModel`
- `infer(model, inputs, timeout) → outputs`
- `capabilities(model) → device capabilities`
- `ComputeUnits` enum (explicit device selection)
- `CapabilityReport` (requested vs actual dispatch)

**Benefits:**
- Candle CPU can be swapped with Core ML, Metal, or other backends
- Test CPU path without native code
- Feature gate Core ML without public API changes
- Telemetry and circuit breaker built into foundation

---

## Technical Highlights

1. **Cache Key Versioning:**
   ```
   {sha256}:{coreml_ver}:{backend}:{compute_units}:{quantization}:{shape_key}:{os_build}
   ```
   Ensures reproducibility and cache invalidation on toolchain updates.

2. **Autorelease Pool Discipline:**
   Every Swift FFI entry point wrapped in `@autoreleasepool {}`
   Prevents memory leaks from autoreleased intermediate objects.

3. **Error Handling:**
   - Swift exceptions → C error codes + UTF-8 error strings
   - All strings allocated with `strdup()` for C caller to free
   - No panics across FFI boundary

4. **Opaque Pointer Management:**
   `Unmanaged<AnyObject>` retains MLModel across FFI call boundaries
   Caller cannot misuse pointer (lifetime tied to handle existence).

---

## Known Limitations & Next Actions

| Aspect | Status | Next Action |
|--------|--------|------------|
| Rust Contracts | ✅ Complete | None (ready for Phase 2) |
| Swift Bridge | ✅ Scaffolding | Integrate with build.rs (Phase 2) |
| Telemetry | ⏳ Pending | Implement CoreMLMetrics (Phase 2) |
| Model Tests | ⏳ Pending | Download FastViT T8 (Phase 2) |
| ANE Validation | ⏳ Pending | Measure op coverage (Phase 3) |
| Device Matrix | ⏳ Pending | Nightly automation (Phase 4) |

---

## Checklist for Phase 2 Kickoff

- [ ] Add build.rs integration
- [ ] Create Rust FFI bindings (core_ml_bridge.rs)
- [ ] Implement core_ml_backend.rs (InferenceEngine)
- [ ] Add telemetry.rs with circuit breaker
- [ ] Download + convert FastViT T8 model
- [ ] Gate A: Run 10k compile+load leak check
- [ ] Document testing workflow (docs/CORE_ML_BRIDGE_GUIDE.md)
