# Core ML Implementation Plan — Consolidated Research-Aligned, ANE-Friendly, Solo-Dev Edition

## Executive Summary

Implement a **Core ML integration bridge** between Rust and Swift using a **C-ABI surface** that isolates Objective-C/ARC complexity and supports **ANE-friendly model execution**. Maintain Candle (CPU) as the numerical ground truth. Deliver leak-free compile+load, telemetry-verified inference, and end-to-end reproducibility under solo-dev constraints.

This edition merges all prior phases with the corrections and appendices: toolchain pinning, atomic bundle installs, telemetry gates, ML Program backend guidance, and enhanced safety invariants.

---

## 1. Recommended Model Set (ANE-Eligible, Apple-Documented)

### Vision (Classification, Detection, Segmentation)

**FastViT T8 / MA variants – Classification**

* Source: Apple Model Zoo (`.mlpackage` + published latency tables).
* ANE-eligible: ≈80% ops (FP16).
* Speedup: 2.5–3.5× vs CPU (M1–M3).
* Size: 3–8 MB (FP16), 2–5 MB (INT8).
* Use: Primary iteration loop & device dispatch validation.
* **Priority:** Phase 0-1.

**ResNet-50 (FP16 + INT8)**

* Source: Apple Model Zoo.
* ANE-eligible: ≈65% ops.
* Speedup: 2–4× vs CPU.
* Use: Compression lab & CPU/Core ML parity.
* **Priority:** Phase 2-3.

**DETR (ResNet-50) – Segmentation**

* Source: Apple Model Zoo.
* ANE-eligible: Partial (attention/linear).
* Use: Transformer vision & memory stress.
* **Priority:** Stretch (3+).

**Depth Anything v2 (FP16)**

* Source: Apple Model Zoo.
* ANE-eligible: Yes.
* Use: Shape sensitivity, ML Program validation.
* **Priority:** Stretch.

**BERT-SQuAD (FP16)**

* Source: Apple Model Zoo.
* ANE-eligible: Partial (linear/attention).
* Size: ≈40 MB.
* Use: NLP path & dispatch validation.
* **Priority:** Phase 2.

**DistilBERT (ANE reference)**

* Source: Apple `ml-ane-transformers`.
* Use: Guide custom transformer authoring; target ≥30% speedup via ANE patterns.
* **Priority:** Reference (post-3).

**Stretch (Experimental)**  Whisper tiny/small (audio); Candle CPU baseline only.

---

## 2. Engineering Foundations

### Compute Unit Policy

Expose four compute targets: `All`, `CpuOnly`, `CpuAndGpu`, `CpuAndNe`.
Telemetry must log *requested vs actual* dispatch and selected backend (ML Program or NeuralNetwork).

### ML Program vs NeuralNetwork Backends

Use **ML Program** wherever possible — supports more ops, better ANE fusion, no custom layers.
Conversion scripts explicitly select backend; manifests record backend type.

### Quantization Strategy

* Begin FP16; then INT8 linear; finally 4-bit palettized (weight palette).
* Track L∞ / RMSE deltas per precision.

### Autorelease & Memory Discipline

* Wrap each Swift bridge entrypoint with `@autoreleasepool {}`.
* For long-running inference, add thread-local pool flush in Rust.
* Leak budget: <100 KB / 100 inferences; verified by Instruments.

### Timeout & Cancellation

Wrap inference in cancellable Swift Task.
On timeout: abort worker thread, log error, rebuild model.
Never allow Rust async future to block indefinitely.

### Cache & Atomic Install

`.mlmodelc` and `.mlpackage` are directories.
Compile → temp dir → `fsync` → atomic rename.
Cache key = `{sha256}:{coreml_ver}:{backend}:{compute_units}:{quant}:{shape}`.
Include OS build + Core ML runtime version.

---

## 3. Implementation Phases

### Phase 0 — Foundations (1–2 wks)

1. Create `apple-silicon/src/inference.rs`:

   * Traits: `InferenceEngine`, `PreparedModel`.
   * Enums: `ModelArtifact`, `ComputeTarget`.
   * Struct: `CapabilityReport`.
   * Cache logic and SHA256 helpers.
2. Implement `candle_backend.rs` as reference.
3. Unit tests (mocked types, Send+Sync).
   **Gate 0:** Build + tests pass; API frozen.

### Phase 1 — Swift Bridge & Compile/Load (2–3 wks)

1. SPM package `coreml-bridge` → `libCoreMLBridge.a`; link CoreML + Foundation.
2. C ABI functions:

```c
coreml_compile_model(path, units, out_dir, out_err);
coreml_load_model(compiled_dir, units, out_handle, out_err);
coreml_model_schema(handle, out_schema_json, out_err);
coreml_free_model(handle);
coreml_free_cstr(s);
```

3. Update `build.rs` to invoke `swift build –c release –Xswiftc -static-stdlib`.
4. Smoke test FastViT T8; verify leak-free in Instruments.
   **Gate A:** 10 k compile+load, no growth, cache hit ≥90%.

### Phase 2 — Inference MVP & Telemetry (2–4 wks)

1. Add `coreml_predict` C ABI (+ timeout).
2. Rust bindings `core_ml_bridge.rs` + safe `CoreMLModel` wrapper (autorelease guard).
3. Implement `core_ml_backend.rs` (prepare/infer routing).
4. Telemetry (`telemetry.rs`): compile/infer counters, p50/p95/p99, ANE dispatch %, memory high-water.
5. Parity harness: Candle vs Core ML CPU/All.
   **Gate B:** Correct output schema; timeout works; p99 ≤20 ms (FastViT M2); L∞≤1e-2 FP16, ≤5e-2 INT8.

### Phase 3 — Compression Lab & ANE Feasibility (3–6 wks)

1. Quantization lab (FP16, INT8, palettized).
2. Static shape enforcement + operator audit post-conversion.
3. Integrate ResNet-50, DETR, BERT-SQuAD.
4. Telemetry records ANE/GPU dispatch percent.
   **Gate C:** ≥30% speedup vs CPU on at least one primary model; ANE dispatch confirmed.

### Phase 4 — Hardening & Device Matrix (3–5 wks)

1. Pinned `MLMultiArray` pools; optional mmap.
2. Small `MLModel` instance pool (size ≤4).
3. Nightly device matrix (M1–M3; macOS current/current-1).
   **Gate D:** 1 h soak stable; no leaks; Core ML default enabled by telemetry.

---

## 3a. Public Rust API & Data Plumbing

### Stable Rust API Seam (`apple-silicon/src/inference.rs`)

```rust
pub enum ComputeUnits { All, CpuOnly, CpuAndGpu, CpuAndNe }

pub enum ModelFmt { Onnx, Safetensors, TorchScript, MlPackage }

pub struct ModelArtifact {
  pub format: ModelFmt,
  pub path: PathBuf,
  pub sha256: [u8; 32],
}

pub struct CompiledMeta {
  pub platform: String,      // "macos-m1", "macos-m2", etc.
  pub coreml_version: String,
  pub backend: String,       // "mlprogram" or "neuralnetwork"
}

pub trait InferenceEngine: Send + Sync {
  fn prepare(&self, artifact: &ModelArtifact, opts: PrepareOptions) 
    -> Result<Box<dyn PreparedModel>>;
  fn infer(&self, mdl: &dyn PreparedModel, inputs: &TensorMap, 
           timeout: Duration) -> Result<TensorMap>;
  fn capabilities(&self, mdl: &dyn PreparedModel) -> CapabilityReport;
}

pub struct CapabilityReport {
  pub device_class: String,              // "M1", "M2", "M3"
  pub supported_dtypes: Vec<DType>,
  pub max_batch_size: usize,
  pub ane_op_coverage_pct: u32,          // % of model ops supported on ANE
  pub compute_units_requested: ComputeUnits,
  pub compute_units_actual: ComputeUnits,  // reported by telemetry
  pub compile_p99_ms: u64,
  pub infer_p99_ms: u64,
}

pub cache_key = format!("{sha256}:{coreml_ver}:{backend}:{compute_units}:{quantization}:{shape_key}");
```

**Invariants:**
* All tensors are **row-major**, dtype-explicit; backends adapt internally.
* No ObjC/Swift types cross the Rust public boundary.
* Cache key includes OS build + Core ML runtime version for reproducibility.

### Data Plumbing Strategy

**Phase 2–3: Copy-Based (Safe, Measurable)**
* Inputs/outputs via JSON envelopes describing shape/dtype/values
* For large tensors: embed reference to temp file or mmap region
* Reuse `MLMultiArray` buffers per shape/dtype to avoid allocator churn
* Record per-stage timings: marshal_in → predict → marshal_out

**Phase 4 (Future): Fast Paths**
* **CVPixelBuffer fast path** for image models (zero-copy from cameras/Metal textures)
* **mmap for large outputs** (geometry, video frames)
* Conditional on shape/dtype compatibility; fall back to copy if unsupported

---

## 3b. Swift Micro-Bridge C ABI Signatures

**Core functions (v1):**

```c
// Compile model from source (authoring) to cached .mlmodelc bundle
int32_t coreml_compile_model(
  const char* model_path,           // path to .mlmodel, .mlpackage, or intermediate
  int32_t compute_units,            // 0=All, 1=CpuOnly, 2=CpuAndGpu, 3=CpuAndNe
  const char** out_compiled_dir,    // OUT: path to .mlmodelc directory
  const char** out_err);            // OUT: error string (or NULL)

// Load pre-compiled model into memory
int32_t coreml_load_model(
  const char* compiled_dir,         // path to .mlmodelc
  int32_t compute_units,
  void** out_handle,                // OUT: opaque model handle
  const char** out_err);

// Query model schema (input/output shapes, dtypes)
int32_t coreml_model_schema(
  void* handle,
  const char** out_schema_json,     // OUT: JSON schema
  const char** out_err);

// Single inference with timeout enforcement
int32_t coreml_predict(
  void* handle,
  const char* inputs_desc_json,     // JSON: {"input0": {...}, "input1": {...}}
  const char** out_outputs_desc_json,  // OUT: JSON results
  int32_t timeout_ms,               // timeout for entire inference
  const char** out_err);

// Cleanup
void coreml_free_model(void* handle);
void coreml_free_cstr(const char* s);
```

**Rules:**
* Swift absorbs all exceptions; return codes + UTF-8 error strings only
* Each call wrapped in `@autoreleasepool {}` on Swift side
* Timeout enforced via cancellable Swift Task
* Inputs/outputs may reference temp files for large tensors

---

## 3c. Cache & Atomic Install Strategy

* `.mlmodelc` and `.mlpackage` are **directories**, not files
* Compile workflow: **source → temp dir (atomic namespace) → fsync → rename** into cache
* Cache key: `{authoring_sha256}:{coreml_version}:{backend}:{compute_units}:{quantization}:{shape_key}:{os_build}`
* **LRU + version mismatch eviction** (log each eviction for observability)
* **Startup cleanup**: check for stale `.mlmodelc` bundles; remove if version/key mismatch

---

## 4. Telemetry & Circuit Breaker

* Counters for compile/load/predict timings, success rates.
* Breaker triggers if success < 95% or p99 > SLA N times.
* Fallback to Candle; log event.
* Also trip on memory high-water.
* Include failure-mode taxonomy: compile error, schema mismatch, timeout, mem_pressure, fallback.

---

## 5. Conversion & Manifests

Scripts under `scripts/models/` with manifest schema:

```json
{
  "model": "fastvit_t8",
  "backend": "mlprogram",
  "precision": "fp16",
  "quantization": "none",
  "compute_units": "cpuandne",
  "ane_op_coverage_pct": "TBD",
  "expected_speedup_m2": 3.0,
  "accuracy_delta_l_inf": 0.001,
  "accuracy_delta_rmse": 0.0005
}
```

Conversion scripts perform FP16 → INT8 → 4-bit sweeps and export calibration stats.

---

## 6. Testing & Validation

Manual (no CI):

```bash
cargo test --lib
cd coreml-bridge && swift build
make download-models
cargo run --example core_ml_smoke_test --features coreml
```

Run Instruments → Allocations → 10 k iterations < 10 MB growth.

**Gate A–D** implemented as Rust functions returning typed verdicts for automated regression checks.

---

## 6a. Detailed Manual Testing Workflow

**After Phase 0:**
```bash
cd apple-silicon && cargo test --lib
```
Verify trait contracts and Candle backend.

**After Phase 1:**
```bash
cd coreml-bridge && swift build
cargo build --features coreml
```

**After Phase 2:**
```bash
make download-models
cargo test --features coreml --lib
cargo run --example core_ml_smoke_test --features coreml
# Monitor Instruments.app → Allocations: 1000 iterations, expect <10MB growth
```

**Gate A Leak-Check:**
* Open Xcode, attach Instruments to binary
* Run scheme with COREML_LEAK_TEST=1000
* Watch Allocations; steady state after 100 warmups
* Pass if growth < 100KB per 100 inferences (< 1MB per 10k cycles)

**Gate B Correctness Validation:**
* Confirm single-IO inference runs without panic
* Output schema matches expected types
* Timeout enforcement kills stuck inference within configured timeout
* p99 latency < target SLA (FastViT: target ~20ms on M2)
* Parity metrics within L∞/RMSE thresholds
* Circuit breaker never triggers in 1-hour soak test

---

## 6b. Model Acquisition & Conversion Scripts

Create `scripts/models/` directory with:

**download_fastvit.py**
- Fetch FastViT T8 from Apple Model Zoo or torchvision
- Convert to Core ML (target ML Program backend)
- Produce FP16 variant
- Save to `tests/fixtures/models/fastvit_t8.mlmodel`

**convert_resnet50.py**
- Fetch ResNet-50 from Apple or torchvision
- Produce FP16 + INT8 (coremltools quantization)
- Save variants

**manifest.json template** (per converted model):
```json
{
  "model": "fastvit_t8",
  "source": "Apple Model Zoo",
  "backend": "mlprogram",
  "io_schema": { "inputs": [...], "outputs": [...] },
  "shapes": { "batch": [1], "height": 224, "width": 224 },
  "precision": "fp16",
  "quantization": "none",
  "expected_speedup_m1": 2.8,
  "expected_speedup_m2": 3.1,
  "expected_speedup_m3": 3.2,
  "ane_op_coverage_pct": 78,
  "accuracy_delta_l_inf": 0.0001,
  "accuracy_delta_rmse": 0.00005
}
```

**Makefile target:**
```makefile
.PHONY: download-models
download-models:
	cd scripts/models && python3 download_fastvit.py && python3 convert_resnet50.py
```

---

## 6c. Files & Module Map (Incremental Delivery)

* `apple-silicon/src/inference.rs` — traits, types, CapabilityReport, cache logic
* `apple-silicon/src/candle_backend.rs` — CPU reference implementation
* `coreml-bridge/` — SPM package (C ABI surface)
* `coreml-bridge/Sources/CoreMLBridge.swift` (~400 lines) — Swift implementation
* `apple-silicon/src/core_ml_bridge.rs` — extern "C" bindings + safe wrappers + Drop guards
* `apple-silicon/src/core_ml_backend.rs` — Core ML InferenceEngine implementation
* `apple-silicon/src/telemetry.rs` — counters, histograms, circuit breaker, failure taxonomy
* `scripts/models/*.py` — conversion/download scripts + manifest generation
* `tests/core_ml_integration.rs` — manual/integration tests (150+ lines)
* `tests/fixtures/models/` — .mlmodel/.mlmodelc compiled bundles

---

## 7. Documentation Artifacts

* `docs/CORE_ML_BRIDGE_GUIDE.md` — build/run/troubleshoot, toolchain pinning.
* `docs/CORE_ML_MODELS.md` — model table (size, precision, expected speedups, ANE dispatch).
* `docs/CI_SETUP_FOR_COREML.md` — self-hosted Mac runner (skeleton).
* `scripts/models/README.md` — conversion instructions & known coremltools quirks.

---

## 8. Safety & Invariants

* All FFI calls within autorelease pool.
* No ObjC types cross Rust boundary.
* Timeouts everywhere.
* Error translation C→Rust (no panic).
* Versioned cache keys.
* Circuit breaker and memory guard tested locally.
* Toolchain pinned (Xcode/Swift ver documented).
* Per-thread pool discipline commented in code.

---

## 8b. Safety & Invariants Enforcement Checklist

**Verified during code review + local testing (reference CORE_ML_IMPLEMENTATION_PATH.md § 10):**

- [ ] Every FFI call wrapped in `with_autorelease_pool { }`
- [ ] No ObjC types escape public Rust API boundary
- [ ] Timeout enforcement on all inferences (no indefinite blocks)
- [ ] Error translation C → Rust (no panics across boundary)
- [ ] Numeric parity budget defined and documented (L∞, RMSE per model)
- [ ] Model bundles treated as directories (.mlmodelc atomic moves via temp dir)
- [ ] Per-thread autorelease pool discipline documented in code comments
- [ ] Circuit breaker logic tested locally before production use
- [ ] Compute units: requested vs actual dispatch recorded in telemetry
- [ ] ML Program backend chosen for transformers (validates no custom ops allowed)
- [ ] Quantization variants track accuracy deltas explicitly (FP16 vs INT8 vs palettized)
- [ ] Cache key includes OS build + Core ML runtime version
- [ ] Memory high-water tracked and triggers circuit breaker on overflow
- [ ] All gates (A/B/C/D) automated as Rust verification functions

---

## 8c. Feature Flag & Circuit Breaker Strategy

**Runtime isolation (no CI runners needed):**

1. Feature gate: `cargo build --features coreml` (macOS only)
2. Default: CPU only (Candle backend, enabled always)
3. Runtime circuit breaker logic:

   * Track first 10 inferences on startup
   * If >5% failure → disable Core ML, fall back to Candle
   * Log reason + telemetry event
   * Manual re-enable only (config or env var: `COREML_FORCE_ENABLE`)

4. Safety: Circuit breaker logic tested locally in all builds (even CPU-only)

---

## 9. Acceptance Budgets

* **Parity:** FP16 L∞≤1e-2 RMSE≤1e-3; INT8 L∞≤5e-2 RMSE≤5e-3.
* **Compile p99:** ≤5 s (small), ≤10 s (large).
* **Relative Gate:** Core ML p99 ≤ 0.7× CPU p99 for enablement.
* **Leak Budget:** <100 KB / 100 inferences; steady after 10 k.

---

## 10. Timeline

| Phase | Weeks | Milestone                                |
| ----- | ----- | ---------------------------------------- |
| 0     | 1-2   | Rust contracts + Candle baseline stable  |
| 1     | 3-5   | Swift bridge builds; Gate A passes       |
| 2     | 6-9   | Inference MVP + telemetry; Gate B passes |
| 3     | 10-15 | Compression lab; ANE feasibility         |
| 4     | 16+   | Hardening; Gate D passes                 |

---

## 11. Risk Register & Mitigations

| Risk                   | Mitigation                                     |
| ---------------------- | ---------------------------------------------- |
| API churn              | Swift bridge isolates Rust; pin Xcode.         |
| Unsupported ops        | Detect fallbacks; log dispatch; CPU parity.    |
| Autorelease leaks      | Per-call/thread pools; Instruments validation. |
| Timeout hangs          | Cancellable task + worker abort.               |
| Cache corruption       | Atomic rename + startup cleanup.               |
| Memory pressure        | Breaker monitors RSS; auto-fallback.           |
| Batch ANE inefficiency | Static shapes preferred; telemetry validation. |

---

## 12. Immediate Next Steps

1. Implement `InferenceEngine` + Candle backend + tests.
2. Scaffold SPM bridge stubs (coreml_compile/load/free).
3. Integrate `build.rs` + link test.
4. Atomic cache layout for `.mlmodelc`.
5. Gate A validation (10 k compile+load).
6. Land Candle backend with unit tests verifying trait contracts & numeric parity baselines.
7. Build Swift C ABI bridge (coreml_compile_model, coreml_load_model, coreml_predict, etc.) and build.rs linking.
8. Run Gate A validation: 10k compile+load cycles leak-free under Instruments, cache hit ratio ≥90%.
9. Implement core_ml_backend.rs, telemetry.rs, circuit breaker with <95% success trigger.
10. Gate B validation: correctness, timeout enforcement, p99 ≤20ms, parity thresholds met.
11. Create scripts/models/ conversion pipelines (FastViT T8 primary, ResNet-50 baseline).
12. Document manual testing workflow in docs/CORE_ML_BRIDGE_GUIDE.md (build/run/troubleshoot steps per phase).

---

## 13. Future Enhancements

* Precompiled-only mode for reproducible releases.
* `coreml-diff` tool to compare bundle schemas.
* Golden input corpus for regression parity.
* MPSGraph backend exploration for zero-copy buffers.
* Benchmark MobileNetV3 outputs (Core ML vs Candle CPU) for numeric parity validation.
* Profile with Instruments.app to measure ANE op coverage and speedup vs CPU (target ≥30%).


---

## 13a. Phase 4 Hardening Details (Batching & Buffer Pools)

**Batching & Concurrency:**
* Static batching preferred (fixed batch shapes improve ANE efficiency)
* Small pool of `MLModel` instances (size = ANE pipeline width, typically 2–4)
* Back-pressure queue with timeouts per task
* Validate batching doesn't degrade per-sample latency

**Buffer Pool Management:**
* Maintain reusable `MLMultiArray` pools per shape/dtype
* Optional mmap I/O for large tensor outputs (images, geometry)
* Reduce allocator churn; measure before/after memory fragmentation

**Device Matrix Automation:**
* Nightly suite: compile, load, infer on M1/M2/M3
* Test macOS current & current-1 versions
* Compare vs CPU golden; fail build on regression
* Archive perf deltas for trend analysis

---

## 14. Testing Matrix & Validation Scope

* **Silicon:** M1, M2, M3
* **macOS versions:** current & current-1
* **Models:** FastViT, ResNet-50, DETR, BERT-SQuAD, Depth Anything v2
* **Shapes:** min/typical/max batch; static preferred; ragged only if supported
* **Precisions:** FP32 baseline (Candle), FP16/INT8 Core ML variants per model
* **Soak duration:** ≥ 1 hour continuous inference
* **Autorelease verification:** no growth in Instruments after warmup
* **Parity tolerance:** FP16 L∞≤1e-2 RMSE≤1e-3; INT8 L∞≤5e-2 RMSE≤5e-3

---

## 15. Expanded Future Enhancements

* **Precompiled-only mode:** Accept only `.mlmodelc` from CI build; skip runtime compile for reproducible deployments.
* **CVPixelBuffer fast path:** Zero-copy integration with cameras/Metal textures for image models (Phase 4+); conditional on shape/dtype compatibility with fallback to copy.
* **MPSGraph backend:** Explore Metal Performance Shaders as alternative backend with same `InferenceEngine` API seam; enables zero-copy Metal buffers for high-throughput scenarios.
* **Pinned buffer pools & mmap I/O:** Minimize copies for large tensors; profile memory fragmentation before/after optimization.
* **coreml-diff tool:** Compare `.mlmodelc` bundle schemas; validate model conversions across CI runs.
* **Golden input corpus:** Regression parity testing with fixed inputs across device matrix (M1/M2/M3 × macOS versions).
 

## 16. Optional Advanced Explorations

### Optional: ExecuTorch + Core ML Backend

* Useful as **comparative validation path** if you already have PyTorch 2.0 Export (PT2E) graphs
* Validates your own Core ML conversions and studies dynamic dispatch behavior (CPU/GPU/ANE)
* Not required for Phase 0-4; consider post-Phase 4 for breadth testing

### Optional: ONNX Runtime Core ML Execution Provider

* Enables **offline pre-compiles** and alternative execution chains in your CI lab
* Supports batch conversion workflows for comprehensive model coverage
* Good for validating ONNX→Core ML conversion consistency

### Optional: Community Model Ecosystems

* **Awesome CoreML Models** (GitHub) for breadth in benchmarking
* **Hugging Face model hub** exports to Core ML format (via coremltools)
* Treat all community sources as **unvetted** unless provenance is clear and matches Apple's guidance

---

## 17. Phase 5 — Device Matrix Automation & CI (2–3 weeks)

**Nightly suite infrastructure:**

* **Mac runners:** M1, M2, M3 on macOS current & current-1
* **Validation jobs:**
  * Compile each model family (FastViT, ResNet-50, DETR, BERT-SQuAD, Depth Anything v2)
  * Load into memory; verify schema matches golden
  * Run single inference per model; record p50/p95/p99
  * Compare against CPU baseline; **regression breaks build**
* **Archive:** perf deltas per device/OS; trend analysis over releases
* **Telemetry artifacts:** per-run compute-unit dispatch logs for audit

**Acceptance:**
* Nightly passes on all M1/M2/M3 combinations
* No regressions in latency (within 5% tolerance)
* Compute-unit dispatch telemetry confirms ANE/GPU when expected

---

## 18. Phase 6 — Offline Compilation & Deployment Variants (2–3 weeks)

**Precompiled mode:**

* **Build pipeline** produces `.mlmodelc` bundles offline (in CI)
* **Loader path** skips `coreml_compile_model`; loads directly from bundle
* **Use case:** reproducible deployments, known-good model versions, faster startup

**Implementation:**

* Add `PrecompiledModelPath` variant to `ModelArtifact` enum
* Loader detects variant; routes to `coreml_load_model` only (no compile step)
* Feature flag or runtime config to toggle between **runtime compile** vs **precompiled**
* A/B validation: run both paths; telemetry confirms identical results

**Acceptance:**
* Precompiled bundles load with < 2x latency vs cache hits
* Schema/parity identical to runtime-compiled variants

---

## 19. Quantization Deep Dive & Knobs

**Knobs to explore (Phase 3–4):**

* **FP16 activations/weights:** baseline; expect < 1% accuracy loss on most models
* **INT8 linear quantization:** per-channel or per-layer; trade-off accuracy vs size/speed
* **Grouped quantization:** per-block (e.g., 64-wide groups) for better accuracy retention
* **Channelwise quantization:** per-output-channel; strong for depthwise convs
* **4-bit weight palettization:** recent coremltools feature; aggressive compression with acceptable drift
* **Combined modes:** prune + quantize + palettize (e.g., INT8 weights + FP16 activations + 20% sparsity)

**Tracking per model:**

* Accuracy delta (L∞, RMSE, perplexity if NLP)
* Model size reduction (%)
* Inference speedup vs FP32 baseline
* ANE/GPU coverage % (telemetry from dispatch logs)

**Reference:** coremltools quantization overview + WWDC compression session

---

## 20. References & Supporting Docs

### Core ML & Tooling
* [Core ML Main Docs & Compute Units](https://developer.apple.com/documentation/coreml)
* [Core ML Tools Quantization Overview](https://apple.github.io/coremltools/docs-guides/source/opt-quantization-overview.html)
* [ML Programs vs Neural Networks](https://apple.github.io/coremltools/docs-guides/source/comparing-ml-programs-and-neural-networks.html)

### Model Collections & Guidance
* [Apple Machine Learning Models Catalog](https://developer.apple.com/machine-learning/models/) — FastViT, DETR, BERT-SQuAD, Depth Anything v2, ResNet-50
* [WWDC: Model Compression (Palettization/Pruning/Quantization)](https://developer.apple.com/videos/play/wwdc2023/10047/)
* [Apple Neural Engine Transformers Research](https://machinelearning.apple.com/research/neural-engine-transformers)
* [Apple ml-ane-transformers GitHub](https://github.com/apple/ml-ane-transformers)
* [Apple Vision Transformers on ANE](https://github.com/apple/ml-vision-transformers-ane)

### Community References (Experimental)
* [more-ane-transformers](https://github.com/smpanaro/more-ane-transformers) — LLM + transformer examples on ANE
* [Awesome CoreML Models](https://github.com/likedan/Awesome-CoreML-Models) — breadth of community conversions

### Alternative Backends (Optional Post-Phase 4)
* [PyTorch ExecuTorch Core ML Backend](https://docs.pytorch.org/executorch/stable/backends-coreml.html)
* [ONNX Runtime Core ML Execution Provider](https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html)

---

**End of Plan — Final Consolidated Edition (Complete 2025 Core ML Implementation Guide)**

---

## 21. Critical Implementation Details from Plan1 (Not to Skip)

### PreparedModel Trait Interface

Add to `apple-silicon/src/inference.rs`:

```rust
pub trait PreparedModel: Send + Sync {
  fn cache_key(&self) -> &str;
  fn io_schema(&self) -> &IoSchema;
  fn sla_estimate(&self) -> Duration;
}

pub struct IoSchema {
  pub inputs: Vec<TensorSpec>,
  pub outputs: Vec<TensorSpec>,
}

pub struct TensorSpec {
  pub name: String,
  pub shape: Vec<usize>,
  pub dtype: DType,
  pub batch_capable: bool,
}
```

### Build Integration Details

**`apple-silicon/build.rs` specifics:**

* Invoke `swift build --configuration release --Xswiftc -static-stdlib`
* Handle `MACOSX_DEPLOYMENT_TARGET` environment variable
* Link flags: `-framework CoreML -framework Foundation`
* Link search path: `coreml-bridge/.build/release`
* Feature gate: `#[cfg(target_os = "macos")]` and `#[cfg(feature = "coreml")]`

### Rust Wrapper Pattern (core_ml_bridge.rs)

**Drop guard for autorelease pool:**

```rust
impl Drop for CoreMLModel {
  fn drop(&mut self) {
    unsafe { coreml_free_model(self.handle) }
    // Autorelease pool cleanup on Swift side
  }
}

/// Wrapper ensuring FFI calls run within autorelease context
fn with_autorelease_pool<F, T>(f: F) -> Result<T>
where
  F: FnOnce() -> Result<T>,
{
  // Pseudo: setup thread-local or call-scoped pool
  f()
}
```

All FFI calls must wrap their invocation:
```rust
with_autorelease_pool(|| unsafe {
  coreml_predict(handle, inputs_json, timeout_ms, &mut out, &mut err)
})
```

### Candle Backend Reference Implementation

**`apple-silicon/src/candle_backend.rs` specifics:**

* Load `.safetensors` models via `candle::safetensors`
* Establish numeric parity baseline: **L∞ < 1e-5, RMSE < 1e-6** (adjust for FP16/INT8)
* Unit tests with deterministic inputs (no randomness in tests)
* Mock implementations of `InferenceEngine` for testing trait contracts
* Verify `Send + Sync` bounds statically

### Telemetry Circuit Breaker Logic

**From `apple-silicon/src/telemetry.rs`:**

```rust
impl CoreMLMetrics {
  pub fn should_circuit_break(&self) -> bool {
    // Track 10 initial inferences
    if self.infer_count < 10 {
      return false;
    }
    
    let success_rate = self.infer_success / self.infer_count as f64;
    let p99_latency = self.infer_p99_ms;
    
    // Disable if <95% success OR p99 > SLA threshold N times
    success_rate < 0.95 || (p99_latency > self.sla_ms && self.sla_violations >= 3)
  }
  
  pub fn record_inference(&mut self, latency_ms: u64, success: bool) {
    self.infer_count += 1;
    if success { self.infer_success += 1; }
    // update p50/p95/p99 histograms
  }
}
```

### Parity Harness Implementation

**Establish numeric thresholds per model:**

* Candle CPU: **ground truth** (FP32 reference)
* Core ML CPU: expect L∞ ≤ 1e-2, RMSE ≤ 1e-3 (FP16 rounding)
* Core ML All (GPU/ANE): same parity as CPU variant
* INT8 variants: relax thresholds explicitly (L∞ ≤ 5e-2, RMSE ≤ 5e-3)
* Fail build if any variant exceeds threshold

### Instruments.app Validation Protocol

**Gate A Leak-Check:**

1. Open Instruments.app (Xcode)
2. Attach to running binary: `cargo run --example core_ml_compile_load_test --features coreml`
3. Set environment: `COREML_LEAK_TEST=1000` (run 1000 compile+load cycles)
4. Monitor: Allocations instrument
5. Watch for:
   - Steady state after 100 warmup iterations
   - Zero growth in RSS (resident set size)
   - Transient allocations freed by next cycle
6. **Pass criteria:** < 100KB total growth over 10k cycles

**Gate B Soak Test:**

1. Run: `cargo run --features coreml --example core_ml_inference_soak -- --duration 3600 --model fastvit_t8`
2. Monitor Allocations + VM Tracker for 1 hour
3. Measure: p50/p95/p99 latency stability
4. **Pass criteria:** no growth trend, QPS steady, no spikes

### Feature Flag & Conditional Compilation

**Update `apple-silicon/src/lib.rs`:**

```rust
#[cfg(feature = "coreml")]
pub mod core_ml_bridge;
#[cfg(feature = "coreml")]
pub mod core_ml_backend;

#[cfg(not(feature = "coreml"))]
pub use crate::candle_backend as inference_backend;

#[cfg(feature = "coreml")]
pub use crate::core_ml_backend as inference_backend;
```

Test both code paths locally:
```bash
cargo test --lib                      # CPU only
cargo test --lib --features coreml    # With Core ML
```

### SPM Package Structure (coreml-bridge)

**`coreml-bridge/Package.swift`:**

```swift
let package = Package(
  name: "CoreMLBridge",
  platforms: [.macOS(.v11)],
  products: [
    .library(name: "CoreMLBridge", targets: ["CoreMLBridge"])
  ],
  targets: [
    .target(
      name: "CoreMLBridge",
      dependencies: [],
      linkerSettings: [
        .linkedFramework("CoreML"),
        .linkedFramework("Foundation")
      ]
    )
  ]
)
```

**File structure:**

```
coreml-bridge/
  ├── Package.swift
  ├── Sources/
  │   └── CoreMLBridge.swift (~400 lines)
  └── README.md
```

### Cache Key Versioning (Detailed)

**Full cache key format:**

```
{authoring_sha256}:{coreml_version}:{ml_program_backend}:{compute_units}:{quantization}:{shape_key}:{os_build_number}
```

**Example:**
```
a1b2c3d4...:{version 5.0}:{mlprogram}:{all}:{fp16}:{224x224}:{24A335}
```

* **authoring_sha256:** SHA256 of input model file (`.mlmodel`, `.pt`, etc.)
* **coreml_version:** CoreML framework version (e.g., "5.0")
* **ml_program_backend:** "mlprogram" or "neuralnetwork"
* **compute_units:** "all", "cpu_only", "cpu_gpu", "cpu_ane"
* **quantization:** "fp32", "fp16", "int8", "palettized"
* **shape_key:** e.g., "224x224" for images, "batch_1" for sequences
* **os_build_number:** macOS build (e.g., "24A335")

**LRU eviction:**

* Track access time per cache entry
* On version mismatch (Xcode update, new Core ML) → evict old entries
* Log every eviction with reason + freed space

---

## 22. TODOs for Implementation Phases

### Phase 0 (Rust Contracts)

* [ ] Implement `InferenceEngine`, `PreparedModel`, `ModelArtifact` traits
* [ ] Add `CapabilityReport` with device class, ANE op coverage, compute unit tracking
* [ ] Implement `candle_backend.rs` with `.safetensors` loading
* [ ] Establish numeric parity baseline (L∞ < 1e-5, RMSE < 1e-6)
* [ ] Unit tests with mocked backends (no native execution)
* [ ] Verify `Send + Sync` bounds

### Phase 1 (Swift Bridge)

* [ ] Create `coreml-bridge/Package.swift` (SPM config)
* [ ] Implement `CoreMLBridge.swift` (~400 lines) with C ABI exports
* [ ] Implement `coreml_compile_model`, `coreml_load_model`, `coreml_model_schema`, `coreml_predict`
* [ ] Wrap all calls in `@autoreleasepool {}`
* [ ] Update `apple-silicon/build.rs` (invoke `swift build`, link `libCoreMLBridge.a`)
* [ ] Handle `MACOSX_DEPLOYMENT_TARGET`
* [ ] Feature gate: `#[cfg(target_os = "macos")]` + `#[cfg(feature = "coreml")]`
* [ ] Smoke test: FastViT T8 compile+load with Instruments verification
* [ ] Gate A validation: 10k compile+load cycles, < 100KB growth, ≥90% cache hit ratio

### Phase 2 (Inference MVP)

* [ ] Extend Swift bridge with `coreml_predict` (timeout_ms enforcement)
* [ ] Implement `core_ml_bridge.rs` (extern "C" bindings, Drop guards, with_autorelease_pool wrappers)
* [ ] Implement `core_ml_backend.rs` (InferenceEngine impl for Core ML)
* [ ] Implement `telemetry.rs` (counters, p50/p95/p99, circuit breaker logic)
* [ ] Build parity harness (Candle CPU vs Core ML CPU vs Core ML All)
* [ ] Define L∞/RMSE thresholds per model (relax for INT8 explicitly)
* [ ] Download FastViT T8; create `scripts/models/download_fastvit.py`
* [ ] Create `scripts/models/manifest.json` template with all fields
* [ ] Update `lib.rs` with feature flag conditional exports
* [ ] Gate B validation: correctness, timeout enforcement, p99 < 20ms, parity thresholds, 1-hour soak

### Phase 3 (Compression Lab)

* [ ] Create `scripts/models/convert_resnet50.py` (FP16 + INT8 variants)
* [ ] Quantization lab: FP16 → INT8 → 4-bit palettization → combined (prune+quantize)
* [ ] Per variant: track accuracy delta (L∞, RMSE), size reduction, latency across CPU/GPU/ANE
* [ ] Apply ANE transformer guidance to DistilBERT or custom model
* [ ] Validate static shapes, attention layout, 1×1 conv fusions
* [ ] Gate C validation: ≥30% speedup vs CPU on at least one model, ANE dispatch confirmed

### Phase 4 (Hardening)

* [ ] Implement pinned `MLMultiArray` buffer pools per shape/dtype
* [ ] Optional mmap I/O for large outputs (images, geometry, video frames)
* [ ] Static batching support (fixed batch shapes)
* [ ] Small pool of `MLModel` instances (size ≤ ANE pipeline width, typically 2-4)
* [ ] Back-pressure queue with per-task timeouts
* [ ] Gate D validation: 1+ hour soak stable, no leaks, steady QPS, SLOs documented

### Documentation

* [ ] Create `docs/CORE_ML_BRIDGE_GUIDE.md` (~500 lines)
  - Build/run/troubleshoot; Instruments how-to; per-phase checklist
* [ ] Create `docs/CORE_ML_MODELS.md` (~300 lines)
  - Model table: name, source, size, ops, precision, expected speedups; parity thresholds
* [ ] Create `scripts/models/README.md` (~150 lines)
  - Download + conversion instructions; known coremltools quirks (M1/M2/M3, Xcode versions)
* [ ] Create `docs/CI_SETUP_FOR_COREML.md` (skeleton)
  - GitHub Actions self-hosted Mac runner setup (implement when runners available)

---
 