# ONNX Conversion Investigation Results

## ✅ ONNX Export Success

- **ONNX model created**: `models/coreml/embeddinggemma.onnx`
- **ONNX verification**: ✅ Model structure valid
- **ONNX Runtime inference**: ✅ Works with correct input shape
- **Operations patched**: ✅ `bitwise_and` and `bitwise_or` replaced

## ❌ CoreML Conversion Issue

**CoreMLTools 8.3.0 does not support ONNX conversion**

- Available sources: `["auto", "tensorflow", "pytorch", "milinternal"]`
- No ONNX frontend in `coremltools.converters.mil.frontend`
- `onnxcoreml` package exists but is deprecated

## 🔧 Alternative Solutions

### Option 1: Use ONNX Runtime Directly (Recommended)

**ONNX Runtime supports Apple Silicon acceleration** and can be used directly:

```python
import onnxruntime as ort
import numpy as np

# Load ONNX model
session = ort.InferenceSession("embeddinggemma.onnx")

# Run inference
input_ids = np.array([[1, 2, 3, 4, 5]], dtype=np.int64)
outputs = session.run(None, {'input_ids': input_ids})
```

**Advantages:**
- ✅ ONNX Runtime supports Apple Silicon (Metal backend)
- ✅ Works with variable-length inputs (if exported correctly)
- ✅ Rust bindings available (`ort`)
- ✅ No CoreML conversion needed

**Integration:**
- Use `onnxruntime` Rust crate instead of CoreML bridge
- Similar performance to CoreML on Apple Silicon

### Option 2: Fix ONNX Export for Variable Length

Re-export ONNX model with proper dynamic axes:

```python
torch.onnx.export(
    wrapped_model,
    example_input_ids,
    str(onnx_path),
    input_names=['input_ids'],
    output_names=['embeddings'],
    dynamic_axes={
        'input_ids': {0: 'batch_size', 1: 'sequence_length'},
        'embeddings': {0: 'batch_size', 1: 'sequence_length'},
    },
    opset_version=18,
)
```

### Option 3: Continue Debugging Direct PyTorch → CoreML

Focus on fixing the CoreML error -5 with the direct conversion:
- Investigate ML Program runtime compilation
- Check CoreML model graph for issues
- Consider using Neural Network format instead

## 📊 Current Status

- ✅ **ONNX Export**: Working
- ✅ **ONNX Runtime**: Works with correct input shape
- ❌ **ONNX → CoreML**: Not supported in CoreMLTools 8.3.0
- ⚠️ **Direct PyTorch → CoreML**: Runtime error -5

## 💡 Recommendation

**Use ONNX Runtime directly** instead of CoreML conversion:
1. Simpler (no conversion step)
2. Better variable-length input support
3. Apple Silicon acceleration available
4. Rust bindings available

Next: Update Rust integration to use ONNX Runtime instead of CoreML.



