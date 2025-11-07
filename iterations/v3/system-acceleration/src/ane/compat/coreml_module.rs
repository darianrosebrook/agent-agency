//! Core ML framework interface
//!
//! This module provides the main Core ML framework integration including
//! ModelRef, registry management, inference operations, and FFI declarations.

use schemars::JsonSchema;
use crate::ane::ane_errors::{ANEError, Result};
use candle_core::Device;
use std::ptr::NonNull;
use std::ffi::CString;
use std::path::Path;
use std::collections::HashMap;

// Import types from the types module
use super::types::*;

// Import utility functions from the model module
use super::model::{coreml_runtime_available, coreml_unavailable_error};

// Import registry types and functions
use super::registry::{CoreMlHandle, ModelRef};
use super::registry::registry;

// Import safety utilities

// Import Tensor type
use super::safety::Tensor;

// Note: MLMultiArray and MLDictionaryFeatureProvider constructors are accessed via super::model

/// Check if ANE is available on this system
pub fn is_ane_available() -> bool {
    coreml_runtime_available()
}

/// Get Core ML driver version (if available)
pub fn driver_version() -> Option<String> {
    None
}

/// Compile a .mlmodel file to .mlmodelc format
pub fn compile_model(source_path: &Path) -> Result<std::path::PathBuf> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal("Core ML not available on this platform".to_string()));
    }

    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;

        // Validate input file exists
        if !source_path.exists() {
            return Err(ANEError::InvalidInput(format!("Source model file does not exist: {:?}", source_path)));
        }

        // Check if it's already a compiled model (.mlmodelc)
        if let Some(extension) = source_path.extension() {
            if extension == "mlmodelc" {
                return Ok(source_path.to_path_buf());
            }
        }

        // Generate output path (.mlmodel -> .mlmodelc)
        let mut output_path = source_path.to_path_buf();
        output_path.set_extension("mlmodelc");

        // Convert paths to C strings
        let source_path_cstr = source_path.to_str()
            .ok_or_else(|| ANEError::InvalidInput("Invalid source path encoding".to_string()))?;
        let source_path_cstr = CString::new(source_path_cstr)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid source path: {}", e)))?;

        let output_path_str = output_path.to_str()
            .ok_or_else(|| ANEError::InvalidInput("Invalid output path encoding".to_string()))?;
        let _output_path_cstr = CString::new(output_path_str)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid output path: {}", e)))?;

        // Create model configuration JSON
        let config = MLModelConfiguration {
            allow_low_precision_accumulation_on_gpu: true,
            compute_units: MLComputeUnits::All,
        };

        let config_json = serde_json::to_string(&config)
            .map_err(|e| ANEError::Internal(format!("Failed to serialize config: {}", e)))?;
        let config_json_cstr = CString::new(config_json)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid config JSON: {}", e)))?;

        // Create model using agentbridge
        let mut model_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let create_result = unsafe {
            agentbridge_model_create(
                source_path_cstr.as_ptr(),
                config_json_cstr.as_ptr(),
                &mut model_ref,
                &mut error_ptr,
            )
        };

        if create_result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    msg
                }
            } else {
                "Unknown error creating model".to_string()
            };
            return Err(ANEError::Internal(format!("Failed to create model: {}", error_msg)));
        }

        if model_ref == 0 {
            return Err(ANEError::Internal("Model creation returned null reference".to_string()));
        }

        // Verify the compiled model was created
        if !output_path.exists() {
            // Clean up the model reference
            unsafe {
                agentbridge_model_destroy(model_ref);
            }
            return Err(ANEError::Internal(format!("Compiled model file was not created: {:?}", output_path)));
        }

        // Verify the file has content
        let metadata = std::fs::metadata(&output_path)
            .map_err(|e| {
                unsafe { agentbridge_model_destroy(model_ref); }
                ANEError::Internal(format!("Failed to verify compiled model: {}", e))
            })?;

        if metadata.len() == 0 {
            unsafe { agentbridge_model_destroy(model_ref); }
            return Err(ANEError::Internal("Compiled model file is empty".to_string()));
        }

        // Clean up the model reference (we don't need it for the compiled file)
        unsafe {
            agentbridge_model_destroy(model_ref);
        }

        Ok(output_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML not available on this platform".to_string()))
    }
}

/// Load a compiled Core ML model and return an opaque reference
/// The raw handle is stored in a thread-local registry for safety
pub fn load_model(path: &str) -> Result<ModelRef> {
    if !coreml_runtime_available() {
        return Err(coreml_unavailable_error());
    }

    #[cfg(target_os = "macos")]
    {
        // Load Core ML model using agentbridge framework
        let model_path_cstr = std::ffi::CString::new(path)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid model path: {}", e)))?;

        let mut model_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_model_create(
                model_path_cstr.as_ptr(),
                std::ptr::null(), // No config for now
                &mut model_ref,
                &mut error_ptr,
            )
        };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    msg
                }
            } else {
                "Unknown Core ML error".to_string()
            };
            return Err(ANEError::Internal(format!("Failed to load Core ML model: {}", error_msg)));
        }

        // Register the handle in the thread-local registry
        let raw_ptr = model_ref as *mut std::ffi::c_void;
        let handle = CoreMlHandle::new(raw_ptr)
            .ok_or_else(|| ANEError::Internal("Null model handle".into()))?;
        let id = registry::register_model(handle);
        Ok(id)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML not available on this platform".to_string()))
    }
}

// I/O safety moved to safety.rs

/// Inference options
pub struct InferenceOptions {
    pub compute_units: ComputeUnits,
    pub allow_low_precision: bool,
}

/// Compute units
pub enum ComputeUnits {
    CpuOnly,
    CpuAndGpu,
    All,
}

/// Core ML model type with opaque reference
#[derive(Debug)]
pub struct CoreMLModel {
    pub model_ref: ModelRef,
    pub metadata: ModelMetadata,
}

/// Model metadata
#[derive(Debug)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Core ML capabilities
pub struct CoreMLCapabilities {
    pub ane_available: bool,
    pub supported_precisions: Vec<String>,
}

/// Detect Core ML capabilities
pub fn detect_coreml_capabilities() -> CoreMLCapabilities {
    CoreMLCapabilities {
        ane_available: coreml_runtime_available(),
        supported_precisions: if coreml_runtime_available() {
            vec!["FP16".to_string(), "FP32".to_string()]
        } else {
            vec![]
        },
    }
}

/// Create input features for Core ML inference
#[allow(dead_code)] // Will be used in v4
fn create_input_features(
    _input_name: &str,
    _input_data: &[f32],
    _input_shape: &[i32],
) -> Result<MLFeatureProvider> {
    #[cfg(target_os = "macos")]
    {
        // Create MLFeatureProvider using agentbridge framework
        let mut provider_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_dict_provider_create(
                &mut provider_ref,
                &mut error_ptr,
            )
        };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    msg
                }
            } else {
                "Unknown Core ML error".to_string()
            };
            return Err(ANEError::Internal(format!("Failed to create MLFeatureProvider: {}", error_msg)));
        }

        let ptr = NonNull::new(provider_ref as *mut std::ffi::c_void)
            .ok_or_else(|| ANEError::Internal("Failed to create MLFeatureProvider".to_string()))?;
        Ok(MLFeatureProvider::new(ptr))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML not available on this platform".to_string()))
    }
}

/// Extract output tensor from prediction results
#[allow(dead_code)] // Will be used in v4
fn extract_output_tensor(_prediction: &MLFeatureProvider) -> Result<Tensor> {
    #[cfg(target_os = "macos")]
    {
        // Extract output tensor from MLFeatureProvider using agentbridge framework
        let output_json_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_dict_provider_destroy(
                _prediction.ptr() as u64,
            )
        };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    msg
                }
            } else {
                "Unknown Core ML error".to_string()
            };
            return Err(ANEError::Internal(format!("Failed to extract output tensor: {}", error_msg)));
        }

        // Parse the output JSON
        let output_json_str = if !output_json_ptr.is_null() {
            unsafe {
                std::ffi::CStr::from_ptr(output_json_ptr)
                    .to_string_lossy()
                    .to_string()
            }
        } else {
            return Err(ANEError::Internal("No output data received".to_string()));
        };

        let output_data: serde_json::Value = serde_json::from_str(&output_json_str)
            .map_err(|e| ANEError::Internal(format!("Failed to parse output JSON: {}", e)))?;

        // Extract tensor data and shape
        let data = output_data["data"].as_array()
            .ok_or_else(|| ANEError::Internal("Invalid output data format".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        let _shape = output_data["shape"].as_array()
            .ok_or_else(|| ANEError::Internal("Invalid output shape format".to_string()))?
            .iter()
            .map(|v| v.as_i64().unwrap_or(1) as usize)
            .collect::<Vec<usize>>();

        // Create tensor with proper shape
        let tensor = Tensor::new(&*data, &Device::Cpu)?;
        Ok(tensor)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML not available on this platform".to_string()))
    }
}

/// Run inference on a loaded model using opaque reference
pub fn run_inference(
    model_ref: ModelRef,
    input_name: &str,
    input_data: &[f32],
    input_shape: &[usize],
) -> Result<Tensor> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal("Core ML not available on this platform".to_string()));
    }

    #[cfg(target_os = "macos")]
    {
        // Get the raw model handle from the registry
        let model_handle = registry::get_model_handle(model_ref)
            .ok_or_else(|| ANEError::InvalidInput("Model not found in registry".to_string()))?;

        // Convert input shape to i32 array for Core ML
        let shape_i32: Vec<i32> = input_shape.iter().map(|&x| x as i32).collect();

        // Create input tensor
        let input_array = MLMultiArray::from_slice(input_data, &shape_i32)
            .map_err(|e| ANEError::Internal(format!("Failed to create input tensor: {}", e)))?;

        // Create input feature dictionary
        let mut input_features = HashMap::new();
        input_features.insert(input_name.to_string(), MLFeatureValue::MultiArray(input_array));

        // Create input provider
        let input_provider = MLDictionaryFeatureProvider::from_dictionary(&input_features)
            .map_err(|e| ANEError::Internal(format!("Failed to create input provider: {}", e)))?;

        // Run inference
        let mut output_provider_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let inference_result = unsafe {
            agentbridge_model_run_inference(
                model_handle.as_ptr() as u64,
                input_provider.ptr() as u64,
                &mut output_provider_ref,
                &mut error_ptr,
            )
        };

        if inference_result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error during Core ML inference".to_string()
            };
            return Err(ANEError::InferenceFailed(error_msg));
        }

        if output_provider_ref == 0 {
            return Err(ANEError::Internal("No output provider returned from inference".to_string()));
        }

        // Extract output tensor - for now, assume the output feature name is the same as input
        // In practice, this would need to be configurable
        let output_name_cstr = CString::new(input_name)
            .map_err(|e| ANEError::Internal(format!("Invalid output name: {}", e)))?;

        let mut output_data_ptr: *mut f32 = std::ptr::null_mut();
        let mut output_shape_ptr: *mut i32 = std::ptr::null_mut();
        let mut output_shape_length: i32 = 0;
        let mut output_data_length: i32 = 0;
        let mut extract_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let extract_result = unsafe {
            agentbridge_provider_get_feature_float32(
                output_provider_ref,
                output_name_cstr.as_ptr(),
                &mut output_data_ptr,
                &mut output_shape_ptr,
                &mut output_shape_length,
                &mut output_data_length,
                &mut extract_error_ptr,
            )
        };

        if extract_result != 0 {
            // Clean up the output provider
            unsafe { agentbridge_provider_destroy(output_provider_ref) };

            let error_msg = if !extract_error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(extract_error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(extract_error_ptr);
                    msg
                }
            } else {
                format!("Unknown error extracting output feature '{}'", input_name)
            };
            return Err(ANEError::Internal(error_msg));
        }

        if output_data_ptr.is_null() || output_data_length <= 0 {
            // Clean up the output provider
            unsafe { agentbridge_provider_destroy(output_provider_ref) };
            return Err(ANEError::Internal("No output data returned from inference".to_string()));
        }

        // Create candle tensor from the output data
        let output_data = unsafe {
            std::slice::from_raw_parts(output_data_ptr, output_data_length as usize)
        };

        let output_tensor = Tensor::new(output_data, &Device::Cpu)
            .map_err(|e| {
                // Clean up the output provider and data
                unsafe {
                    agentbridge_provider_destroy(output_provider_ref);
                    agentbridge_free_array_data(output_data_ptr);
                    agentbridge_free_array_data(output_shape_ptr as *mut f32);
                }
                ANEError::Internal(format!("Failed to create output tensor: {}", e))
            })?;

                // Clean up resources
                unsafe {
                    agentbridge_provider_destroy(output_provider_ref);
                    agentbridge_free_array_data(output_data_ptr);
                    agentbridge_free_array_data(output_shape_ptr as *mut f32);
                }

        Ok(output_tensor)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal("Core ML not available on this platform".to_string()))
    }
}

/// Unload a model and free resources using opaque reference
pub fn unload_model(model_ref: ModelRef) {
    // Unregister from thread-local registry - this will drop the CoreMlHandle
    // and trigger proper cleanup
    let _handle = registry::unregister_model(model_ref);
    // Handle is dropped here, which calls the Drop impl for CoreMlHandle
}

/// Model input/output specification
#[derive(Debug, Clone, JsonSchema)]
pub struct ModelIOSpec {
    pub name: String,
    pub dtype: String,
    pub shape: Vec<i32>,
    pub batch_capable: bool,
}

/// Query model inputs
pub fn query_model_inputs(_model_ref: ModelRef) -> Result<Vec<ModelIOSpec>> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal("Core ML not available on this platform".to_string()));
    }

    // For now, return a default input spec
    // In a real implementation, this would query the actual model
    Ok(vec![ModelIOSpec {
        name: "input".to_string(),
        dtype: "F32".to_string(),
        shape: vec![-1, -1], // [batch_size, sequence_length]
        batch_capable: true,
    }])
}

/// Query model outputs
pub fn query_model_outputs(_model_ref: ModelRef) -> Result<Vec<ModelIOSpec>> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal("Core ML not available on this platform".to_string()));
    }

    // For now, return a default output spec
    // In a real implementation, this would query the actual model
    Ok(vec![ModelIOSpec {
        name: "output".to_string(),
        dtype: "F32".to_string(),
        shape: vec![-1, -1, -1], // [batch_size, sequence_length, vocab_size]
        batch_capable: true,
    }])
}

// ============================================================================
// FFI Declarations for BridgesFFI
// ============================================================================

// Link to agentbridge functions (provided by Swift bridge static library)
// The actual linking is handled by build.rs which builds and links the Swift bridge
// Note: Some functions may not be implemented yet - they will fail at runtime if called
#[cfg(target_os = "macos")]
extern "C" {
    // Re-export FFI functions for use in this module
    pub fn agentbridge_init() -> i32;
    pub fn agentbridge_shutdown() -> i32;
    pub fn agentbridge_get_version(out_version: *mut *mut std::ffi::c_char) -> i32;

    pub fn agentbridge_model_download(
        identifier: *const std::ffi::c_char,
        channel: *const std::ffi::c_char,
        out_model_path: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_is_cached(
        identifier: *const std::ffi::c_char,
        channel: *const std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_remove_cached(
        identifier: *const std::ffi::c_char,
        channel: *const std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_get_cache_stats(
        out_stats: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_clear_cache(
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_create(
        model_path: *const std::ffi::c_char,
        config_json: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_destroy(model_ref: u64) -> i32;

    pub fn agentbridge_model_get_info(
        model_ref: u64,
        out_info: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_mistral_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_mistral_generate(
        model_ref: u64,
        prompt: *const std::ffi::c_char,
        max_tokens: i32,
        temperature: f32,
        out_text: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_mistral_encode(
        text: *const std::ffi::c_char,
        out_tokens: *mut *mut i32,
        out_token_count: *mut i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_mistral_decode(
        tokens: *const i32,
        token_count: i32,
        out_text: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_mistral_free_tokens(tokens: *mut i32, count: i32);

    pub fn agentbridge_free_string(ptr: *mut std::ffi::c_char);

    pub fn agentbridge_array_create_float32(
        data: *const f32,
        data_length: i32,
        shape: *const i32,
        shape_length: i32,
        out_array_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_array_destroy(array_ref: u64) -> i32;

    pub fn agentbridge_run_inference(
        model_ref: u64,
        input_name: *const std::ffi::c_char,
        input_data: *const f32,
        input_shape: *const i32,
        input_shape_len: i32,
        out_output_data: *mut *mut f32,
        out_output_shape: *mut *mut i32,
        out_output_shape_len: *mut i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_dict_provider_create(
        out_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_dict_provider_set_feature_float32(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        data: *const f32,
        shape: *const i32,
        shape_length: i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_dict_provider_set_feature_multiarray(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        array_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_dict_provider_destroy(provider_ref: u64) -> i32;

    pub fn agentbridge_model_run_inference(
        model_ref: u64,
        input_provider_ref: u64,
        out_output_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    // KV cache state management functions
    pub fn agentbridge_kv_state_create(
        model_ref: u64,
        n_layers: i32,
        n_kv_heads: i32,
        head_dim: i32,
        max_seq_len: i32,
        out_state_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_kv_state_destroy(state_ref: u64) -> i32;

    pub fn agentbridge_model_run_inference_with_kv(
        model_ref: u64,
        input_provider_ref: u64,
        kv_state_ref: u64,
        out_output_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_kv_state_step(
        kv_state_ref: u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_kv_state_reset(
        kv_state_ref: u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_provider_destroy(provider_ref: u64) -> i32;

    pub fn agentbridge_provider_get_feature_float32(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        out_data: *mut *mut f32,
        out_shape: *mut *mut i32,
        out_shape_length: *mut i32,
        out_data_length: *mut i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_free_array_data(data: *mut f32) -> i32;

    pub fn agentbridge_audio_whisper_create(
        model_path: *const std::ffi::c_char,
        model_size: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_audio_whisper_transcribe(
        model_ref: u64,
        audio_path: *const std::ffi::c_char,
        language: *const std::ffi::c_char,
        out_text: *mut *mut std::ffi::c_char,
        out_segments_json: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_audio_speech_create(
        language: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_audio_speech_transcribe(
        model_ref: u64,
        audio_path: *const std::ffi::c_char,
        out_text: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_vision_yolo_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_vision_yolo_detect(
        model_ref: u64,
        image_data: *const u8,
        data_length: i32,
        confidence_threshold: f32,
        out_detections_json: *mut *mut std::ffi::c_char,
        out_detection_count: *mut i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_vision_ocr_create(
        language: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_vision_ocr_extract(
        model_ref: u64,
        image_data: *const u8,
        data_length: i32,
        out_text: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_diffusion_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_diffusion_generate(
        model_ref: u64,
        prompt: *const std::ffi::c_char,
        width: i32,
        height: i32,
        steps: i32,
        guidance_scale: f32,
        seed: u64,
        out_image_data: *mut *mut u8,
        out_data_length: *mut i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_text_diffusion_free_image(image_data: *mut u8);

    pub fn agentbridge_system_get_metrics(
        out_metrics: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_system_profile_start(
        session_name: *const std::ffi::c_char,
        out_session_id: *mut u64,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_system_profile_stop(
        session_id: u64,
        out_report: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;
}
