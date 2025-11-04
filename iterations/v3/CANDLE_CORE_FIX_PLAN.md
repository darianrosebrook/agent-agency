# Candle-Core Dependency Conflicts - Fix Plan

**Status**: Analysis Complete - Root Causes Identified
**Impact**: Blocking CoreML inference, YOLO, Whisper, and Mistral functions

---

## Root Cause Analysis

### Primary Issue: Tokenizers Version Conflicts

**Version Mismatch**:
- `system-acceleration`: `tokenizers = "0.19.1"`
- `data-infrastructure`: `tokenizers = "0.15.2"`

**Impact**: Workspace cannot resolve compatible tokenizers version, causing:
1. CoreML inference execution disabled (fallback to placeholder tensors)
2. YOLO module disabled
3. Whisper inference disabled
4. Mistral inference functions disabled

### Secondary Issues

1. **Half-precision (fp16) conversion disabled**
   - Location: `system-acceleration/src/ane/infer/execute.rs:180-201`
   - Issue: `half` crate conflicts preventing fp16 optimization
   - Impact: ANE acceleration less efficient

2. **Model loading conflicts**
   - Multiple crates using different candle-core versions
   - Potential runtime compatibility issues

---

## Fix Strategies

### Strategy 1: Version Alignment (Recommended)

**Align tokenizers versions across workspace:**

```toml
# In workspace Cargo.toml - add version constraints
[workspace.dependencies]
tokenizers = "0.19"  # Use latest version

# Update individual crate Cargo.toml files
# system-acceleration/Cargo.toml
tokenizers = { workspace = true }  # Uses 0.19

# data-infrastructure/Cargo.toml
tokenizers = { workspace = true }  # Uses 0.19
```

**Pros:**
- Resolves version conflicts
- Enables all CoreML features
- Maintains compatibility

**Cons:**
- May require code changes for API differences between v0.15 and v0.19
- Potential breaking changes

### Strategy 2: Feature Gating

**Separate tokenizers usage by features:**

```toml
# system-acceleration/Cargo.toml
[features]
default = []
coreml = ["tokenizers"]
yolo = ["tokenizers", "image"]

[dependencies]
tokenizers = { version = "0.19", optional = true }
# ... other deps

# data-infrastructure/Cargo.toml
[features]
default = ["embeddings"]
embeddings = ["tokenizers"]

[dependencies]
tokenizers = { version = "0.15", optional = true }
```

**Pros:**
- Allows different tokenizers versions
- Feature flags control functionality

**Cons:**
- Complex feature flag management
- Runtime checks needed for feature availability

### Strategy 3: Dependency Isolation

**Use separate dependency graphs:**

```toml
# Create separate workspace members for conflicting crates
# Move system-acceleration to separate workspace
# Keep data-infrastructure in main workspace
```

**Pros:**
- No version conflicts
- Independent dependency management

**Cons:**
- Integration complexity
- Duplicated dependencies
- Build complexity

---

## Implementation Plan

### Phase 1: Version Alignment (High Priority)

1. **Update workspace dependencies**
   ```toml
   # Cargo.toml
   [workspace.dependencies]
   tokenizers = "0.19"
   half = "2.4"
   candle-core = "0.9"
   candle-nn = "0.9"
   candle-transformers = "0.9"
   ```

2. **Update crate dependencies to use workspace versions**
   ```toml
   # system-acceleration/Cargo.toml
   tokenizers = { workspace = true }
   half = { workspace = true }
   candle-core = { workspace = true }
   candle-nn = { workspace = true }
   candle-transformers = { workspace = true }
   
   # data-infrastructure/Cargo.toml
   tokenizers = { workspace = true }
   candle-core = { workspace = true }
   candle-transformers = { workspace = true }
   half = { workspace = true }
   ```

3. **Update agent-research to use workspace versions**
   ```toml
   # agent-research/Cargo.toml
   tokenizers = { workspace = true }
   candle-core = { workspace = true }
   candle-nn = { workspace = true }
   candle-transformers = { workspace = true }
   ```

### Phase 2: API Migration (If Needed)

**Check for breaking API changes between tokenizers v0.15 → v0.19:**

1. **Tokenizer initialization**
   ```rust
   // Check if API changed
   let tokenizer = Tokenizer::from_file("model.json")?;
   ```

2. **Encoding/decoding methods**
   ```rust
   // Check if encode/decode methods changed
   let encoding = tokenizer.encode("text", true)?;
   let decoded = tokenizer.decode(&encoding.get_ids(), true)?;
   ```

### Phase 3: Re-enable CoreML Features

1. **Uncomment inference execution**
   ```rust
   // system-acceleration/src/ane/infer/execute.rs:242-250
   let output_tensor = crate::ane::compat::coreml::coreml::run_inference(
       model.model_ref,
       &model.input_name,
       &prepared_input,
       &model.input_shape,
   )?;
   ```

2. **Re-enable YOLO module**
   ```rust
   // system-acceleration/src/ane/infer/mod.rs
   pub mod yolo;
   pub use yolo::{YOLOInferenceExecutor, create_yolo_executor};
   ```

3. **Re-enable precision conversion**
   ```rust
   // system-acceleration/src/ane/infer/execute.rs:180-201
   if let Some(precision) = &options.precision {
       match precision.as_str() {
           "fp16" => {
               prepared_input = prepared_input
                   .iter()
                   .map(|&x| half::f16::from_f32(x).to_f32())
                   .collect();
           }
           "fp32" => {
               // Keep as f32
           }
           _ => {
               return Err(ANEError::InvalidInput(format!(
                   "Unsupported precision: {}",
                   precision
               )));
           }
       }
   }
   ```

4. **Re-enable Mistral inference functions**
   ```rust
   // system-acceleration/src/ane/manager.rs:770,809,846
   // Uncomment the MistralInferenceOptions functions
   ```

### Phase 4: Testing & Validation

1. **Test CoreML inference pipeline**
   ```bash
   cargo test -p system-acceleration coreml
   ```

2. **Test YOLO functionality**
   ```bash
   cargo test -p system-acceleration yolo
   ```

3. **Test workspace compilation**
   ```bash
   cargo check --workspace
   ```

4. **Test precision conversion**
   ```bash
   cargo test -p system-acceleration precision
   ```

---

## Alternative Quick Fix

If version alignment is too complex, implement a **compatibility layer**:

```rust
// Create compatibility module
pub mod tokenizers_compat {
    #[cfg(feature = "tokenizers-0_19")]
    pub use tokenizers_0_19::*;
    
    #[cfg(feature = "tokenizers-0_15")]
    pub use tokenizers_0_15::*;
}
```

---

## Success Criteria

- [ ] `cargo check --workspace` passes
- [ ] `cargo test -p system-acceleration` passes
- [ ] YOLO inference works
- [ ] CoreML inference uses real tensors, not placeholders
- [ ] Precision conversion (fp16) works
- [ ] Mistral inference functions enabled
- [ ] No tokenizers version conflicts

---

## Risk Assessment

**High Risk**: Version alignment may break existing tokenizers usage
**Medium Risk**: API changes between tokenizers versions
**Low Risk**: Feature gating approach (more complex but safer)

**Recommended**: Start with version alignment, rollback to feature gating if issues arise.

---

## Timeline

- **Phase 1**: 1-2 hours (dependency updates)
- **Phase 2**: 2-4 hours (API migration if needed)
- **Phase 3**: 1-2 hours (re-enable features)
- **Phase 4**: 2-4 hours (testing and validation)

**Total**: 6-12 hours depending on API migration complexity.
