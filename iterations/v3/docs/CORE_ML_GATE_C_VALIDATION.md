# Core ML Gate C Validation – Comprehensive Guide

**Author:** @darianrosebrook  
**Date:** October 18, 2025  
**Purpose:** ANE acceleration measurement and production readiness verification  
**Status:** Ready for execution

---

## Overview

Gate C validates that Core ML integration delivers real ANE acceleration with acceptable numeric parity. This guide walks through setup, execution, and measurement of:

- Model compilation & loading (infrastructure readiness)
- Inference execution (crash-free operation)
- ANE dispatch (acceleration confirmed)
- Performance improvement (speedup measured)
- Numeric accuracy (parity verified)

---

## Prerequisites

### Software

```bash
# Install Python model conversion dependencies
pip install torch torchvision coremltools onnx numpy

# Verify Xcode compatibility
xcode-select --print-path
# Should output: /Applications/Xcode.app/Contents/Developer

# Check Xcode version (macOS 14+)
xcodebuild -version
# Should be: Xcode 15.0+ or later
```

### Hardware

- **Required:** Apple Silicon Mac (M1/M2/M3/M4)
- **Recommended:** 16GB+ RAM, SSD, thermal headroom
- **Testing Time:** 30 mins - 2 hours depending on model size

---

## Phase 1: Model Acquisition

### 1.1 Download and Convert FastViT T8

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3
python3 scripts/models/download_fastvit.py
```

**Expected Output:**
```
======================================================================
FastViT T8 → Core ML Conversion Pipeline
======================================================================
[*] Downloading FastViT T8 from torchvision...
[✓] Model downloaded successfully
[*] Converting model to Core ML (fp16)...
[*] Exporting to ONNX: ...
[*] Converting ONNX to Core ML: ...
[*] Applying FP16 quantization...
[✓] Model saved to tests/fixtures/models/fastvit_t8.mlmodel
[✓] Manifest created: tests/fixtures/models/manifest.json

======================================================================
[✓] SUCCESS: Model ready at tests/fixtures/models/fastvit_t8.mlmodel
[✓] Next step: Run 'cargo test --lib --features coreml'
======================================================================
```

### 1.2 Verify Model File

```bash
file tests/fixtures/models/fastvit_t8.mlmodel
# Should output: ... Mac OS X ... (or directory if .mlmodelc)

ls -lh tests/fixtures/models/fastvit_t8.mlmodel
# Should show ~8-10 MB size for FP16 variant
```

### 1.3 Inspect Manifest

```bash
cat tests/fixtures/models/manifest.json | jq .
```

**Expected manifest structure:**
```json
{
  "model": "fastvit_t8",
  "backend": "mlprogram",
  "precision": "fp16",
  "ane_op_coverage_pct": 78,
  "expected_speedup_m1": 2.8,
  "expected_speedup_m2": 3.1,
  "expected_speedup_m3": 3.2,
  "accuracy_delta_l_inf": 0.0001,
  "accuracy_delta_rmse": 0.00005
}
```

---

## Phase 2: Rust Unit Tests

### 2.1 Build with Core ML Feature

```bash
cargo build --lib --features coreml
```

**Verify compilation:** Should complete with warnings but no errors.

### 2.2 Run Telemetry Tests

```bash
cargo test --lib telemetry --features coreml -- --nocapture
```

**Expected output (7/7 tests passing):**
```
running 7 tests
test telemetry::tests::test_metrics_record_inference ... ok
test telemetry::tests::test_metrics_record_compile ... ok
test telemetry::tests::test_circuit_breaker_needs_sample_size ... ok
test telemetry::tests::test_circuit_breaker_low_success_rate ... ok
test telemetry::tests::test_telemetry_collector_thread_safe ... ok
test telemetry::tests::test_failure_mode_tracking ... ok
test core_ml_backend::tests::test_core_ml_backend_telemetry_integration ... ok

test result: ok. 7 passed; 0 failed
```

### 2.3 Run Core ML Backend Tests

```bash
cargo test --lib core_ml_backend --features coreml -- --nocapture
```

**Expected output (4/4 tests passing):**
```
running 4 tests
test core_ml_backend::tests::test_core_ml_backend_creation ... ok
test core_ml_backend::tests::test_core_ml_backend_default ... ok
test core_ml_backend::tests::test_core_ml_backend_telemetry_integration ... ok
test core_ml_backend::tests::test_core_ml_backend_circuit_breaker_integration ... ok

test result: ok. 4 passed; 0 failed
```

---

## Phase 3: Inference Testing (Manual - Local)

### 3.1 Candle CPU Baseline (Ground Truth)

```bash
# Run inference harness with Candle backend (CPU only)
# This will establish baseline latency for comparison
```

**Expected baseline (FastViT T8 on M1):**
- P50: ~15-25 ms
- P99: ~40-60 ms
- P99.9: ~80-120 ms

**Record these values** for Gate C comparison.

### 3.2 Core ML Inference (GPU/ANE)

```bash
# Once Swift bridge is built and linked, run:
# cargo run --example core_ml_validation --features coreml -- \
#   --model tests/fixtures/models/fastvit_t8.mlmodel \
#   --iterations 1000
```

**Expected behavior:**
1. Model compilation: < 5 seconds
2. Model loading: < 1 second
3. 1000 inferences: ~ 10-30 seconds (vs 30-60 seconds for CPU)
4. Zero crashes
5. Telemetry metrics recorded

---

## Phase 4: Telemetry Analysis

### 4.1 Metrics to Record

**Compile Metrics:**
```
- Compile count: 1
- Compile success: 1
- Compile time: < 5000 ms
- Compile p99: < 5000 ms
```

**Inference Metrics:**
```
- Infer count: 1000
- Infer success: >= 950 (95%)
- Infer p50: target 7-15 ms
- Infer p99: target < 20 ms
- Infer p99.9: target < 30 ms
```

**Dispatch Metrics:**
```
- ANE usage count: > 0 (confirms ANE dispatch)
- GPU usage count: >= 0 (optional)
- CPU fallback count: < 50 (< 5%)
```

**Circuit Breaker:**
```
- Circuit breaker enabled: true (initially)
- Circuit breaker trips: 0 (if success rate >= 95%)
- SLA violations: < 30 (< 3%)
```

### 4.2 Export Telemetry

```bash
# Telemetry is available via:
# - backend.telemetry_summary() → formatted string
# - backend.get_metrics() → structured CoreMLMetrics
```

---

## Phase 5: Performance Profiling (Instruments.app)

### 5.1 Setup Profiling

```bash
# Build release binary for profiling
cargo build --release --features coreml

# Launch Instruments
open -a Instruments
```

### 5.2 Profiling Steps

1. **Select template:** "System Trace" or "Counters"
2. **Select process:** Your binary
3. **Start recording**
4. **Run 100-1000 inferences**
5. **Stop recording**
6. **Analyze:**
   - Look for ANE activity (Neural Engine tracks)
   - Check GPU utilization if applicable
   - Verify no excessive autorelease pool churn

### 5.3 What to Look For

**Good Signs:**
- ✅ ANE tracks show activity during inference
- ✅ GPU activity visible if GPU path used
- ✅ No memory growth over time
- ✅ No excessive context switches

**Red Flags:**
- ❌ Flat CPU timeline (indicates CPU fallback)
- ❌ Memory unbounded growth (leak)
- ❌ Excessive context switching (contention)

---

## Phase 6: Numeric Parity Validation

### 6.1 Parity Testing (Optional but Recommended)

```bash
python3 << 'EOF'
# Compare outputs: CPU (Candle) vs Core ML
# For each test image:
#  1. Run through Candle (ground truth)
#  2. Run through Core ML
#  3. Compute L∞ norm and RMSE
#  4. Verify within acceptable thresholds

import numpy as np

def l_inf_norm(a, b):
    return np.max(np.abs(a - b))

def rmse(a, b):
    return np.sqrt(np.mean((a - b) ** 2))

# For FP16 quantization:
threshold_l_inf = 1e-2
threshold_rmse = 1e-3

# Example:
candle_out = np.random.randn(1, 1000)  # Placeholder
coreml_out = np.random.randn(1, 1000)  # Placeholder

l_inf = l_inf_norm(candle_out, coreml_out)
rmse_val = rmse(candle_out, coreml_out)

assert l_inf <= threshold_l_inf, f"L∞ {l_inf} exceeds {threshold_l_inf}"
assert rmse_val <= threshold_rmse, f"RMSE {rmse_val} exceeds {threshold_rmse}"

print(f"✅ Parity verified: L∞={l_inf:.6f}, RMSE={rmse_val:.6f}")
EOF
```

---

## Gate C Success Criteria

### Acceptance Checklist

- [ ] **Model Acquisition**: Model downloads and converts without errors
- [ ] **Unit Tests**: 7/7 telemetry tests pass
- [ ] **Infrastructure**: Model loads without crash
- [ ] **Inference**: 1000 inferences complete with 0 crashes
- [ ] **Success Rate**: >= 95% inference success rate
- [ ] **ANE Dispatch**: `ane_usage_count > 0` in telemetry
- [ ] **Speedup**: Core ML p99 ≤ 0.7 × Candle p99
- [ ] **Parity**: L∞ ≤ 1e-2 (FP16), RMSE ≤ 1e-3 (FP16)
- [ ] **Memory**: No unbounded growth over 1000 iterations
- [ ] **Profiling**: Instruments confirms ANE/GPU utilization

---

## Troubleshooting

### Model Not Found

```bash
# Regenerate model
python3 scripts/models/download_fastvit.py

# Verify output
ls -la tests/fixtures/models/
```

### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build --lib --features coreml

# Check Rust version
rustc --version
# Should be 1.70+
```

### Runtime Crashes

```bash
# Check system state
sysctl -n hw.memsize
# At least 8GB

sysctl -n hw.physicalcpu
# At least 4 cores recommended

# Check macOS version
sw_vers
# macOS 11+ required
```

### ANE Not Detected

```bash
# Verify device has ANE
system_profiler SPMDataType | grep "Apple Neural Engine"

# If not present: testing on x86_64, no ANE available
# This is expected on Intel Macs
```

---

## Success and Next Steps

### Gate C Passed ✅

**Actions:**
1. Document measured speedup and parity
2. Update README with "Core ML Production-Ready" status
3. Proceed to Gate D (1-hour soak testing)
4. Merge to main with feature flag enabled by default

### Gate C Failed ⚠️

**Debug Steps:**
1. Check telemetry summary for failure mode
2. Review circuit breaker logs
3. Profile with Instruments for bottlenecks
4. Return to Phase 2-3 for fixes
5. Retry with updated code

---

## References

- Core ML Docs: https://developer.apple.com/documentation/coreml
- Performance Tips: https://developer.apple.com/videos/play/wwdc2023/10047/
- ANE Guide: https://github.com/apple/ml-ane-transformers
- Instruments Guide: https://developer.apple.com/forums/topics/instruments

---

**Status:** Gate C documentation complete. Ready for manual local testing.
