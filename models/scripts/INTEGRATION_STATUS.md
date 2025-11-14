# EmbeddingGemma CoreML Integration Status

## Conversion Complete ✅

**Model**: `models/coreml/embeddinggemma.mlpackage` (578 MB)
**Format**: ML Program (CoreML)
**Input Type**: INT32 ✅ (correctly configured)
**Tokenizer**: `models/coreml/embeddinggemma_tokenizer/` ✅

---

## Integration Status

### ✅ Completed

1. **Model Conversion**: Successfully converted with model surgery
   - Replaced `bitwise_or` → `logical_or`
   - Replaced `new_ones` → `torch.full`
   - Fixed input type to INT32

2. **Model Detection**: Updated `EmbeddingServiceFactory` to detect `.mlpackage` format
   - Priority: `.mlpackage` → `.mlmodel` → `.gguf`

3. **Tokenizer Integration**: Updated to load HuggingFace tokenizer from saved location
   - Falls back to SimpleTokenizer if not found

### ⚠️ Known Issues

1. **Bridge Input Type Mismatch**
   - **Issue**: Rust bridge function `agentbridge_run_inference` expects `*const f32`
   - **Model Expects**: INT32 input_ids
   - **Current**: Rust code sends f32 (may work if CoreML auto-converts)
   - **Status**: Needs testing

2. **Bridge Function Signature**
   ```rust
   fn agentbridge_run_inference(
       model_ref: u64,
       input_name: *const std::ffi::c_char,
       input_data: *const f32,  // ← Currently f32, model needs INT32
       ...
   )
   ```

### 🔧 Required Updates

1. **Update Bridge for INT32 Support** (if f32 doesn't work)
   - Add `agentbridge_run_inference_int32` function
   - Or update existing function to handle both types
   - Update Swift bridge implementation

2. **Update Rust Provider** (if bridge updated)
   ```rust
   // Change from:
   let input_data: Vec<f32> = tokens.iter().map(|&t| t as f32).collect();
   
   // To:
   let input_data: Vec<i32> = tokens.iter().map(|&t| t as i32).collect();
   ```

---

## Testing

### Test Model Loading

```bash
# Set environment variable
export COREML_EMBEDDING_MODEL_PATH=models/coreml/embeddinggemma.mlpackage

# Or use default path detection
# (automatically checks models/coreml/embeddinggemma.mlpackage)
```

### Test with Python

```python
import coremltools as ct
import numpy as np

model = ct.models.MLModel("models/coreml/embeddinggemma.mlpackage")

# Test with INT32 input
input_ids = np.array([[1, 2, 3, 4, 5]], dtype=np.int32)
prediction = model.predict({"input_ids": input_ids})
print(f"Output shape: {prediction[next(iter(prediction))].shape}")
```

---

## Next Steps

1. **Test CoreML Bridge**: Verify if f32 → INT32 conversion works automatically
2. **Update Bridge** (if needed): Add INT32 support to bridge functions
3. **Integration Testing**: Test full embedding pipeline with Rust code
4. **Performance Benchmarking**: Measure ANE acceleration vs CPU

---

## Files Modified

- `models/scripts/convert_embeddinggemma_to_coreml.py` - INT32 input type, model surgery
- `iterations/v3/data-infrastructure/src/embedding/embedding_service.rs` - `.mlpackage` detection, HfTokenizer loading
- `models/scripts/coreml_surgery.py` - Operation replacement utilities

---

## Model Specifications

- **Input**: `input_ids` (INT32, shape: [1, 1..2048])
- **Output**: Embedding vector (FLOAT16, 768 dimensions)
- **Model Size**: 578 MB
- **Format**: ML Program (`.mlpackage`)
- **Precision**: FP16
- **Deployment Target**: macOS 13+ (ANE support)



