# ONNX Runtime Integration Plan

## Status: ONNX Export Complete ✅

- **ONNX Model**: `models/coreml/embeddinggemma.onnx` ✅
- **Dynamic Axes**: Variable-length inputs supported ✅
- **ONNX Runtime Test**: Inference works correctly ✅
- **CoreML Conversion**: Not supported in CoreMLTools 8.3.0 ❌

## Decision: Use ONNX Runtime Directly

Based on kokoro-onnx patterns, we'll use ONNX Runtime with CoreMLExecutionProvider for ANE acceleration.

### Advantages

1. **Apple Silicon Acceleration**: CoreMLExecutionProvider uses ANE
2. **Variable-Length Inputs**: Dynamic axes support native
3. **Proven Pattern**: kokoro-onnx has production-ready implementation
4. **Rust Bindings**: `ort` crate available for Rust integration
5. **No Conversion Needed**: Direct ONNX model usage

### Implementation Plan

1. **Add `ort` crate** to `data-infrastructure/Cargo.toml`
2. **Create `OnnxEmbeddingProvider`** following kokoro-onnx patterns:
   - Hardware detection (Apple Silicon, ANE cores)
   - Provider priority: CoreMLExecutionProvider → CPUExecutionProvider
   - Session options optimization
   - ANE optimization similar to kokoro-onnx
3. **Update `EmbeddingServiceFactory`** to prefer ONNX model
4. **Test with variable-length inputs** to verify dynamic axes

### Next Steps

1. ✅ Export ONNX model with dynamic axes
2. 🔄 Add `ort` crate dependency
3. 🔄 Implement `OnnxEmbeddingProvider` with ANE optimization
4. 🔄 Update factory to use ONNX Runtime
5. 🔄 Test inference with variable-length inputs

## Reference: kokoro-onnx Patterns

- **Hardware Detection**: `api/model/hardware/detection.py`
- **Session Options**: `api/model/providers/ort.py::create_optimized_session_options`
- **ANE Optimization**: `api/model/optimization/ane_optimizer.py`
- **Provider Configuration**: `api/model/providers/coreml.py`

## Rust Implementation

```rust
// Use ort crate for ONNX Runtime
use ort::{Session, ExecutionProvider, SessionBuilder};

// Provider priority: CoreML → CPU
let providers = if is_apple_silicon {
    vec![
        ExecutionProvider::CoreML(Default::default()),
        ExecutionProvider::CPU(Default::default()),
    ]
} else {
    vec![ExecutionProvider::CPU(Default::default())]
};

// Create session with optimized options
let session = SessionBuilder::new()?
    .with_execution_providers(providers)?
    .commit_from_file(onnx_model_path)?;

// Run inference
let outputs = session.run(ort::inputs!["input_ids" => input_ids]?)?;
```



