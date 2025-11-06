# Core ML Module Refactor Plan

## Overview

The `coreml.rs` file has grown to 2614 lines and contains multiple concerns that should be separated into focused modules. This refactor will improve maintainability, testability, and code organization.

## Current Structure Analysis

The monolithic `coreml.rs` contains:

1. **Basic Types** (lines 17-205): MLModel, MLFeatureProvider, MLMultiArray, etc.
2. **MLModel Operations** (lines 207-600): Model loading, saving, prediction
3. **Mistral Tokenizer** (lines 856-1062): Text encoding/decoding functions
4. **Core ML Framework Interface** (lines 1069-2215): ModelRef, registry, inference, FFI declarations
5. **Testing Infrastructure** (lines 2217-2416): InferenceTestResults and testing methods
6. **KV Cache Management** (lines 2419-2582): KvStateHandle implementation
7. **Tests** (lines 2586-2613): Unit tests

## Proposed Module Structure

### 1. `types.rs` - Core ML Type Definitions
**Purpose**: Basic data structures and types for Core ML operations

**Contents**:
- `MLModel` struct
- `MLModelConfiguration` and `MLComputeUnits`
- `MLMultiArray` and `MLMultiArrayDataType`
- `MLFeatureProvider`, `MLDictionaryFeatureProvider`, `MLFeatureValue`, `MLFeatureType`
- `KvStateHandle`
- Type implementations (Default, constructors, etc.)

### 2. `model.rs` - MLModel Operations
**Purpose**: Core MLModel functionality and operations

**Contents**:
- `MLModel` implementation (loading, saving, prediction)
- Model compilation functions
- Model information retrieval
- Drop implementations

### 3. `tokenizer.rs` - Mistral Tokenizer
**Purpose**: Text tokenization and encoding/decoding

**Contents**:
- `mistral_encode()`, `mistral_decode()`, `mistral_free_string()`
- Legacy FFI-style function aliases
- Tokenizer management functions

### 4. `coreml_module.rs` - Framework Interface
**Purpose**: Core ML framework integration and FFI

**Contents**:
- `ModelRef` and `CoreMlHandle`
- Thread-local registry (`ModelRegistry`)
- Inference functions (`run_inference`)
- FFI declarations and extern "C" functions
- Model loading/compilation utilities

### 5. `registry.rs` - Handle Management
**Purpose**: Thread-local model handle registry

**Contents**:
- `ModelRegistry` struct and operations
- Thread-local storage management
- `registry` module with safe operations
- Handle registration/unregistration

### 6. `inference.rs` - Inference Operations
**Purpose**: High-level inference operations and utilities

**Contents**:
- Inference testing infrastructure
- Performance measurement
- Input/output tensor handling
- Inference result processing

### 7. `kv_cache.rs` - KV Cache Management
**Purpose**: Key-value cache state management for efficient inference

**Contents**:
- `KvStateHandle` implementation
- KV cache creation, stepping, resetting
- KV-aware inference extensions

### 8. `safety.rs` - Safety and Validation
**Purpose**: Memory safety and I/O validation

**Contents**:
- `io_safety` module (tensor validation, conversion)
- Memory safety checks
- Input validation helpers

### 9. `testing.rs` - Testing Infrastructure
**Purpose**: Performance testing and measurement

**Contents**:
- `InferenceTestResults` struct and methods
- Testing utilities and helpers
- Performance benchmarking functions

## Refactor Execution Plan

### Phase 1: Extract Types (`types.rs`)
1. Create `types.rs` with basic type definitions
2. Move `MLModel`, `MLModelConfiguration`, `MLComputeUnits`
3. Move `MLMultiArray`, `MLFeatureProvider`, etc.
4. Update imports in main `coreml.rs`

### Phase 2: Extract Model Operations (`model.rs`)
1. Create `model.rs` with MLModel implementation
2. Move model loading, saving, prediction methods
3. Move model compilation functions
4. Update main module to re-export

### Phase 3: Extract Tokenizer (`tokenizer.rs`)
1. Create `tokenizer.rs` for Mistral functions
2. Move all tokenizer-related functions
3. Maintain backward compatibility
4. Update module exports

### Phase 4: Extract Core Framework (`coreml_module.rs`)
1. Create `coreml_module.rs` for framework interface
2. Move the nested `coreml` module
3. Handle FFI declarations and extern functions
4. Update module structure

### Phase 5: Extract Registry (`registry.rs`)
1. Create `registry.rs` for handle management
2. Move `ModelRegistry` and thread-local storage
3. Extract registry operations
4. Update imports

### Phase 6: Extract Safety (`safety.rs`)
1. Create `safety.rs` for validation
2. Move `io_safety` module
3. Add additional safety utilities
4. Update module references

### Phase 7: Extract Testing (`testing.rs`)
1. Create `testing.rs` for test infrastructure
2. Move `InferenceTestResults` and testing methods
3. Add testing utilities
4. Update test imports

### Phase 8: Extract KV Cache (`kv_cache.rs`)
1. Create `kv_cache.rs` for cache management
2. Move `KvStateHandle` implementation
3. Move KV-aware inference extensions
4. Update module structure

### Phase 9: Update Main Module
1. Update `coreml.rs` to re-export from modules
2. Maintain API compatibility
3. Add module documentation
4. Verify all tests pass

## Benefits

### Maintainability
- Each module has a single responsibility
- Easier to locate and modify specific functionality
- Reduced cognitive load when working on specific features

### Testability
- Focused unit tests for each module
- Clear boundaries for mocking and isolation
- Easier to test individual components

### Code Organization
- Logical separation of concerns
- Clear module boundaries
- Easier onboarding for new developers

### Performance
- Better compilation parallelism (separate modules compile independently)
- Reduced recompilation scope for changes
- Smaller dependency graphs for incremental builds

## Migration Strategy

### API Compatibility
- All public APIs remain unchanged
- Re-exports maintain backward compatibility
- No breaking changes for consumers

### Testing Strategy
- Run full test suite after each extraction
- Verify performance characteristics unchanged
- Ensure memory safety and threading behavior preserved

### Documentation Updates
- Update module-level documentation
- Maintain comprehensive API docs
- Add cross-references between modules

## Risk Mitigation

### Thread Safety
- Thread-local registry operations remain unchanged
- FFI safety guarantees preserved
- Memory management patterns maintained

### Performance Impact
- Measure compilation time changes
- Verify inference performance unchanged
- Monitor memory usage patterns

### Compilation Issues
- Handle circular dependencies carefully
- Maintain proper import hierarchies
- Use feature flags for optional components

## Success Criteria

- [ ] All tests pass after refactor
- [ ] Compilation time within 10% of original
- [ ] Memory usage unchanged
- [ ] API compatibility maintained
- [ ] Code coverage maintained or improved
- [ ] No new linting warnings
- [ ] Documentation updated and accurate

## File Size Reduction

| Module | Lines | Purpose |
|--------|-------|---------|
| `types.rs` | ~200 | Basic type definitions |
| `model.rs` | ~400 | Model operations |
| `tokenizer.rs` | ~200 | Text tokenization |
| `coreml_module.rs` | ~1000 | Framework interface |
| `registry.rs` | ~150 | Handle management |
| `safety.rs` | ~300 | Validation and safety |
| `testing.rs` | ~250 | Test infrastructure |
| `kv_cache.rs` | ~200 | Cache management |
| `coreml.rs` | ~150 | Main module (re-exports) |

Total: ~2850 lines (slight increase due to module overhead, but much better organization)
