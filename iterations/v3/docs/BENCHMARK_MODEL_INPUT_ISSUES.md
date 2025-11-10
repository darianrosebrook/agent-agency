# Benchmark Model Input Format Issues

## Summary

The CoreML performance benchmarks are encountering model-specific input format requirements that need to be resolved.

## Mistral 7B FP16 Model

### Current Status
- Model loads successfully
- Inference fails due to input shape/format mismatches

### Discovered Requirements
- Feature names: `inputIds` (camelCase) and `causalMask` (camelCase)
- Sequence length: Must be 1 for stateful models
- Shape requirements: **Inconsistent error messages**
  - Sometimes: "MultiArray 4-d shape is not allowed, expected 2-d"
  - Sometimes: "MultiArray 2-d shape is not allowed, expected 4-d"

### Root Cause
The model's actual input schema is not being queried. We're guessing the format based on error messages, which leads to inconsistent results.

### Solution Required
1. Implement model metadata querying to get actual input schema
2. Parse the JSON returned by `agentbridge_model_get_info` to extract:
   - Input feature names
   - Input shapes and dimensions
   - Data types
3. Use the actual schema to create correct inputs

### Current Workaround
- Using 2D shape `[batch_size, sequence_length]` = `[1, 1]`
- Feature names: `inputIds` and `causalMask` (camelCase)
- Data type: `f32` (FFI bridge limitation - only supports float32)

## FastViT T8 F16 Model

### Current Status
- ✅ **IMPLEMENTED** - Image feature support added to FFI bridge
- Tests enabled in benchmarks

### Implementation
- ✅ Added `agentbridge_dict_provider_set_feature_image` to Swift bridge
- ✅ Added corresponding FFI declaration in Rust (`model.rs`, `coreml_direct.rs`)
- ✅ Updated `coreml_direct.rs` to handle `MLFeatureValue::Image` features
- ✅ Swift bridge converts raw RGB bytes to `CVPixelBuffer` and creates `MLFeatureValue`
- ✅ Image dimension inference from data length (assumes RGB, 3 bytes per pixel)

### Testing
- FastViT tests are now enabled in `ane_performance_benchmarks.rs`
- Model info updated to use `"image"` dtype to trigger Image feature type

## Ingestors Status

### Implementation Status
All core ingestors are implemented:
- ✅ FileIngestor
- ✅ UrlIngestor
- ✅ StreamIngestor
- ✅ DatabaseIngestor
- ✅ ApiIngestor
- ✅ Specialized: CaptionsIngestor, DiagramsIngestor, VideoIngestor, SlidesIngestor

### TODOs (Enhancements, not blockers)
- Full ASS/SSA subtitle format parser
- Proper PDF page detection and content extraction
- Proper glob pattern matching library
- Mock HTTP server for comprehensive testing
- Clock dependency injection for deterministic testing

These are enhancements, not placeholders. Core functionality is implemented.

## Next Steps

1. **High Priority**: Implement model metadata querying to resolve Mistral input format
2. **Medium Priority**: Add Image feature support to FFI bridge for FastViT
3. **Low Priority**: Enhance ingestors with additional format support

