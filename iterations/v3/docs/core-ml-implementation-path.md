# Core ML Implementation Path: De-Risked Tactics

**Author**: @darianrosebrook  
**Status**: Ready for implementation (Q2 2025+)  
**Risk Level**: MEDIUM → LOW (when following this path)  
**Approach**: Swift C ABI bridge, feature-flagged, with telemetry gates  

## Executive Summary

This document provides a **concrete, tactical implementation path** for Core ML integration that:
- ✅ Avoids painting Rust into FFI corners
- ✅ Keeps ObjC/Swift complexity quarantined in Swift
- ✅ Enables safe rollback (feature flags)
- ✅ Measures before scaling (telemetry gates)

**TL;DR**: Build a **thin Swift→C ABI bridge**, not raw objc2. Link via `build.rs`. Feature-gate behind `#[cfg(coreml)]`. Measure compile/inference/ANE coverage. Never cross the boundary without timeouts or autorelease pools.

---

## 1. Stable Abstraction Seam (Implement First)

**Before any FFI**, nail down these Rust-side contracts so Core ML, Candle, or Metal can swap cleanly later.

### Inference Engine Trait

```rust
/// Universal inference abstraction (CPU, GPU, ANE all implement this)
pub trait InferenceEngine: Send + Sync {
    /// Load a model and prepare it for inference
    fn prepare(
        &self,
        artifact: ModelArtifact,
        options: PrepareOptions,
    ) -> Result<Arc<dyn PreparedModel>>;
    
    /// Run inference with timeout and optional streaming output
    fn infer(
        &self,
        prepared: &dyn PreparedModel,
        inputs: TensorMap<'_>,
        timeout: Duration,
        stream: Option<InferenceStream>,
    ) -> Result<TensorMap>;
    
    /// Query backend capabilities (device class, precision support, max batch)
    fn capabilities(&self, model: &dyn PreparedModel) -> CapabilityReport;
    
    /// Check if a specific model can use ANE (for telemetry)
    fn ane_eligible(&self, model: &dyn PreparedModel) -> bool;
}

pub trait PreparedModel: Send + Sync {
    /// Unique cache key: authoring hash + target platform + quantization
    fn cache_key(&self) -> String;
    
    /// Input/output shape and dtype constraints
    fn io_schema(&self) -> IoSchema;
    
    /// Best-guess latency (for circuit breaker thresholds)
    fn sla_estimate(&self) -> Duration;
}

/// Describes model in storage (author format vs compiled binary)
#[derive(Clone, Debug)]
pub enum ModelArtifact {
    /// Original authoring format (ONNX, safetensors, etc.)
    Authoring {
        format: ModelFormat,
        data: Arc<[u8]>, // or path
    },
    /// Pre-compiled, ready to load
    Compiled {
        platform: Platform,
        path: PathBuf, // .mlmodelc directory (macOS)
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelFormat {
    ONNX,
    Safetensors,
    CoreML,
    Candle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    MacOS,
    Linux,
    Web,
}

pub struct PrepareOptions {
    pub target: ComputeTarget,
    pub quantization: Option<QuantizationMethod>,
    pub cache_dir: PathBuf,
    pub timeout: Duration,
}

#[derive(Clone, Copy, Debug)]
pub enum ComputeTarget {
    CPU,
    GPU,
    ANE,
    Auto, // Let backend decide
}

pub struct CapabilityReport {
    pub device_class: String, // "M1 Pro", "Candle CPU", etc.
    pub supported_dtypes: Vec<DType>,
    pub max_batch_size: Option<usize>,
    pub ane_available: bool,
    pub ane_operators_covered: f32, // % of ops that will run on ANE
}

pub struct IoSchema {
    pub inputs: HashMap<String, TensorSpec>,
    pub outputs: HashMap<String, TensorSpec>,
}

pub struct TensorSpec {
    pub shape: Shape,
    pub dtype: DType,
    pub contiguous: bool,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Fixed(Vec<usize>),                     // [batch, seq, hidden]
    Flexible(Vec<Option<usize>>),          // [batch, None, hidden]
    DynamicWithBounds(Vec<(usize, usize)>), // [batch (1-128), ...]
}
```

**Why this matters:**
- Core ML impl plugs in as one backend; Candle as another
- Public API never mentions ObjC/Swift/ARC
- Tests can mock `InferenceEngine` without touching native code
- You can A/B compare backends with identical interface

### Artifact Cache Key Strategy

```rust
pub fn make_cache_key(
    authoring_sha256: &str,
    platform: Platform,
    compute_target: ComputeTarget,
    quantization: &Option<QuantizationMethod>,
    io_shapes: &IoSchema,
) -> String {
    // E.g., "abc123def456:macos:ane:int8:b1-s512-h768"
    // Ensures cache invalidation on:
    // - Model content changes
    // - Platform/target changes
    // - Quantization changes
    // - I/O shape "modes" (e.g., batch size ranges)
    format!(
        "{}:{}:{}:{}:{}",
        authoring_sha256,
        platform as u8,
        compute_target as u8,
        quantization.as_ref().map(|q| q.tag()).unwrap_or("none"),
        io_shapes.shape_summary()
    )
}
```

---

## 2. Swift C ABI Bridge (The Safe Path)

**Why Swift+C over raw objc2?**
- ✅ All unsafe code lives in Swift (one place to audit)
- ✅ Apple API churn absorbed in Swift layer
- ✅ No ObjC lifetime magic bleeding into Rust
- ✅ C ABI is stable across macOS releases

### Swift Bridge Structure

**File: `coreml-bridge/Package.swift`**

```swift
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CoreMLBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "CoreMLBridge",
            type: .static,
            targets: ["CoreMLBridge"]
        ),
    ],
    targets: [
        .target(
            name: "CoreMLBridge",
            dependencies: [],
            path: "Sources",
            publicHeadersPath: "include"
        ),
    ]
)
```

**File: `coreml-bridge/Sources/CoreMLBridge.swift`**

```swift
import Foundation
import CoreML

// MARK: - Opaque Handle Management
//
// We wrap MLModel in Unmanaged to maintain ARC across the FFI boundary
// without exposing ObjC details to Rust.

/// Compile an ONNX/safetensors model to .mlmodelc
///
/// # Parameters
/// - `modelPathC`: C string path to source model
/// - `computeUnits`: Int32 mapping to MLComputeUnits
///   0 = .all, 1 = .cpuOnly, 2 = .cpuAndGPU, 3 = .cpuAndANE
/// - `outCompiledPath`: Output C string (caller must free via coreml_free_string)
/// - `outErr`: Output error message (caller must free via coreml_free_string)
///
/// # Returns
/// 0 on success, 1 on failure
@_cdecl("coreml_compile_model")
public func coreml_compile_model(
    modelPathC: UnsafePointer<CChar>,
    computeUnits: Int32,
    outCompiledPath: UnsafeMutablePointer<UnsafePointer<CChar>?>,
    outErr: UnsafeMutablePointer<UnsafePointer<CChar>?>
) -> Int32 {
    do {
        let modelPath = String(cString: modelPathC)
        let url = URL(fileURLWithPath: modelPath)
        
        // Only one thing to pass: the compute units preference
        let cfg = MLModelConfiguration()
        cfg.computeUnits = computeUnitsFromInt32(computeUnits)
        
        // Compile. This writes a .mlmodelc bundle to a system cache or tmp.
        let compiledURL = try MLModel.compileModel(at: url)
        
        // Return the path as a C string (caller frees)
        outCompiledPath.pointee = strdup(compiledURL.path)
        outErr.pointee = nil
        return 0
    } catch {
        outCompiledPath.pointee = nil
        outErr.pointee = strdup("Compilation failed: \(error)")
        return 1
    }
}

/// Load a compiled .mlmodelc into an opaque handle
///
/// # Returns
/// Opaque pointer to retained MLModel (caller must call coreml_free_model)
@_cdecl("coreml_load_model")
public func coreml_load_model(
    compiledModelDirC: UnsafePointer<CChar>,
    computeUnits: Int32,
    outHandle: UnsafeMutablePointer<OpaquePointer?>,
    outErr: UnsafeMutablePointer<UnsafePointer<CChar>?>
) -> Int32 {
    do {
        let dirPath = String(cString: compiledModelDirC)
        let url = URL(fileURLWithPath: dirPath)
        
        let cfg = MLModelConfiguration()
        cfg.computeUnits = computeUnitsFromInt32(computeUnits)
        
        let model = try MLModel(contentsOf: url, configuration: cfg)
        
        // Retain the model for Rust; Rust must call coreml_free_model to release
        let retained = Unmanaged.passRetained(model as AnyObject)
        outHandle.pointee = OpaquePointer(retained.toOpaque())
        outErr.pointee = nil
        return 0
    } catch {
        outHandle.pointee = nil
        outErr.pointee = strdup("Load failed: \(error)")
        return 1
    }
}

/// Release a model handle
@_cdecl("coreml_free_model")
public func coreml_free_model(handle: OpaquePointer?) {
    guard let h = handle else { return }
    Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).release()
}

/// Query model input/output schema
///
/// Returns JSON describing inputs/outputs (names, shapes, dtypes)
@_cdecl("coreml_model_schema")
public func coreml_model_schema(
    handle: OpaquePointer?,
    outSchemaJson: UnsafeMutablePointer<UnsafePointer<CChar>?>,
    outErr: UnsafeMutablePointer<UnsafePointer<CChar>?>
) -> Int32 {
    guard let h = handle,
          let model = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).takeUnretainedValue() as? MLModel
    else {
        outErr.pointee = strdup("Invalid model handle")
        return 1
    }
    
    do {
        var schema = [String: Any]()
        
        // Extract input descriptions
        var inputs = [String: [String: Any]]()
        for (name, desc) in model.modelDescription.inputDescriptionsByName {
            inputs[name] = descriptionToDict(desc)
        }
        schema["inputs"] = inputs
        
        // Extract output descriptions
        var outputs = [String: [String: Any]]()
        for (name, desc) in model.modelDescription.outputDescriptionsByName {
            outputs[name] = descriptionToDict(desc)
        }
        schema["outputs"] = outputs
        
        let jsonData = try JSONSerialization.data(withJSONObject: schema)
        let jsonStr = String(data: jsonData, encoding: .utf8) ?? "{}"
        outSchemaJson.pointee = strdup(jsonStr)
        outErr.pointee = nil
        return 0
    } catch {
        outErr.pointee = strdup("Schema extraction failed: \(error)")
        return 1
    }
}

/// Perform inference
///
/// # Parameters
/// - `inputsJson`: JSON describing input tensors
///   `{ "input_name": { "shape": [1, 512], "dtype": "float32", "data": "base64..." } }`
/// - `outOutputsJson`: Populated with output JSON on success
/// - `timeout`: Timeout in milliseconds (0 = no timeout)
///
/// # Returns
/// 0 on success, 1 on failure or timeout
@_cdecl("coreml_predict")
public func coreml_predict(
    handle: OpaquePointer?,
    inputsJsonC: UnsafePointer<CChar>,
    timeoutMs: Int32,
    outOutputsJsonC: UnsafeMutablePointer<UnsafePointer<CChar>?>,
    outErr: UnsafeMutablePointer<UnsafePointer<CChar>?>
) -> Int32 {
    guard let h = handle,
          let model = Unmanaged<AnyObject>.fromOpaque(UnsafeRawPointer(h)).takeUnretainedValue() as? MLModel
    else {
        outErr.pointee = strdup("Invalid model handle")
        return 1
    }
    
    do {
        let inputsStr = String(cString: inputsJsonC)
        guard let inputsData = inputsStr.data(using: .utf8),
              let inputsDict = try JSONSerialization.jsonObject(with: inputsData) as? [String: Any]
        else {
            throw NSError(domain: "JSON", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid input JSON"])
        }
        
        // Convert JSON inputs → MLMultiArray
        let inputFeatures = try MLDictionaryFeatureProvider(dictionary: inputsDict)
        
        // Run inference with timeout if specified
        let outputs: MLFeatureProvider
        if timeoutMs > 0 {
            let deadline = Date(timeIntervalSinceNow: Double(timeoutMs) / 1000.0)
            // Note: MLModel.prediction doesn't have built-in timeout.
            // For production, wrap in a cancellable Task or use a separate thread.
            outputs = try model.prediction(from: inputFeatures)
        } else {
            outputs = try model.prediction(from: inputFeatures)
        }
        
        // Convert MLFeatureProvider → JSON
        let outputsDict = featureProviderToDict(outputs)
        let outputsJson = try JSONSerialization.data(withJSONObject: outputsDict)
        let outputsStr = String(data: outputsJson, encoding: .utf8) ?? "{}"
        
        outOutputsJsonC.pointee = strdup(outputsStr)
        outErr.pointee = nil
        return 0
    } catch {
        outErr.pointee = strdup("Prediction failed: \(error)")
        return 1
    }
}

/// Free a C string returned by Core ML bridge
@_cdecl("coreml_free_string")
public func coreml_free_string(ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

// MARK: - Helpers

private func computeUnitsFromInt32(_ val: Int32) -> MLComputeUnits {
    switch val {
    case 0: return .all
    case 1: return .cpuOnly
    case 2: return .cpuAndGPU
    case 3: return .cpuAndANE
    default: return .all
    }
}

private func descriptionToDict(_ desc: MLFeatureDescription) -> [String: Any] {
    var dict = [String: Any]()
    dict["type"] = desc.type.description
    
    if let multiArrayType = desc.multiArrayConstraint {
        dict["shape"] = multiArrayType.shape
        dict["dtype"] = multiArrayType.dataType.description
    }
    
    return dict
}

private func featureProviderToDict(_ provider: MLFeatureProvider) -> [String: Any] {
    var dict = [String: Any]()
    
    for featureName in provider.featureNames {
        if let feature = provider.featureValue(for: featureName) {
            if let multiArray = feature.multiArrayValue {
                dict[featureName] = [
                    "shape": multiArray.shape,
                    "data": multiArray.dataPointer.withMemoryRebound(to: UInt8.self, capacity: multiArray.count) {
                        Data(bytes: $0, count: multiArray.count * MemoryLayout<Float>.size).base64EncodedString()
                    }
                ]
            } else {
                dict[featureName] = feature.description
            }
        }
    }
    
    return dict
}
```

**Key invariants baked in:**
- ✅ All ObjC types (`MLModel`, etc.) never cross the FFI boundary
- ✅ JSON contracts are stable and version-agnostic
- ✅ Opaque `OpaquePointer` handles managed by Rust's Rust side
- ✅ Every function returns `Int32` (0 = success, 1 = error) plus optional error message

---

## 3. Rust Wrapper & Integration

**File: `apple-silicon/src/core_ml_bridge.rs` (new)**

```rust
//! Safe Rust wrapper around Swift Core ML C bridge
//!
//! This module provides:
//! - Autorelease pool management (required for ObjC calls)
//! - Timeout enforcement (Core ML has no built-in timeouts)
//! - Error translation (C → Rust)
//! - Memory safety guarantees

use std::ffi::{CStr, CString};
use std::ptr;
use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use tracing::{debug, warn};

// Link to Swift-built static library
#[cfg(target_os = "macos")]
#[link(name = "CoreMLBridge", kind = "static")]
extern "C" {
    fn coreml_compile_model(
        model_path: *const i8,
        compute_units: i32,
        out_compiled_path: *mut *const i8,
        out_err: *mut *const i8,
    ) -> i32;

    fn coreml_load_model(
        compiled_dir: *const i8,
        compute_units: i32,
        out_handle: *mut *const std::ffi::c_void,
        out_err: *mut *const i8,
    ) -> i32;

    fn coreml_free_model(handle: *const std::ffi::c_void);

    fn coreml_model_schema(
        handle: *const std::ffi::c_void,
        out_schema_json: *mut *const i8,
        out_err: *mut *const i8,
    ) -> i32;

    fn coreml_predict(
        handle: *const std::ffi::c_void,
        inputs_json: *const i8,
        timeout_ms: i32,
        out_outputs_json: *mut *const i8,
        out_err: *mut *const i8,
    ) -> i32;

    fn coreml_free_string(ptr: *mut i8);
}

/// Opaque Core ML model handle (managed by Swift, wrapped in Rust)
pub struct CoreMLModel {
    handle: *const std::ffi::c_void,
}

impl CoreMLModel {
    /// Compile a source model to .mlmodelc
    pub fn compile(
        model_path: &str,
        compute_units: CoreMLComputeUnits,
    ) -> Result<String> {
        with_autorelease_pool(|| unsafe {
            let path_c = CString::new(model_path)?;
            let mut out_path: *const i8 = ptr::null();
            let mut out_err: *const i8 = ptr::null();

            let rc = coreml_compile_model(
                path_c.as_ptr(),
                compute_units as i32,
                &mut out_path,
                &mut out_err,
            );

            if rc != 0 {
                let err_msg = if !out_err.is_null() {
                    CStr::from_ptr(out_err).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                if !out_err.is_null() {
                    coreml_free_string(out_err as *mut _);
                }
                return Err(anyhow!("Compilation failed: {}", err_msg));
            }

            let result = CStr::from_ptr(out_path)
                .to_string_lossy()
                .to_string();
            coreml_free_string(out_path as *mut _);
            Ok(result)
        })
    }

    /// Load a compiled .mlmodelc
    pub fn load(
        compiled_dir: &str,
        compute_units: CoreMLComputeUnits,
    ) -> Result<Self> {
        with_autorelease_pool(|| unsafe {
            let dir_c = CString::new(compiled_dir)?;
            let mut out_handle: *const std::ffi::c_void = ptr::null();
            let mut out_err: *const i8 = ptr::null();

            let rc = coreml_load_model(
                dir_c.as_ptr(),
                compute_units as i32,
                &mut out_handle,
                &mut out_err,
            );

            if rc != 0 {
                let err_msg = if !out_err.is_null() {
                    CStr::from_ptr(out_err).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                if !out_err.is_null() {
                    coreml_free_string(out_err as *mut _);
                }
                return Err(anyhow!("Load failed: {}", err_msg));
            }

            Ok(CoreMLModel {
                handle: out_handle,
            })
        })
    }

    /// Query model I/O schema
    pub fn schema(&self) -> Result<Value> {
        with_autorelease_pool(|| unsafe {
            let mut out_json: *const i8 = ptr::null();
            let mut out_err: *const i8 = ptr::null();

            let rc = coreml_model_schema(self.handle, &mut out_json, &mut out_err);

            if rc != 0 {
                let err_msg = if !out_err.is_null() {
                    CStr::from_ptr(out_err).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                if !out_err.is_null() {
                    coreml_free_string(out_err as *mut _);
                }
                return Err(anyhow!("Schema retrieval failed: {}", err_msg));
            }

            let json_str = CStr::from_ptr(out_json).to_string_lossy();
            let schema: Value = serde_json::from_str(&json_str)?;
            coreml_free_string(out_json as *mut _);
            Ok(schema)
        })
    }

    /// Run inference with timeout
    pub fn predict(
        &self,
        inputs: &Value,
        timeout: Duration,
    ) -> Result<Value> {
        with_autorelease_pool(|| unsafe {
            let inputs_json = serde_json::to_string(inputs)?;
            let inputs_c = CString::new(inputs_json)?;
            let timeout_ms = std::cmp::min(timeout.as_millis() as i32, i32::MAX);

            let mut out_json: *const i8 = ptr::null();
            let mut out_err: *const i8 = ptr::null();

            let rc = coreml_predict(
                self.handle,
                inputs_c.as_ptr(),
                timeout_ms,
                &mut out_json,
                &mut out_err,
            );

            if rc != 0 {
                let err_msg = if !out_err.is_null() {
                    CStr::from_ptr(out_err).to_string_lossy().to_string()
                } else {
                    "Inference failed".to_string()
                };
                if !out_err.is_null() {
                    coreml_free_string(out_err as *mut _);
                }
                return Err(anyhow!("Prediction failed: {}", err_msg));
            }

            let output_str = CStr::from_ptr(out_json).to_string_lossy();
            let outputs: Value = serde_json::from_str(&output_str)?;
            coreml_free_string(out_json as *mut _);
            Ok(outputs)
        })
    }
}

impl Drop for CoreMLModel {
    fn drop(&mut self) {
        with_autorelease_pool(|| unsafe {
            coreml_free_model(self.handle);
        });
    }
}

// Safety: ObjC handles are thread-safe if used with proper pool management
unsafe impl Send for CoreMLModel {}
unsafe impl Sync for CoreMLModel {}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum CoreMLComputeUnits {
    All = 0,
    CpuOnly = 1,
    CpuAndGPU = 2,
    CpuAndANE = 3,
}

/// CRITICAL: Wrap every FFI call in an autorelease pool
/// macOS/iOS requires this for ObjC interop
#[cfg(target_os = "macos")]
fn with_autorelease_pool<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use objc2_foundation::NSAutoreleasePool;
    unsafe {
        let _pool = NSAutoreleasePool::new();
        f()
    }
}

#[cfg(not(target_os = "macos"))]
fn with_autorelease_pool<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f() // No-op on non-macOS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires .mlmodelc artifact
    fn test_load_and_infer() {
        let model = CoreMLModel::load("/path/to/model.mlmodelc", CoreMLComputeUnits::CpuAndANE)
            .expect("load");
        let schema = model.schema().expect("schema");
        println!("Schema: {}", schema);

        let inputs = json!({ "input": { "shape": [1], "data": "..." } });
        let outputs = model.predict(&inputs, Duration::from_secs(5)).expect("infer");
        println!("Outputs: {}", outputs);
    }
}
```

---

## 4. Build Integration (`build.rs`)

**File: `apple-silicon/build.rs` (update existing)**

```rust
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Only build Core ML bridge on macOS
    if cfg!(target_os = "macos") && cfg!(feature = "coreml") {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");

        // Build Swift package
        let bridge_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("coreml-bridge");

        let output = Command::new("swift")
            .args(&["build", "-c", "release"])
            .current_dir(&bridge_dir)
            .output()
            .expect("Failed to build Swift bridge");

        if !output.status.success() {
            panic!(
                "Swift build failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Link the static library
        let build_dir = bridge_dir.join(".build/release");
        println!(
            "cargo:rustc-link-search=native={}",
            build_dir.display()
        );
        println!("cargo:rustc-link-lib=static=CoreMLBridge");

        // System frameworks
        println!("cargo:rustc-link-lib=framework=CoreML");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
}
```

**File: `Cargo.toml` (apple-silicon crate)**

```toml
[dependencies]
# ...existing...
serde_json = "1.0"
objc2-foundation = { version = "0.3", features = ["NSAutoreleasePool"] }

[features]
default = []
coreml = []  # Feature-gate the bridge

[build-dependencies]
# swift build is in PATH (comes with Xcode)
```

---

## 5. Observability & Telemetry Gates

**File: `apple-silicon/src/telemetry.rs` (new)**

```rust
//! Telemetry for Core ML compilation, inference, and ANE coverage
//!
//! These metrics drive circuit breakers and inform decisions about
//! keeping Core ML enabled or falling back to CPU.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CoreMLMetrics {
    // Compilation
    pub compile_count: Arc<AtomicU64>,
    pub compile_success_count: Arc<AtomicU64>,
    pub compile_p99_ms: Arc<AtomicU64>,

    // Inference
    pub infer_count: Arc<AtomicU64>,
    pub infer_success_count: Arc<AtomicU64>,
    pub infer_p99_ms: Arc<AtomicU64>,

    // ANE coverage
    pub ane_eligible_count: Arc<AtomicU64>,
    pub ane_actually_used_count: Arc<AtomicU64>,

    // Errors
    pub timeout_count: Arc<AtomicU64>,
    pub pool_error_count: Arc<AtomicU64>,
}

impl CoreMLMetrics {
    pub fn new() -> Self {
        Self {
            compile_count: Arc::new(AtomicU64::new(0)),
            compile_success_count: Arc::new(AtomicU64::new(0)),
            compile_p99_ms: Arc::new(AtomicU64::new(0)),
            infer_count: Arc::new(AtomicU64::new(0)),
            infer_success_count: Arc::new(AtomicU64::new(0)),
            infer_p99_ms: Arc::new(AtomicU64::new(0)),
            ane_eligible_count: Arc::new(AtomicU64::new(0)),
            ane_actually_used_count: Arc::new(AtomicU64::new(0)),
            timeout_count: Arc::new(AtomicU64::new(0)),
            pool_error_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a compilation attempt
    pub fn record_compile(&self, duration: Duration, success: bool) {
        self.compile_count.fetch_add(1, Ordering::Relaxed);
        if success {
            self.compile_success_count.fetch_add(1, Ordering::Relaxed);
        }
        let ms = duration.as_millis() as u64;
        self.compile_p99_ms.store(ms, Ordering::Relaxed); // Simplified; use histogram in production
    }

    /// Record an inference attempt
    pub fn record_inference(&self, duration: Duration, success: bool, ane_used: bool) {
        self.infer_count.fetch_add(1, Ordering::Relaxed);
        if success {
            self.infer_success_count.fetch_add(1, Ordering::Relaxed);
        }
        if ane_used {
            self.ane_actually_used_count.fetch_add(1, Ordering::Relaxed);
        }
        let ms = duration.as_millis() as u64;
        self.infer_p99_ms.store(ms, Ordering::Relaxed);
    }

    /// Check if Core ML should be disabled (circuit breaker)
    pub fn should_circuit_break(&self) -> bool {
        let count = self.infer_count.load(Ordering::Relaxed);
        if count < 10 {
            return false; // Warm up period
        }

        let success = self.infer_success_count.load(Ordering::Relaxed) as f64;
        let success_rate = success / count as f64;

        // Disable if <95% success rate or p99 > 5s
        let p99 = self.infer_p99_ms.load(Ordering::Relaxed);
        success_rate < 0.95 || p99 > 5000
    }

    /// ANE coverage (0.0 - 1.0)
    pub fn ane_coverage(&self) -> f64 {
        let eligible = self.ane_eligible_count.load(Ordering::Relaxed);
        if eligible == 0 {
            return 0.0;
        }
        let used = self.ane_actually_used_count.load(Ordering::Relaxed);
        used as f64 / eligible as f64
    }
}
```

**Integration with inference loop:**

```rust
pub async fn infer_with_telemetry(
    model: &CoreMLModel,
    inputs: &Value,
    metrics: &CoreMLMetrics,
    timeout: Duration,
) -> Result<Value> {
    metrics.ane_eligible_count.fetch_add(1, Ordering::Relaxed);

    let start = Instant::now();
    let result = model.predict(inputs, timeout);
    let duration = start.elapsed();

    let success = result.is_ok();
    metrics.record_inference(duration, success, true); // Mark ANE as used (query model later)

    if metrics.should_circuit_break() {
        warn!("Core ML circuit breaker triggered; falling back to CPU");
        // Switch to Candle backend
    }

    result
}
```

---

## 6. Decision Gates & Testing Matrix

### Gate A: Compile + Load (Week 1)

```
✓ Compile .mlmodel → .mlmodelc
✓ Load .mlmodelc into handle
✓ Zero autorelease pool leaks (1000 compile+load cycles)
✓ Cache hit ratio ≥ 90%
```

### Gate B: Inference Correctness (Week 2)

```
✓ Single-input/single-output inference runs
✓ CPU vs Candle parity (L∞ < 1e-5, RMSE < 1e-6)
✓ Timeout enforcement works (kill stuck inference)
✓ p99 latency < 5s on typical models
```

### Gate C: ANE Eligibility (Week 3)

```
✓ Capability probe reports ANE coverage
✓ For eligible ops: ANE actually runs (profile with Instruments)
✓ ANE speedup ≥ 30% vs CPU (or accept as-is)
✓ Quantization doesn't break parity
```

### Gate D: Production Readiness (Week 4)

```
✓ 1-hour soak: no memory growth, no pool leaks
✓ Circuit breaker engages at SLA boundary
✓ Fallback to CPU automatic and transparent
✓ Telemetry dashboard shows all metrics
```

### Testing Matrix

```
Platforms: macOS 13+, 14.x, 15.x
Silicon:   M1, M2, M3
Models:    ResNet50 (classic CNN), Llama2-7B (large), Vision (image)
Precisions: fp32 (baseline), fp16, int8
Batch:     1, 4, 8, 16, 32
```

---

## 7. Concurrency & Autorelease Pool Discipline

**Critical invariants:**

1. **Every FFI call must run inside `with_autorelease_pool { }`**
   - This is non-negotiable. Missing one pool will cause memory leaks or crashes.

2. **Model sharing across threads:**
   ```rust
   // SAFE: MLModel thread-safe for read-only use
   let model = Arc::new(CoreMLModel::load(...)?);
   for i in 0..N {
       let m = Arc::clone(&model);
       tokio::spawn(async move {
           let _ = m.predict(&inputs, timeout);
       });
   }
   ```

3. **Timeout enforcement (Core ML has no built-in timeout):**
   ```rust
   pub fn predict_with_timeout(
       model: &CoreMLModel,
       inputs: &Value,
       timeout: Duration,
   ) -> Result<Value> {
       let (tx, rx) = tokio::sync::oneshot::channel();
       
       std::thread::spawn(move || {
           let result = with_autorelease_pool(|| {
               model.predict(inputs, timeout)
           });
           let _ = tx.send(result);
       });
       
       tokio::time::timeout(timeout, rx)
           .await
           .context("Inference timed out")?
           .context("Thread error")?
   }
   ```

---

## 8. Rollout Strategy (Feature Flag)

**`Cargo.toml`:**
```toml
[features]
default = []
coreml = []
```

**`apple-silicon/src/lib.rs`:**
```rust
#[cfg(feature = "coreml")]
pub mod core_ml_bridge;

#[cfg(feature = "coreml")]
pub fn inference_engine() -> Box<dyn InferenceEngine> {
    Box::new(CoreMLBackend::new())
}

#[cfg(not(feature = "coreml"))]
pub fn inference_engine() -> Box<dyn InferenceEngine> {
    Box::new(CandleCpuBackend::new())
}
```

**First release:** `cargo build --no-default-features` (CPU only)  
**Later release:** `cargo build --features coreml` (if gates pass)  
**Production fallback:** Runtime circuit breaker auto-disables on failures

---

## 9. Implementation Checklist (Ordered)

### Phase 1: Rust-side contracts (Week 0)
- [ ] `InferenceEngine` trait + `PreparedModel`
- [ ] `ModelArtifact` enum and cache key logic
- [ ] `CapabilityReport` struct
- [ ] Tests mock these (no Core ML calls yet)

### Phase 2: Swift bridge (Week 1)
- [ ] `Package.swift` + `CoreMLBridge.swift`
- [ ] `build.rs` integration
- [ ] Link test (verify `.a` links to Rust binary)
- [ ] Smoke test: compile + load one model

### Phase 3: Rust wrapper (Week 1-2)
- [ ] `core_ml_bridge.rs` safe wrappers
- [ ] Autorelease pool guards
- [ ] Error translation (C → Rust)
- [ ] Unit tests (no ObjC calls)

### Phase 4: Telemetry (Week 2)
- [ ] `telemetry.rs` metrics
- [ ] Circuit breaker logic
- [ ] Integration with inference loop
- [ ] Dashboard/Prometheus export

### Phase 5: Integration tests (Week 3)
- [ ] Load a real `.mlmodelc`
- [ ] Run inference, check parity vs Candle
- [ ] Soak test (1 hour, watch for leaks)
- [ ] Gate A, B, C checks

### Phase 6: Rollout (Week 4+)
- [ ] Feature flag defaults to off
- [ ] Document known constraints (see §10 blind spots)
- [ ] Canary rollout with circuit breaker armed
- [ ] Monitor telemetry; adjust gates if needed

---

## 10. Blind Spots & Gotchas to Avoid

1. **CoreFoundation vs ObjC boundary**: Prefer `CFURLRef`/`CFStringRef` in C headers; Swift bridges them to `URL/String`. Saves you from ObjC lifetime logic.

2. **Per-thread autorelease pools**: If any Rust thread touches ObjC directly, it crashes without a pool. Document this in comments.

3. **ABI stability of Swift**: You're not linking Swift ABI; you're calling C symbols (`@_cdecl`). That's stable. But ensure Swift runtime ships with the binary (static stdlib for `.a` linking).

4. **Model bundle layout**: `.mlmodelc` is a **directory**, not a file. Atomic moves (rename via temp) required for safe concurrent access.

5. **Entitlements/sandbox**: If moving to app/agent, ensure Core ML usage complies with restrictions. Usually fine for CLI.

6. **ANE availability probe**: Add a post-load hook that actually measures whether ANE was used (via Instruments or a hidden flag from Swift side).

7. **Numerical parity budget**: Define per-tensor thresholds (e.g., `L∞ < 0.01`, `RMSE < 0.001`) now. Prevents "fast but wrong" regressions.

8. **Timeout & cancellation**: Core ML doesn't have built-in timeout. Wrap in a separate thread and interrupt if needed. Never block Rust async on ObjC without a timeout.

9. **Pre-compiled model path**: Add a fast path that only loads `.mlmodelc` (no runtime compile). Lets you experiment with offline conversion pipelines.

10. **Operator/shape constraints**: Not all ops are ANE-friendly. At model load time, query which ops will run where. If ANE coverage < 70%, don't count on speedup.

---

## 11. Next Steps

1. **Implement Phase 1 first** (Rust-side contracts): Zero dependencies on ObjC. Test the abstraction seams with mocks.

2. **Get Phase 2 building**: Swift → C ABI bridge linking cleanly. No inference yet, just compile + load.

3. **Run Gate A** (leak checking): 1000 compile+load cycles, zero growth in Instruments Leaks.

4. **Then add inference** (Phase 3) and run Gate B (parity vs CPU).

5. **Use telemetry** to decide if ANE is worth it (Gate C).

6. **Ship behind feature flag** with circuit breaker armed.

This path defers risk to proven decision points and avoids committing to raw objc2 or invasive unsafe code in Rust.

---

## References

- [Core ML Documentation](https://developer.apple.com/documentation/coreml)
- [Swift C Interop](https://developer.apple.com/documentation/swiftc/swift-c-interop-example)
- [Autorelease Pools](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/mmAutoreleasePools.html)
- [Performance Profiling (Instruments)](https://help.apple.com/instruments/mac/current/)
