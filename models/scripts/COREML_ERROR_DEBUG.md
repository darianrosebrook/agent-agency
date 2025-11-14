# CoreML Error -5 Debugging Notes

## Issue

CoreML model loads successfully but inference fails with error code -5:
```
Unable to compute the prediction using a neural network model. 
It can be an invalid input data or broken/unsupported model (error code: -5).
```

## Model Status

- ✅ Model converts successfully (578 MB)
- ✅ Model loads without errors
- ✅ Input specification correct (INT32, shape range [1, 1..2048])
- ❌ Inference fails with error -5

## Possible Causes

### 1. ML Program Runtime Compilation

ML Program models (.mlpackage) require runtime compilation on first use. This might fail if:
- Model has incompatible operations
- Device doesn't support ML Program format
- Model structure has issues

### 2. Input Shape Mismatch

Model spec shows:
- Fixed shape: [1, 1]
- ShapeRange: [1, 1] and [1, 2048]

This suggests the model might expect exactly [1, 1] shape, not variable-length inputs.

### 3. Model Architecture Issues

The traced model works correctly in PyTorch, but CoreML conversion might have:
- Missing operations
- Unsupported operations
- Incorrect output shape inference

## Next Steps

1. **Test with fixed shape [1, 1]**: See if model works with exact fixed shape
2. **Check ML Program compilation**: Verify if model compiles correctly
3. **Compare with working CoreML models**: Check what's different
4. **Alternative: Convert full SentenceTransformer pipeline**: Include pooling/normalization layers

## Workaround Options

1. **Use ONNX intermediate format**: Convert PyTorch → ONNX → CoreML
2. **Convert Neural Network format**: Use legacy format instead of ML Program
3. **Include full pipeline**: Convert SentenceTransformer with all layers



