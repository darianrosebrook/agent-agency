# Core ML Integration: Technical Risk & Implementation Plan

**Author**: @darianrosebrook  
**Status**: Blocked – Requires Objective-C FFI Bridge  
**Risk Level**: HIGH  
**Priority**: Medium-term (Q2 2025+)

## Executive Summary

Core ML optimization for Apple Silicon is a high-value but high-risk capability. The system currently has **scaffolding** for model loading and optimization configuration, but **lacks the native bridge** to Core ML's Objective-C APIs. Implementing this requires shipping unsafe code with significant maintenance burden, which is why it remains incomplete.

This document outlines:
- **What exists today** (and what's missing)
- **Why it's blocked** (FFI complexity, safety concerns)
- **What's required** to unblock it (timeline, risk mitigation)
- **Alternative approaches** if the native path proves infeasible

---

## Current State: Scaffolding Without Runtime

### What We Have

```
apple-silicon/src/core_ml.rs (1,484 lines)
├── CoreMLManager         [Async trait with channel-based inference]
├── Model loading         [Path handling, cache management]
├── Optimization stubs    [Config application, placeholder compile step]
├── Benchmarking         [Timing collection, throughput measurement]
└── Type layer           [ModelInfo, InferenceRequest, OptimizationTarget]
```

**Specific placeholders:**

```rust
// Line 473: perform_core_ml_optimization_placeholder
#[cfg(target_os = "macos")]
{
    use objc2_core_ml::MLModelConfiguration;
    
    // Creates config object but doesn't compile/optimize models
    let config = match target {
        OptimizationTarget::ANE => unsafe { MLModelConfiguration::new() },
        // ...
    };
    
    // TODO: Implement Core ML model optimization
    // 1. Model loading: Load the original Core ML model
    // 2. Configuration application: Apply config to model
    // 3. Hardware compilation: Compile for ANE/GPU/CPU
    // 4. Model persistence: Save optimized version
    
    info!("Core ML optimization completed successfully");
    Ok(())
}
```

**What this means**: We can instantiate Core ML config objects, but cannot:
1. Load `.mlmodel` files from disk
2. Apply quantization/compilation settings
3. Trigger hardware-specific optimization
4. Persist optimized models
5. Handle real inference through Core ML runtime

---

## The Blocker: Objective-C FFI Complexity

### Why This Is Hard

Core ML is **not exposed through Rust bindings**. To use it, we must either:

#### Option A: Raw `objc2` (Current Status)
```rust
// What we CAN do:
use objc2_core_ml::MLModelConfiguration;
let config = unsafe { MLModelConfiguration::new() };
```

**Problems**:
- **No model loading API** in `objc2_core_ml` – would need to add it
- **No compilation API** – Core ML uses `MLModel.compileModelAtURL()`, not exposed
- **Unsafe block explosion** – Every Core ML call requires `unsafe { }`
- **Version coupling** – Apple updates Core ML APIs every macOS release
- **Testing nightmare** – Can only test on actual Apple Silicon hardware

#### Option B: Fork/Extend `objc2-coreml`
```rust
// Hypothetical:
pub struct MLModel {
    inner: objc2::runtime::Object,
}

impl MLModel {
    pub unsafe fn compile_at_url(url: &NSUrl) -> Result<Self> {
        // Need to write unsafe wrapper around:
        // MLModel::compileModelAtURL:error: selector
        // which is a class method returning (NSError**) out-param
        todo!()
    }
}
```

**Requirements**:
- **Understand Core ML C API contract** (error semantics, memory management)
- **Write ~200-300 lines of unsafe code** (wrapper structs, selector dispatch)
- **Maintain indefinitely** – Apple changes Core ML every OS release
- **Test on real hardware** – Can't mock objc2 runtime calls easily
- **Audit for soundness** – Unsafe code must never segfault or leak memory

#### Option C: Swift Bridge Layer
```swift
// CoreMLBridge.swift
import CoreML

@objc class CoreMLBridge: NSObject {
    @objc static func compileAndOptimize(
        modelPath: String,
        target: String,
        completion: @escaping (String?, Error?) -> Void
    ) {
        DispatchQueue.global().async {
            do {
                let modelURL = URL(fileURLWithPath: modelPath)
                let compiledURL = try MLModel.compileModel(at: modelURL)
                completion(compiledURL.path, nil)
            } catch {
                completion(nil, error)
            }
        }
    }
}
```

**Tradeoffs**:
- ✓ Uses Apple's **official public API** (safe, documented)
- ✓ **Simpler than raw objc2** (no unsafe blocks)
- ✗ **Requires Swift runtime** in the binary (+3-5MB, build complexity)
- ✗ **Build system coupling** – Xcode integration needed
- ✗ **Deployment friction** – CocoaPods/SPM dependency graph

---

## What Would Success Look Like?

### Minimum Viable Implementation (1-2 weeks)

```rust
pub struct CoreMLOptimizer {
    cache_dir: PathBuf,
}

impl CoreMLOptimizer {
    /// Load a .mlmodel and compile it for a specific target
    pub async fn optimize(
        &self,
        model_path: &Path,
        target: OptimizationTarget,
        quantization: Option<QuantizationMethod>,
    ) -> Result<CompiledModelPath> {
        // 1. Validate input
        let canonical_path = Self::resolve_model_path(model_path)?;
        
        // 2. Check cache
        if let Some(cached) = self.lookup_cache(&canonical_path, &target, &quantization)? {
            debug!("Using cached compiled model: {:?}", cached);
            return Ok(cached);
        }
        
        // 3. Load Core ML model
        let ml_model = unsafe {
            MLModel::load_from_url(&canonical_path)?
        };
        
        // 4. Apply optimization config
        let config = Self::build_config(&target, &quantization)?;
        ml_model.set_config(&config)?;
        
        // 5. Compile for hardware
        let compiled = unsafe {
            ml_model.compile_for_platform(target)?
        };
        
        // 6. Persist and cache
        let out_path = self.cache_compiled(&compiled)?;
        self.update_cache_index(&canonical_path, &target, &out_path)?;
        
        Ok(out_path)
    }
}
```

**Work breakdown**:
- [ ] **Unsafe wrapper structs** (40 lines) – MLModel, MLModelConfiguration
- [ ] **Selector dispatch helpers** (60 lines) – selector matching, error handling
- [ ] **Memory management** (40 lines) – NSUrl creation, autoreleasepool usage
- [ ] **Error translation** (30 lines) – NSError → anyhow::Error
- [ ] **Integration tests** (50 lines) – Test on real hardware
- [ ] **Documentation** (30 lines) – Safety invariants, usage examples

**Total**: ~250 lines of unsafe Rust + integration tests

### Quality Checkpoints

1. **No segfaults** – Fuzz with malformed inputs
2. **No memory leaks** – Profile with Instruments.app
3. **Error handling** – NSError exceptions properly translated
4. **Concurrency safety** – Send/Sync bounds verified
5. **Hardware coverage** – Test on M1, M2, M3 (different ANE capabilities)

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| **API incompatibility** | Breaks on new macOS | High | Version pin Core ML deps; add CI on beta releases |
| **Memory unsafety** | Segfault in production | Medium | Exhaustive unsafe code review; runtime assertion checks |
| **ANE availability** | Fails on Intel Mac | Low | Feature-gate code; graceful CPU fallback |
| **Compilation timeout** | Hangs on large models | Medium | Add timeout wrapper; async task cancellation |
| **Quantization loss** | Accuracy degradation | Medium | Benchmark pre/post; accept user tolerance thresholds |

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Goal**: Establish safe unsafe FFI bridge

```
├─ Research Core ML C API contract
├─ Write objc2 wrapper types (MLModel, MLModelConfiguration)
├─ Implement NSUrl/NSError translation
├─ Add basic error handling tests
└─ Performance: No regression on CPU baseline
```

**Owner**: @darianrosebrook  
**Blocker**: Must read Apple's private headers or reverse-engineer selector names

### Phase 2: Integration (Week 3-4)

**Goal**: Connect to CoreMLManager public API

```
├─ Implement MLModel::load_from_url()
├─ Implement MLModel::compile_for_platform()
├─ Wire into optimize_model() call chain
├─ Add caching layer for compiled models
└─ Performance: <5s compile time for typical models
```

**Owner**: Apple Silicon specialist  
**Dependency**: Phase 1 complete

### Phase 3: Validation (Week 5-6)

**Goal**: Comprehensive testing on real hardware

```
├─ Unit tests for wrapper safety invariants
├─ Integration tests on M1/M2/M3 hardware
├─ Fuzzing with pathological model files
├─ Benchmark optimization vs. CPU baseline
├─ Memory profiling (Instruments.app)
└─ Performance: 15-30% throughput gain vs. CPU
```

**Owner**: QA + hardware lab access  
**Risk**: Requires multiple Mac models (M1, M2, M3)

### Phase 4: Hardening (Week 7-8)

**Goal**: Production readiness

```
├─ Handle model format variations
├─ Add telemetry for optimization success rate
├─ Implement circuit breaker for Core ML failures
├─ Document safety invariants in code
├─ Add observability/metrics for compilation performance
└─ Performance: SLA <5s for 99%ile models
```

**Owner**: Platform engineer  
**Dependency**: Phase 3 passing all tests

---

## Resource Requirements

### Personnel

| Role | Effort | Skills Required |
|------|--------|-----------------|
| **Objective-C/FFI Expert** | 2-3 weeks | objc2, Apple frameworks, unsafe Rust |
| **Rust Platform Engineer** | 1-2 weeks | Async/await, error handling, profiling |
| **QA / Hardware Access** | 1-2 weeks | M1/M2/M3 testing, regression detection |
| **Documentation** | 0.5 weeks | Technical writing, safety invariants |

### Infrastructure

| Resource | Count | Notes |
|----------|-------|-------|
| **Apple Silicon Macs** | 3+ | M1 (baseline), M2, M3 (testing) |
| **Build machine** | 1 | macOS with Xcode 15+ |
| **CI/CD integration** | 1 machine-month | GitHub Actions or similar |

---

## Alternatives to Native Core ML

### Alternative 1: Pure Candle-Core (All Rust)

```rust
use candle_core::{Device, Tensor, DType};

pub async fn optimize_candle(
    model_path: &Path,
    target: OptimizationTarget,
) -> Result<OptimizedModel> {
    // Load weights from safetensors
    let tensor = Tensor::load_safetensors(model_path)?;
    
    // Quantize in pure Rust
    let quantized = match target {
        OptimizationTarget::ANE => quantize_int8(&tensor)?,
        OptimizationTarget::GPU => quantize_fp16(&tensor)?,
        OptimizationTarget::CPU => tensor,
        OptimizationTarget::Auto => tensor,
    };
    
    Ok(OptimizedModel { tensor: quantized })
}
```

**Pros**:
- ✓ 100% safe Rust – no unsafe code
- ✓ Cross-platform – works on Linux, Windows, macOS
- ✓ No Apple dependency – decouples from Core ML API churn
- ✓ Easier testing – mock quantization algorithms

**Cons**:
- ✗ **No ANE acceleration** – computation runs on CPU
- ✗ **Missed 40-60% speedup** – ANE inference 2.5-3x faster than CPU
- ✗ **Thermal/power cost** – CPU inference burns more power

**Verdict**: Good fallback; not a replacement for ANE optimization.

### Alternative 2: ONNX Runtime + CoreML Export

```rust
// Convert ONNX → CoreML offline, load pre-compiled
pub async fn load_precompiled_coreml(
    model_path: &Path, // Already .mlmodelc
) -> Result<InferenceEngine> {
    // Just load, don't compile
    let engine = load_compiled_model_fast(model_path)?;
    Ok(engine)
}
```

**Pros**:
- ✓ **Avoid Core ML compilation** – models pre-compiled offline
- ✓ **Simpler Rust code** – just load binary blobs
- ✓ **CI/CD friendly** – compile in build pipeline, not runtime

**Cons**:
- ✗ **Offline-only** – can't adapt to runtime conditions
- ✗ **Build tool dependency** – requires ONNX Runtime on build machine
- ✗ **Model versioning** – need to rebuild on Core ML API changes

**Verdict**: Good for known-good models; bad for dynamic optimization.

---

## Decision Matrix

| Approach | Safety | Maintainability | Performance | Cost | Timeline |
|----------|--------|-----------------|-------------|------|----------|
| **Native Core ML FFI** | Medium (unsafe) | Low (API churn) | **High** (ANE) | High | 6-8 weeks |
| **Pure Candle** | **High** | High | Low (no ANE) | Low | 1-2 weeks |
| **ONNX + Precompile** | High | Medium | Medium | Medium | 3-4 weeks |
| **Swift Bridge Layer** | High | Medium | High | Medium | 4-6 weeks |

---

## Recommendation

### Short Term (Now – Q1 2025)

**DO NOT implement Core ML bridging.** The scaffolding is sufficient for:
- ✓ Interface contracts (optimize_model API)
- ✓ Integration testing (mock implementations)
- ✓ Falling back to CPU inference

**Instead**:
1. Focus on Core CPU performance optimization (Candle vectorization)
2. Establish benchmark baseline
3. Document what ANE would unlock

### Medium Term (Q2 2025)

**IF** performance targets require ANE acceleration:

1. **Contract with Objective-C expert** (2-3 weeks)
2. **Build Swift bridge layer** (cleaner than raw objc2)
3. **Comprehensive hardware testing** (M1/M2/M3 coverage)
4. **Ship with feature flag** (safe rollout)

**Success criteria**:
- 30-40% throughput improvement vs. CPU
- <5s 99%ile compilation time
- Zero crashes on malformed inputs
- Documented safety invariants

### Long Term (2026+)

- **Upstream contributions** – Work with Apple/Rust team on official bindings
- **Ecosystem maturation** – As Rust-on-Apple ecosystem matures, alternatives emerge
- **Hardware updates** – New Apple Silicon capabilities may change cost/benefit

---

## References

### Apple Documentation

- [Core ML Documentation](https://developer.apple.com/documentation/coreml)
- [Core ML Python Tools](https://github.com/apple/coremltools)
- [Neural Engine Architecture](https://www.apple.com/newsroom/articles/2020/11/app-tracking-transparency-privacy-labels-on-the-app-store-and-more-coming-soon/)

### Rust Ecosystem

- [objc2 crate](https://github.com/madsmtm/objc2)
- [objc2-core-ml crate](https://docs.rs/objc2-core-ml/latest/objc2_core_ml/)
- [candle-core for pure-Rust quantization](https://github.com/huggingface/candle)

### Related Issues

- [GitHub Issue: Core ML Model Loading](./../../TODO)
- [Architecture Decision: Hardware Acceleration](./../../docs/adr/04-apple-silicon.md)

---

## Appendix: Safety Checklist

If/when implementing native Core ML FFI, use this checklist:

- [ ] All unsafe blocks justified with SAFETY comments
- [ ] No raw pointers escape public API
- [ ] All NSError out-params properly translated
- [ ] Autoreleasepool scoping around all objc2 calls
- [ ] Send/Sync bounds validated with miri
- [ ] Fuzz testing with malformed model files
- [ ] Memory profiling with Instruments.app
- [ ] Tested on M1, M2, M3 hardware
- [ ] Documented invariants and limitations
- [ ] Code review from Rust safety expert
