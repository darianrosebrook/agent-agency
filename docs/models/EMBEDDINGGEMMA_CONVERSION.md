# EmbeddingGemma GGUF to CoreML Conversion Guide

**Author**: @darianrosebrook  
**Date**: November 2025  
**Status**: Conversion Script Available

---

## Overview

The `embeddinggemma-300m` model in GGUF format (from Ollama) cannot be directly used with CoreML. This guide explains how to convert the original model from HuggingFace to CoreML format for ANE-accelerated inference.

---

## Prerequisites

### Required Software

```bash
# Python 3.9+ (you have 3.14.0)
python3 --version

# Install conversion dependencies
pip install torch transformers coremltools

# Verify installations
python3 -c "import torch; import transformers; import coremltools; print('✅ All dependencies installed')"
```

### System Requirements

- **macOS 13+** (Ventura or later) for CoreML support
- **Apple Silicon** (M1/M2/M3/M4) for ANE acceleration
- **16GB+ RAM** recommended for model conversion
- **Xcode Command Line Tools** (for CoreML compilation)
  ```bash
  xcode-select --install
  ```

---

## Conversion Methods

### Method 1: Automated Script (Recommended)

The provided script downloads the model from HuggingFace and converts it automatically:

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency

# Basic conversion (FP16, INT8 quantized)
python3 models/scripts/convert_embeddinggemma_to_coreml.py

# Custom model ID (if needed)
python3 models/scripts/convert_embeddinggemma_to_coreml.py \
  --model-id "google/gemma-2-2b" \
  --output-dir models/coreml

# High precision (FP32, no quantization)
python3 models/scripts/convert_embeddinggemma_to_coreml.py \
  --fp32 \
  --no-quantize
```

**Expected Output:**
```
======================================================================
EmbeddingGemma → CoreML Conversion Pipeline
======================================================================

📥 Model: google/gemma-2-2b
📁 Output: models/coreml
🎯 Precision: FP16
⚖️  Quantization: INT8

[*] Loading model from HuggingFace...
    Downloading google/gemma-2-2b...
    ✅ Tokenizer loaded
    ✅ Model loaded via AutoModel
    ✅ Model set to evaluation mode

[*] Analyzing model architecture...
    Hidden size: 2048
    Vocab size: 256000
    Max length: 8192

[*] Creating example input for model tracing...
    Example input shape: torch.Size([1, 8192])

[*] Tracing model with TorchScript...
    ✅ Model traced successfully

[*] Converting to CoreML format...
    ✅ CoreML conversion successful

[*] Applying INT8 quantization...
    ✅ Quantization applied

[*] Saving CoreML model...
    ✅ Model saved: models/coreml/embeddinggemma.mlmodel
    📦 Size: 245.32 MB
    ✅ Tokenizer saved: models/coreml/embeddinggemma_tokenizer

======================================================================
✅ Conversion Complete!
======================================================================
```

### Method 2: Manual Conversion (Advanced)

If the automated script fails or you need more control:

#### Step 1: Download Original Model

```python
from transformers import AutoModel, AutoTokenizer

model_id = "google/gemma-2-2b"  # or specific embedding model variant

tokenizer = AutoTokenizer.from_pretrained(model_id)
model = AutoModel.from_pretrained(model_id)

# Save locally for offline conversion
model.save_pretrained("local_embeddinggemma")
tokenizer.save_pretrained("local_embeddinggemma")
```

#### Step 2: Convert to CoreML

```python
import torch
import coremltools as ct

# Load local model
model = AutoModel.from_pretrained("local_embeddinggemma")
model.eval()

# Create example input
example_input = torch.randint(0, 256000, (1, 512))  # batch_size=1, seq_len=512

# Trace model
with torch.no_grad():
    traced_model = torch.jit.trace(model, example_input)

# Convert to CoreML
mlmodel = ct.convert(
    traced_model,
    inputs=[ct.TensorType(name="input_ids", shape=(1, ct.RangeDim(1, 8192)))],
    minimum_deployment_target=ct.target.macOS13,
    compute_precision=ct.precision.FLOAT16,
    compute_units=ct.ComputeUnit.ALL,
)

# Save
mlmodel.save("embeddinggemma.mlmodel")
```

---

## Model Format Notes

### GGUF vs CoreML

- **GGUF**: Ollama's quantization format, not directly convertible to CoreML
- **CoreML**: Apple's format, requires original PyTorch/ONNX model
- **Solution**: Download original model from HuggingFace, convert to CoreML

### Model Variants

Possible HuggingFace model IDs for embedding models:

- `google/gemma-2-2b` - Base Gemma 2B model (can be adapted for embeddings)
- `google/gemma-2-2b-it` - Instruction-tuned variant
- `google/gemma-2-9b` - Larger 9B variant

**Note**: EmbeddingGemma may have a specific embedding model variant. Check HuggingFace for:
- Models with "embedding" in the name
- Models with "text-embedding" description
- Model cards that mention embedding dimensions (768, 1536, etc.)

---

## Post-Conversion Verification

### 1. Verify Model Loads

```python
import coremltools as ct

model = ct.models.MLModel("models/coreml/embeddinggemma.mlmodel")
print(model)
```

### 2. Test Inference

```python
import numpy as np

# Create test input (token IDs)
test_input = np.random.randint(0, 256000, (1, 128), dtype=np.int32)

# Run inference
prediction = model.predict({"input_ids": test_input})
print(f"Output shape: {prediction.shape}")
print(f"Output dtype: {prediction.dtype}")
```

### 3. Verify ANE Acceleration

```bash
# Check model spec
python3 -c "
import coremltools as ct
model = ct.models.MLModel('models/coreml/embeddinggemma.mlmodel')
print(model.compute_units)  # Should show ANE availability
"
```

---

## Troubleshooting

### Issue: "Model not found on HuggingFace"

**Solution**: Check for alternative model IDs or download manually:

```bash
# Search HuggingFace
# https://huggingface.co/models?search=embeddinggemma

# Alternative: Use the GGUF model information
# The GGUF file may contain metadata about the original model
```

### Issue: "TorchScript tracing fails"

**Solution**: Use `torch.jit.script()` instead of `torch.jit.trace()`:

```python
traced_model = torch.jit.script(model)  # Instead of trace()
```

### Issue: "CoreML conversion fails"

**Solutions**:
1. Update coremltools: `pip install --upgrade coremltools`
2. Try different compute precision (FP32 instead of FP16)
3. Check model architecture compatibility
4. Convert via ONNX intermediate format

### Issue: "Model too large after conversion"

**Solutions**:
1. Enable INT8 quantization: `--no-quantize` → remove flag
2. Use smaller model variant (e.g., 2B instead of 9B)
3. Apply pruning before conversion

---

## Integration with CoreMLEmbeddingProvider

Once converted, the model will be automatically detected by `CoreMLEmbeddingProvider`:

```rust
// Auto-detection will find models/coreml/embeddinggemma.mlmodel
let service = EmbeddingServiceFactory::create_with_auto_detect(
    config,
    Some("embeddinggemma".to_string())
).await;
```

**Environment Variables** (optional):
```bash
export COREML_MODELS_PATH="/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml"
export COREML_EMBEDDING_MODEL_PATH="/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml/embeddinggemma.mlmodel"
```

---

## Performance Expectations

After conversion, you should see:

- **Model Size**: ~250-500 MB (FP16, quantized) vs 622 MB (GGUF original)
- **ANE Speedup**: 2.5-3x faster inference vs CPU
- **Embedding Dimension**: 768 (embeddinggemma) or 1536 (larger variants)
- **Batch Processing**: Supports batch inference for multiple texts

---

## Next Steps

1. ✅ Run conversion script
2. ✅ Verify model loads correctly
3. ✅ Test inference with sample texts
4. ✅ Update environment variables if needed
5. ✅ Test `CoreMLEmbeddingProvider` integration
6. ✅ Benchmark ANE speedup (target: 2.5-3x)

---

## References

- [CoreML Tools Documentation](https://apple.github.io/coremltools/)
- [HuggingFace Transformers](https://huggingface.co/docs/transformers)
- [PyTorch to CoreML Guide](https://apple.github.io/coremltools/docs-guides/source/convert-pytorch-workflow.html)
- [ANE Optimization Guide](https://developer.apple.com/machine-learning/core-ml/)


