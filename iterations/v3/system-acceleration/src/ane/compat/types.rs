//! Core ML type definitions
//!
//! This module contains the basic type definitions for Core ML operations,
//! including models, configurations, arrays, and feature providers.

use schemars::JsonSchema;
use std::collections::HashMap;
use std::ptr::NonNull;

// Import functions needed for method implementations
use super::model::{coreml_runtime_available, coreml_unavailable_error};
// Note: KvStateHandle is defined in this file (types.rs), not in kv_cache.rs

// Import FFI functions needed for implementations
use super::model::{
    agentbridge_array_create_float32, agentbridge_dict_provider_create,
    agentbridge_dict_provider_destroy, agentbridge_dict_provider_set_feature_multiarray,
    agentbridge_free_string,
};

// FFI declaration for setting state features
#[cfg(target_os = "macos")]
extern "C" {
    fn agentbridge_dict_provider_set_feature_state(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        kv_state_ref: u64,
        model_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;
}

/// Opaque handle to a Core ML model managed by the BridgesFFI framework
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub struct MLModel(u64);

impl MLModel {
    /// Create a new MLModel from a raw handle value
    pub fn new(handle: u64) -> Self {
        Self(handle)
    }

    /// Get the raw model handle value
    pub fn handle(&self) -> u64 {
        self.0
    }
}

/// Core ML model configuration
#[derive(Debug, Clone, JsonSchema, serde::Serialize)]
pub struct MLModelConfiguration {
    /// Whether to allow low precision accumulation on GPU
    pub allow_low_precision_accumulation_on_gpu: bool,
    /// Compute units to use
    pub compute_units: MLComputeUnits,
}

/// Compute units for Core ML inference
#[derive(Debug, Clone, Copy, PartialEq, JsonSchema, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MLComputeUnits {
    /// Use CPU only
    #[serde(rename = "cpuOnly")]
    CpuOnly,
    /// Use CPU and GPU
    #[serde(rename = "cpuAndGPU")]
    CpuAndGpu,
    /// Use CPU and Neural Engine (ANE) - explicit ANE acceleration
    #[serde(rename = "cpuAndNeuralEngine")]
    CpuAndNeuralEngine,
    /// Use all available compute units (including ANE if available)
    #[serde(rename = "all")]
    All,
}

/// Multi-dimensional array for Core ML I/O
#[derive(Debug)]
pub struct MLMultiArray {
    /// Raw pointer to the MLMultiArray
    ptr: NonNull<std::ffi::c_void>,
    /// Shape information
    shape: Vec<i32>,
    /// Data type
    data_type: MLMultiArrayDataType,
}

impl MLMultiArray {
    /// Get the raw pointer to the MLMultiArray
    pub fn ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }

    /// Get the shape of the multi-array
    pub fn shape(&self) -> &[i32] {
        &self.shape
    }

    /// Get the data type of the multi-array
    pub fn data_type(&self) -> MLMultiArrayDataType {
        self.data_type
    }

    /// Create an MLMultiArray from a slice of float data
    pub fn from_slice(data: &[f32], shape: &[i32]) -> std::result::Result<Self, String> {
        if !coreml_runtime_available() {
            let err = coreml_unavailable_error();
            return Err(format!("{}", err));
        }

        // Validate shape
        if shape.is_empty() {
            return Err("Shape cannot be empty".to_string());
        }

        let total_elements: usize = shape.iter().map(|&x| x as usize).product();
        if total_elements != data.len() {
            return Err(format!(
                "Data length {} doesn't match shape product {}",
                data.len(),
                total_elements
            ));
        }

        // Create the MLMultiArray through FFI
        let mut array_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_array_create_float32(
                data.as_ptr(),
                data.len() as i32,
                shape.as_ptr(),
                shape.len() as i32,
                &mut array_ref,
                &mut error_ptr,
            )
        };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error creating MLMultiArray".to_string()
            };
            return Err(error_msg);
        }

        if array_ref == 0 {
            return Err("Failed to create MLMultiArray handle".to_string());
        }

        // Get the raw pointer from the registry
        let ptr = NonNull::new(array_ref as *mut std::ffi::c_void)
            .ok_or_else(|| "Failed to create array pointer".to_string())?;

        Ok(MLMultiArray {
            ptr,
            shape: shape.to_vec(),
            data_type: MLMultiArrayDataType::Float32,
        })
    }
}

// Drop implementation moved to model.rs to access runtime functions

/// Feature value that can be passed to Core ML models
#[derive(Debug)]
pub enum MLFeatureValue {
    /// Multi-dimensional array data
    MultiArray(MLMultiArray),
    /// String data
    String(String),
    /// Numeric data
    Double(f64),
    /// Dictionary of feature values
    Dictionary(HashMap<String, MLFeatureValue>),
    /// Image data (raw bytes for CoreML image processing)
    Image(Vec<u8>),
    /// State feature for stateful models (KV cache)
    State(KvStateHandle),
}

/// Provider of feature values for Core ML model input
#[derive(Debug)]
pub struct MLFeatureProvider {
    /// Raw pointer to the feature provider
    ptr: NonNull<std::ffi::c_void>,
}

impl MLFeatureProvider {
    /// Create a new MLFeatureProvider from a raw pointer
    pub fn new(ptr: NonNull<std::ffi::c_void>) -> Self {
        Self { ptr }
    }

    /// Get the raw pointer to the feature provider
    pub fn ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }

    /// Get a reference to the internal pointer
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }
}

// Drop implementation moved to model.rs to access runtime functions

/// Dictionary-based feature provider for Core ML
#[derive(Debug)]
pub struct MLDictionaryFeatureProvider {
    /// Raw pointer to the dictionary feature provider
    ptr: NonNull<std::ffi::c_void>,
}

impl MLDictionaryFeatureProvider {
    /// Get the raw pointer to the dictionary feature provider
    pub fn ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }

    /// Create a dictionary feature provider from a map of feature values
    ///
    /// For state features, a model reference is required. Pass `Some(model_ref)` if the dictionary
    /// contains State features, otherwise `None` is sufficient.
    pub fn from_dictionary(
        dict: &std::collections::HashMap<String, MLFeatureValue>,
        model_ref: Option<u64>,
    ) -> std::result::Result<Self, String> {
        if !coreml_runtime_available() {
            let err = coreml_unavailable_error();
            return Err(format!("{}", err));
        }

        // Create the dictionary provider through FFI
        let mut provider_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe { agentbridge_dict_provider_create(&mut provider_ref, &mut error_ptr) };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error creating dictionary feature provider".to_string()
            };
            return Err(error_msg);
        }

        if provider_ref == 0 {
            return Err("Failed to create dictionary provider handle".to_string());
        }

        // Add features to the provider
        for (name, value) in dict {
            match value {
                MLFeatureValue::MultiArray(array) => {
                    //       Currently only supports float32 arrays; should implement comprehensive support for multiple data types (int32, int64, float64, etc.) for complete Core ML feature value handling.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Multiple data types are supported (int32, int64, float64, etc.)
                    // - Data type conversion is accurate
                    // - Unsupported types are handled gracefully
                    // - Type validation is comprehensive
                    //
                    // DEPENDENCIES:
                    // - Data type conversion utilities (Required)
                    // - Core ML type mapping (Required)
                    // - Type validation system (Required)
                    //
                    // ESTIMATED EFFORT: 8-12 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (Core ML integration functionality)
                    // - Change Budget: ~250 LOC
                    // - Reviewer Requirements: Core ML and type system expertise
                    if array.data_type != MLMultiArrayDataType::Float32 {
                        return Err(format!("Unsupported data type for feature '{}'", name));
                    }

                    let name_cstr = std::ffi::CString::new(name.clone())
                        .map_err(|e| format!("Invalid feature name '{}': {}", name, e))?;

                    let mut feature_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

                    // Use the array handle directly instead of treating it as a data pointer
                    let set_result = unsafe {
                        agentbridge_dict_provider_set_feature_multiarray(
                            provider_ref,
                            name_cstr.as_ptr(),
                            array.ptr.as_ptr() as u64,
                            &mut feature_error_ptr,
                        )
                    };

                    if set_result != 0 {
                        let error_msg = if !feature_error_ptr.is_null() {
                            unsafe {
                                let cstr = std::ffi::CStr::from_ptr(feature_error_ptr);
                                let msg = cstr.to_string_lossy().to_string();
                                agentbridge_free_string(feature_error_ptr);
                                msg
                            }
                        } else {
                            format!("Unknown error setting feature '{}'", name)
                        };

                        // Clean up the provider we created
                        unsafe { agentbridge_dict_provider_destroy(provider_ref) };
                        return Err(error_msg);
                    }
                }
                MLFeatureValue::State(kv_state) => {
                    // State features require a model reference
                    let model_ref_val = model_ref.ok_or_else(|| {
                        format!("Model reference required for state feature '{}'", name)
                    })?;

                    let name_cstr = std::ffi::CString::new(name.clone())
                        .map_err(|e| format!("Invalid feature name '{}': {}", name, e))?;

                    let mut feature_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

                    let set_result = unsafe {
                        agentbridge_dict_provider_set_feature_state(
                            provider_ref,
                            name_cstr.as_ptr(),
                            kv_state.handle(),
                            model_ref_val,
                            &mut feature_error_ptr,
                        )
                    };

                    if set_result != 0 {
                        let error_msg = if !feature_error_ptr.is_null() {
                            unsafe {
                                let cstr = std::ffi::CStr::from_ptr(feature_error_ptr);
                                let msg = cstr.to_string_lossy().to_string();
                                agentbridge_free_string(feature_error_ptr);
                                msg
                            }
                        } else {
                            format!("Unknown error setting state feature '{}'", name)
                        };

                        // Clean up the provider we created
                        unsafe { agentbridge_dict_provider_destroy(provider_ref) };
                        return Err(error_msg);
                    }
                }
                _ => {
                    // Clean up the provider we created
                    unsafe { agentbridge_dict_provider_destroy(provider_ref) };
                    return Err(format!(
                        "Unsupported feature type for '{}': {:?}",
                        name, value
                    ));
                }
            }
        }

        let ptr = NonNull::new(provider_ref as *mut std::ffi::c_void)
            .ok_or_else(|| "Failed to create provider pointer".to_string())?;

        Ok(MLDictionaryFeatureProvider { ptr })
    }
}

// Drop implementation moved to model.rs to access runtime functions

/// Data types supported by Core ML multi-arrays
#[derive(Debug, Clone, Copy, PartialEq, JsonSchema)]
pub enum MLMultiArrayDataType {
    /// 32-bit floating point
    Float32,
    /// 16-bit floating point
    Float16,
}

/// Feature types supported by Core ML
#[derive(Debug, Clone, Copy, PartialEq, JsonSchema)]
pub enum MLFeatureType {
    /// Invalid feature type
    Invalid,
    /// Multi-dimensional array
    MultiArray,
    /// Image data
    Image,
    /// Dictionary of features
    Dictionary,
}

/// Opaque handle to KV cache state for stateful inference
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub struct KvStateHandle(u64);

impl KvStateHandle {
    /// Create a new KV state handle from a raw handle value
    pub fn new(handle: u64) -> Self {
        Self(handle)
    }

    /// Get the raw handle value
    pub fn handle(&self) -> u64 {
        self.0
    }
}

impl Default for MLModelConfiguration {
    fn default() -> Self {
        Self {
            allow_low_precision_accumulation_on_gpu: false,
            compute_units: MLComputeUnits::All,
        }
    }
}

impl MLModelConfiguration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_compute_units(&mut self, units: MLComputeUnits) {
        self.compute_units = units;
    }

    pub fn set_allow_low_precision_accumulation_on_gpu(&mut self, allow: bool) {
        self.allow_low_precision_accumulation_on_gpu = allow;
    }
}

impl MLComputeUnits {
    pub fn all() -> Self {
        MLComputeUnits::All
    }

    pub fn cpu_only() -> Self {
        MLComputeUnits::CpuOnly
    }

    pub fn cpu_and_gpu() -> Self {
        MLComputeUnits::CpuAndGpu
    }
}

impl MLMultiArrayDataType {
    pub const FLOAT32: Self = MLMultiArrayDataType::Float32;
    pub const FLOAT16: Self = MLMultiArrayDataType::Float16;
}

impl MLFeatureType {
    pub const MULTI_ARRAY: Self = MLFeatureType::MultiArray;
    pub const IMAGE: Self = MLFeatureType::Image;
    pub const DICTIONARY: Self = MLFeatureType::Dictionary;
    pub const INVALID: Self = MLFeatureType::Invalid;
}
