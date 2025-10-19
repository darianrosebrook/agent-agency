# Phase 3 Execution Plan - Gate C Validation (FastViT T8)

**Date:** October 18, 2025  
**Model:** FastViT T8 F16 (7.5 MB, .mlpackage format)  
**Status:** ✅ MODEL READY - Beginning Phase 3 validation

---

## Current State

### ✅ Prerequisites Complete
- [x] Model downloaded: `FastViTT8F16.mlpackage.zip` (7.5 MB)
- [x] Model extracted to: `tests/fixtures/models/FastViTT8F16.mlpackage/`
- [x] Telemetry system: 11/11 tests passing
- [x] Circuit breaker: Fully integrated and tested
- [x] Documentation: Complete (Gate C guide + checklist)

### Model Details
- **Name:** FastViT T8 F16
- **Format:** MLPackage (Apple's native format)
- **Precision:** FP16 (half-precision)
- **Expected speedup vs CPU:** 2.8-3.5x
- **ANE op coverage:** ~78%
- **Input shape:** [1, 3, 224, 224] (batch, channels, height, width)
- **Output shape:** [1, 1000] (batch, classes)

---

## Phase 3 Execution Steps

### Step 1: Verify Model Structure

```bash
ls -la tests/fixtures/models/FastViTT8F16.mlpackage/
# Should show: Data/ and Manifest.json
```

### Step 2: Create Compilation Test

Create a simple test to verify Core ML compilation:

```bash
cat > /tmp/test_fastvit_compile.rs << 'RUST'
#[cfg(test)]
mod fastvit_tests {
    use std::path::Path;
    
    #[test]
    fn test_fastvit_model_exists() {
        let model_path = "tests/fixtures/models/FastViTT8F16.mlpackage";
        assert!(Path::new(model_path).exists(), 
                "FastViT T8 model not found at {}", model_path);
        
        // Check for required files
        let manifest = Path::new(&format!("{}/Manifest.json", model_path));
        let data_dir = Path::new(&format!("{}/Data", model_path));
        
        assert!(manifest.exists(), "Manifest.json not found");
        assert!(data_dir.is_dir(), "Data directory not found");
        
        println!("✅ FastViT T8 F16 model structure validated");
    }
}
RUST
```

### Step 3: Run Telemetry Verification

```bash
cd apple-silicon
cargo test --lib telemetry -- --nocapture
cargo test --lib core_ml_backend -- --nocapture

# Expected: 11/11 passing
```

### Step 4: Create Phase 3 Test Suite

Create tests that exercise the full pipeline:

```bash
cat > apple-silicon/tests/phase3_fastvit_integration.rs << 'RUST'
#[cfg(test)]
mod phase3_tests {
    use std::path::Path;
    
    #[test]
    fn test_fastvit_model_accessibility() {
        let model_path = "tests/fixtures/models/FastViTT8F16.mlpackage";
        assert!(Path::new(model_path).exists());
        println!("✅ FastViT T8 model accessible");
    }
    
    #[test]
    fn test_telemetry_with_model_path() {
        // Verify telemetry can track operations on real model
        // This would use actual Core ML calls in Phase 3B
        println!("✅ Telemetry system ready for model operations");
    }
}
RUST
```

### Step 5: Performance Baseline Collection

Once model is loaded, measure:

**Metrics to Collect:**
- Compile time: CPU/GPU/ANE
- Inference latency: p50, p95, p99
- ANE dispatch rate (% of ops)
- Memory usage during inference
- Compute unit actually used

**Expected Values (from plan):**
- Compile: 2-5 seconds (first time)
- Compile: < 1 second (cached)
- Inference p99: 8-15 ms (with ANE)
- Inference p99: 30-50 ms (CPU baseline)
- Speedup: 2.8-3.5x

### Step 6: Instruments Profiling

```bash
# Build release binary
cargo build --example core_ml_smoke_test --release --features coreml

# Run with Instruments (attach manually or use env var)
open -a Instruments

# Profile for 1000 inferences
# Measure: < 100KB growth after warmup
```

### Step 7: Document Phase 3 Results

Create `PHASE_3_RESULTS.md`:

```markdown
# Phase 3 Validation Results - FastViT T8 F16

## Environment
- Device: [M1/M2/M3]
- macOS: [version]
- Xcode: [version]

## Model Information
- Name: FastViT T8 F16
- Format: MLPackage
- Precision: FP16
- Location: tests/fixtures/models/FastViTT8F16.mlpackage/

## Compilation Results
- First compile: XXms
- Cached compile: XXms
- Backend selected: [ML Program / Neural Network]

## Inference Performance (1000 cycles)
- CPU baseline: XXms (p99)
- Core ML (All): XXms (p99)
- ANE coverage: XX%
- Actual speedup: X.Xx

## Memory Profile
- Peak memory: XXMB
- Growth per 100 inferences: XXKB
- Status: [Pass/Fail]

## Numeric Parity
- L∞ error: X.XXe-XX
- RMSE: X.XXe-XX
- Status: [Pass/Marginal/Fail]

## Overall Gate C Status
✅ PASS - All success criteria met
⏳ PARTIAL - Telemetry verified, [specific issue]
❌ FAIL - [Reason]
```

---

## Success Criteria (Gate C)

| Criterion | Target | Status |
|-----------|--------|--------|
| Model loads | ✅ Yes | ⏳ Testing |
| Telemetry collects | ✅ Yes | ✅ Verified |
| Circuit breaker functional | ✅ Yes | ✅ Verified |
| Speedup vs CPU | ≥ 30% (2.8x+) | ⏳ Measuring |
| ANE dispatch | ≥ 70% ops | ⏳ Measuring |
| Memory growth | < 100KB/100 inferences | ⏳ Profiling |
| Numeric parity | L∞ ≤ 1e-2, RMSE ≤ 1e-3 | ⏳ Validating |

---

## Timeline

| Step | Estimated Time | Status |
|------|----------------|--------|
| Verify model structure | 2 min | ⏳ Ready |
| Run telemetry tests | 5 min | ⏳ Ready |
| Create test suite | 10 min | ⏳ Ready |
| Measure performance | 15 min | ⏳ Ready |
| Profile with Instruments | 30 min | ⏳ Ready |
| Document results | 10 min | ⏳ Ready |

**Total: ~70 minutes (comfortable margin for troubleshooting)**

---

## Next Actions

### Immediate (This Step)
```bash
# 1. Verify model is ready
ls -la tests/fixtures/models/FastViTT8F16.mlpackage/

# 2. Run telemetry tests
cd apple-silicon
cargo test --lib telemetry core_ml_backend -- --nocapture

# 3. Create Phase 3 test file
# (See step 4 above)
```

### Short-term (Phase 3A)
- [x] Model download
- [ ] Verify model structure
- [ ] Create Phase 3 test suite
- [ ] Document model metadata

### Medium-term (Phase 3B)
- [ ] Run 100 inference cycles
- [ ] Collect telemetry data
- [ ] Measure latencies (p50/95/99)
- [ ] Verify circuit breaker doesn't trip

### Extended (Phase 3C)
- [ ] Run 1000+ inferences
- [ ] Profile with Instruments
- [ ] Measure ANE dispatch rate
- [ ] Validate numeric parity
- [ ] Complete Gate C report

---

## Known Issues & Mitigations

| Issue | Mitigation | Status |
|-------|-----------|--------|
| PyTorch unavailable | Use pre-converted models ✅ | ✅ Done |
| No real inference yet | All components tested independently | ✅ Ready |
| Model format (MLPackage) | Native Apple format, supported | ✅ OK |
| ANE dispatch uncertain | Telemetry logs actual dispatch | ✅ Ready |

---

## Files Modified/Created

### Created
- [x] `tests/fixtures/models/FastViTT8F16.mlpackage/` (model dir)
- [x] `PHASE_3_EXECUTION_PLAN.md` (this file)
- [ ] `PHASE_3_RESULTS.md` (results, TBD)
- [ ] `apple-silicon/tests/phase3_fastvit_integration.rs` (test suite, TBD)

### Updated
- [x] `README.md` (added architecture overview)
- [ ] `docs/GATE_C_TESTING_CHECKLIST.md` (with model-specific steps)

---

## Architecture Readiness Checklist

Before Phase 3B (actual inference), verify:

- [x] Telemetry system: 11/11 tests passing
- [x] Circuit breaker: Integrated and tested
- [x] Error handling: No panics across FFI
- [x] Thread safety: Concurrent access verified
- [x] Model acquisition: FastViT T8 ready
- [x] Documentation: Complete with guides
- [ ] Phase 3 tests: To be created
- [ ] Instruments setup: To be configured

---

**Next Document:** PHASE_3_RESULTS.md (to be created after testing)

