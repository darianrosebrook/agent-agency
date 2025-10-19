# Phase 3B: Model Inference Testing & Performance Validation

**Date:** October 18, 2025  
**Model:** FastViT T8 F16 (MLPackage, 7.5 MB)  
**Status:** ✅ BEGINNING PHASE 3B EXECUTION  
**Objective:** Run actual inferences, collect telemetry, measure performance

---

## Phase 3B Overview

Phase 3B focuses on **actual inference execution** - loading the model and running predictions while collecting comprehensive telemetry data.

### Steps
1. ✅ Model structure validated (Phase 3A)
2. ⏳ Load model into memory
3. ⏳ Run inference cycles (100-1000)
4. ⏳ Collect telemetry metrics
5. ⏳ Measure performance (latency, speedup, ANE dispatch)
6. ⏳ Profile memory behavior
7. ⏳ Document findings

---

## Current Readiness Status

### ✅ Prerequisites Met
- Model downloaded: FastViT T8 F16 (7.5 MB)
- Model extracted: `tests/fixtures/models/FastViTT8F16.mlpackage/`
- Model validated: ✅ (Phase 3A)
- Telemetry system: ✅ (11/11 tests passing)
- Circuit breaker: ✅ (tested)
- Documentation: ✅ (complete)

### ⏳ Next: Actual Testing

To proceed, we need to:
1. Create a minimal Swift test that can load and compile the model
2. Create a Rust integration that calls the Swift bridge
3. Run multiple inference cycles
4. Collect and analyze telemetry data

---

## Implementation Strategy

### Option A: Manual Testing (Recommended First)

Since this is Phase 3B and we're validating the architecture, manual testing is appropriate:

```bash
# 1. Build a simple test binary
cargo build --example core_ml_smoke_test --release --features coreml

# 2. Run with model path
./target/release/examples/core_ml_smoke_test \
  --model tests/fixtures/models/FastViTT8F16.mlpackage \
  --iterations 100 \
  --output telemetry.json

# 3. Analyze results
cat telemetry.json | jq '.'
```

### Option B: Automated Integration Test

Create a test that exercises the full pipeline:

```bash
# Run Phase 3B tests
cargo test --test phase3b_inference_cycles -- --nocapture
```

---

## What Phase 3B Measures

### Performance Metrics
- **Compile time** (first run vs cached)
- **Load time** (from disk to memory)
- **Inference latency** (p50, p95, p99)
- **Throughput** (inferences/second)
- **Speedup** vs CPU baseline

### Device Metrics
- **Compute units used** (ANE/GPU/CPU actual)
- **Memory usage** (peak, growth rate)
- **ANE op coverage** (% of model on ANE)

### Reliability Metrics
- **Success rate** (% inferences without error)
- **Circuit breaker trips** (if any)
- **Fallbacks to CPU** (if triggered)

### Quality Metrics
- **Numeric parity** vs CPU (L∞, RMSE)
- **Memory leaks** (post-warmup stability)
- **Timeout compliance** (all within budget)

---

## Success Criteria (Gate C)

| Criterion | Target | Measurement Method |
|-----------|--------|-------------------|
| Model loads | No panic | Test doesn't crash |
| Telemetry works | Metrics collected | Check telemetry.json |
| Speedup | ≥2.8x (30% vs CPU) | Compare p99 latencies |
| ANE dispatch | ≥70% ops | Check telemetry dispatch % |
| Memory growth | <100KB per 100 inf | Profile with Instruments |
| Numeric parity | L∞ ≤1e-2 | Compare vs CPU baseline |
| Circuit breaker | <95% failure to trip | Verify in logs |

---

## Next Immediate Actions

### 1. Create Phase 3B Integration Test

We'll create a test that:
- Loads the FastViT model
- Runs 100 inference cycles
- Collects telemetry
- Reports results

### 2. Manual Profiling Session

Using Instruments.app to:
- Monitor memory (Allocations instrument)
- Check for leaks (Leaks instrument)
- Measure CPU/GPU usage (System Trace)

### 3. Document Results

Create `PHASE_3B_RESULTS.md` with:
- Environment details
- Performance measurements
- Telemetry analysis
- Gate C pass/fail verdict

---

## Timeline for Phase 3B

| Step | Time | Status |
|------|------|--------|
| Create test suite | 15 min | ⏳ Next |
| Run 100 inferences | 5 min | ⏳ Pending |
| Measure latencies | 10 min | ⏳ Pending |
| Profile with Instruments | 30 min | ⏳ Pending |
| Analyze results | 15 min | ⏳ Pending |
| Document findings | 15 min | ⏳ Pending |

**Total: ~90 minutes**

---

## Files to Create/Update

- [ ] `apple-silicon/tests/phase3b_inference_cycles.rs` - Main test suite
- [ ] `PHASE_3B_RESULTS.md` - Results and measurements
- [ ] `docs/PHASE_3B_PROFILING_GUIDE.md` - Instruments.app guide
- [x] `PHASE_3B_INFERENCE_TESTING.md` - This file

---

## Next Steps

1. ✅ Read this document
2. ⏳ Create Phase 3B test suite
3. ⏳ Run inference cycles
4. ⏳ Collect telemetry
5. ⏳ Profile with Instruments
6. ⏳ Document results

---

**Ready to proceed to Phase 3B integration testing.**

