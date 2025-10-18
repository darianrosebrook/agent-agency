# CoreMLBridge

Swift C-ABI bridge for Core ML integration with Rust.

This package provides a minimal C interface to Core ML APIs, isolating all Objective-C and ARC complexity on the Swift side. Each function wraps exceptions and manages autoreleasepool to ensure safe FFI boundaries.

## Building

```bash
swift build --configuration release
```

Produces `libCoreMLBridge.a` static library in `.build/release/`.

## Functions

### `coreml_compile_model`
Compile a `.mlmodel` file to `.mlmodelc` bundle.
- Input: model path, compute units (0=All, 1=CPU, 2=CPU+GPU, 3=CPU+ANE)
- Output: compiled directory path, error string
- Returns: 0 on success, 1 on failure

### `coreml_load_model`
Load a compiled `.mlmodelc` bundle into memory.
- Input: compiled directory, compute units
- Output: opaque model handle, error string
- Returns: 0 on success, 1 on failure

### `coreml_model_schema`
Query model input/output schema.
- Input: model handle
- Output: JSON schema, error string
- Returns: 0 on success, 1 on failure

### `coreml_predict`
Run single inference.
- Input: model handle, inputs JSON, timeout (ms)
- Output: outputs JSON, error string
- Returns: 0 on success, 1 on failure

### `coreml_free_model`
Release model handle.

### `coreml_free_cstr`
Free C strings allocated by bridge.

## Notes

- All string returns must be freed by caller
- Autoreleasepool wraps every entry point
- No ObjC exceptions cross FFI boundary
- Error codes + localized descriptions only
