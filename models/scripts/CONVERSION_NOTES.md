# EmbeddingGemma CoreML Conversion Notes

## Status

Conversion script is functional but encounters CoreML limitation:

### ✅ Completed

- Model loading via SentenceTransformer ✓
- TorchScript tracing (CPU mode) ✓  
- Input/output shape detection ✓
- Attention mask handling ✓

### ❌ Known Limitation

**CoreML Conversion Error**: `bitwise_or` operation not supported

The EmbeddingGemma model (Gemma3 architecture) uses `bitwise_or` operations internally, which CoreML does not support. This occurs at layer `base_model/104`.

### Workarounds

1. **ONNX Intermediate Format** (recommended)
   ```bash
   pip install onnx onnxruntime
   # Convert PyTorch → ONNX → CoreML
   ```

2. **Model Architecture Modification**
   - Replace `bitwise_or` with equivalent operations CoreML supports
   - Requires model surgery or using a different model variant

3. **Use Alternative Embedding Model**
   - Consider models without unsupported operations
   - Check CoreML compatibility before conversion

### Dependencies

- Python 3.9+ (tested with 3.9.6)
- torch 2.8.0
- transformers 4.57.1
- coremltools 8.3.0
- sentence-transformers 5.1.2

### Virtual Environment

The conversion uses `venv-py39` (Python 3.9) to avoid compatibility issues with Python 3.14.

### Usage

```bash
source venv-py39/bin/activate
python3 models/scripts/convert_embeddinggemma_to_coreml.py \
  --model-id models/coreml/embeddinggemma-300m-raw \
  --output-dir models/coreml
```

### Next Steps

1. Install ONNX and attempt PyTorch → ONNX → CoreML conversion
2. Contact CoreML team about `bitwise_or` support
3. Consider alternative embedding models compatible with CoreML



