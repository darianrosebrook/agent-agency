# Core ML Model Acquisition & Conversion

Scripts for downloading, converting, and quantizing machine learning models to Core ML format with ANE optimization.

## Setup

### Prerequisites

```bash
pip install torch torchvision coremltools onnx numpy
```

### Xcode & macOS

- macOS 11.0+ (for Core ML framework)
- Xcode 12.0+ (for ML models support)
- Apple Silicon (M1/M2/M3) recommended for testing (x86_64 supported but slower)

## Models

### FastViT T8 (Primary)

**Purpose**: Fast Vision Transformer T8 - lightweight classification model optimized for ANE

**Variants**:
- FP16 (primary): ~5-8 MB, 2.5-3.5x speedup vs CPU

**Run Conversion**:
```bash
cd scripts/models
python3 download_fastvit.py
```

**Output**:
- `tests/fixtures/models/fastvit_t8.mlmodel`
- `tests/fixtures/models/manifest.json`

### ResNet-50 (Baseline)

**Purpose**: Compression lab baseline - test quantization variants

**Variants**:
- FP32: ~90 MB (baseline reference)
- FP16: ~45 MB, minimal accuracy loss
- INT8: ~22.5 MB, 2-3% accuracy loss

**Run Conversion**:
```bash
cd scripts/models
python3 convert_resnet50.py
```

**Output**:
- `tests/fixtures/models/resnet50_fp32.mlmodel`
- `tests/fixtures/models/resnet50_fp16.mlmodel`
- `tests/fixtures/models/resnet50_int8.mlmodel`
- `tests/fixtures/models/manifest_*.json`

## Manifest Schema

Each model includes a `manifest.json` describing:

```json
{
  "model": "fastvit_t8",
  "source": "Apple Model Zoo",
  "backend": "mlprogram",
  "io_schema": {
    "inputs": [{"name": "input", "dtype": "fp32", "shape": [1, 3, 224, 224]}],
    "outputs": [{"name": "output", "dtype": "fp32", "shape": [1, 1000]}]
  },
  "precision": "fp16",
  "quantization": "none",
  "ane_op_coverage_pct": 78,
  "expected_speedup_m1": 2.8,
  "expected_speedup_m2": 3.1,
  "expected_speedup_m3": 3.2,
  "accuracy_delta_l_inf": 0.0001,
  "accuracy_delta_rmse": 0.00005
}
```

## Known Issues & Quirks

### macOS Version Compatibility

- macOS 11-13: Use Xcode 13.2.1+
- macOS 14: Use Xcode 14.0+
- macOS 15: Use Xcode 15.0+

**Workaround**: Check Xcode version with `xcode-select --version`

### coremltools Version Constraints

- coremltools >= 6.0: Required for ML Program backend
- coremltools < 7.0: Avoid FP16 precision issues in convert

```bash
pip install 'coremltools>=6.0,<7.0'
```

### GPU/ANE Acceleration During Conversion

- Conversion (PyTorch → Core ML) runs on CPU (no GPU acceleration)
- Inference (model.predict) uses ANE/GPU when configured
- Plan 2-5 minutes per large model (ResNet-50) on M1/M2

### Common Errors

**Error: `MLModelCollection: Failed to load model`**
- Cause: Incompatible Core ML version
- Fix: Update Xcode to latest (or use matching coremltools)

**Error: `ONNX to Core ML conversion failed: Unsupported op`**
- Cause: PyTorch model uses ops unsupported by Core ML
- Fix: Use simpler architectures or older PyTorch ops (opset 11-13)

**Error: `Quantization failed: Weight threshold too low`**
- Cause: INT8/FP16 quantization excluded all weights
- Fix: Lower `weight_threshold` in script (current: 512 bytes)

## Testing Generated Models

### Quick Validation

```bash
# Test Fast ML compilation
cargo test --lib --features coreml core_ml_backend

# Test full inference pipeline
cargo run --example core_ml_smoke_test --features coreml -- \
  --model tests/fixtures/models/fastvit_t8.mlmodel \
  --iterations 100
```

### Profiling with Instruments

```bash
# Profile memory during 1000 inferences
cargo build --example core_ml_smoke_test --release --features coreml
open -a Instruments /path/to/binary
# Select: Allocations, then start recording
# Run: COREML_LEAK_TEST=1000 ./binary
```

## Model Accuracy Validation

**L∞ Norm** (max absolute error):
- FP32 baseline: 0 (ground truth)
- FP16: < 1e-2 (typical: 5e-4)
- INT8: < 5e-2 (typical: 2e-3)

**RMSE** (root mean squared error):
- FP32: 0
- FP16: < 1e-3 (typical: 2e-4)
- INT8: < 5e-3 (typical: 1e-3)

Validation comparison:
```bash
python3 scripts/compare_parity.py \
  --model fastvit_t8 \
  --baseline fp32 \
  --variants fp16 int8 \
  --samples 100
```

## References

- [Apple Core ML Documentation](https://developer.apple.com/documentation/coreml)
- [coremltools GitHub](https://github.com/apple/coremltools)
- [PyTorch ONNX Export](https://pytorch.org/docs/stable/onnx.html)
- [Core ML Tools Quantization](https://apple.github.io/coremltools/docs-guides/)

## Troubleshooting

Check logs:
```bash
# Enable verbose output
export COREML_VERBOSE=1
python3 download_fastvit.py
```

Verify models:
```bash
file tests/fixtures/models/*.mlmodel
```

Clean rebuild:
```bash
cargo clean
cargo build --features coreml
```

