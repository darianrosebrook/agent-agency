# Model Surgery Progress Report

## Successfully Replaced Operations

### ✅ bitwise_or → logical_or
- **Status**: Successfully replaced
- **Method**: Global monkey-patching of `torch.bitwise_or` and `torch.Tensor.__or__`
- **Result**: CoreML conversion no longer fails on bitwise_or

### ✅ new_ones → ones (partial)
- **Status**: Partially working
- **Issue**: dtype preservation problem
- **Error**: `Op "fill_0" expects tensor or scalar of dtype from type domain ['int32'] but got tensor[0,fp32]`

## Current Blocker

The `new_ones` replacement creates tensors with incorrect dtype. CoreML expects int32 but receives float32.

### Root Cause
When `tensor.new_ones()` is called with integer dtype, our replacement uses `torch.ones()` which may not preserve integer dtypes correctly, or CoreML's fill operation has strict dtype requirements.

### Next Steps

1. **Investigate dtype preservation**: Check what dtype `new_ones` actually uses in the traced model
2. **Alternative replacement**: Use `torch.full` instead of `torch.ones` for better dtype control
3. **Direct tensor creation**: Create tensors directly with proper dtype without using fill operations

## Model Surgery Architecture

The surgery system works by:
1. **Global patching**: Replacing operations before model loading
2. **Operation replacement**: Substituting CoreML-incompatible operations with compatible ones
3. **Dtype preservation**: Attempting to maintain original tensor dtypes

## Files Modified

- `models/scripts/convert_embeddinggemma_to_coreml.py`: Added operation patching
- `models/scripts/coreml_surgery.py`: Created surgery utilities module

## Testing

```bash
source venv-py39/bin/activate
python3 models/scripts/convert_embeddinggemma_to_coreml.py \
  --model-id models/coreml/embeddinggemma-300m-raw \
  --output-dir models/coreml
```

## Progress Metrics

- Operations replaced: 2/3 (bitwise_or ✅, new_ones ⚠️, dtype issue 🔄)
- Conversion progress: ~1.4% (66/4743 ops converted before failure)
- Blocking issue: dtype mismatch in fill operation



