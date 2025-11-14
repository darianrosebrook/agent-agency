//! Core ML framework interface
//!
//! This module provides the main Core ML framework integration including
//! ModelRef, registry management, inference operations, and FFI declarations.

use crate::ane::ane_errors::{ANEError, Result};
use candle_core::Device;
use schemars::JsonSchema;
use serde_json;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;
use std::ptr::NonNull;

// Import types from the types module
use super::types::*;

// Import utility functions from the model module
use super::model::{coreml_runtime_available, coreml_unavailable_error};

// Import registry types and functions
use super::registry::registry;
use super::registry::{CoreMlHandle, ModelRef};

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
        return Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;

        // Validate input file exists
        if !source_path.exists() {
            return Err(ANEError::InvalidInput(format!(
                "Source model file does not exist: {:?}",
                source_path
            )));
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
        let source_path_cstr = source_path
            .to_str()
            .ok_or_else(|| ANEError::InvalidInput("Invalid source path encoding".to_string()))?;
        let source_path_cstr = CString::new(source_path_cstr)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid source path: {}", e)))?;

        let output_path_str = output_path
            .to_str()
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
            return Err(ANEError::Internal(format!(
                "Failed to create model: {}",
                error_msg
            )));
        }

        if model_ref == 0 {
            return Err(ANEError::Internal(
                "Model creation returned null reference".to_string(),
            ));
        }

        // Verify the compiled model was created
        if !output_path.exists() {
            // Clean up the model reference
            unsafe {
                agentbridge_model_destroy(model_ref);
            }
            return Err(ANEError::Internal(format!(
                "Compiled model file was not created: {:?}",
                output_path
            )));
        }

        // Verify the file has content
        let metadata = std::fs::metadata(&output_path).map_err(|e| {
            unsafe {
                agentbridge_model_destroy(model_ref);
            }
            ANEError::Internal(format!("Failed to verify compiled model: {}", e))
        })?;

        if metadata.len() == 0 {
            unsafe {
                agentbridge_model_destroy(model_ref);
            }
            return Err(ANEError::Internal(
                "Compiled model file is empty".to_string(),
            ));
        }

        // Clean up the model reference (we don't need it for the compiled file)
        unsafe {
            agentbridge_model_destroy(model_ref);
        }

        Ok(output_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
    }
}

/// Load a compiled Core ML model and return an opaque reference
/// The raw handle is stored in a thread-local registry for safety
pub fn load_model(path: &str) -> Result<ModelRef> {
    load_model_with_config(path, None)
}

/// Load a compiled Core ML model with specific compute unit configuration
pub fn load_model_with_config(path: &str, compute_units: Option<ComputeUnits>) -> Result<ModelRef> {
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

        // Create model configuration with specified compute units
        let config = MLModelConfiguration {
            compute_units: compute_units
                .map(|cu| match cu {
                    ComputeUnits::CpuOnly => MLComputeUnits::CpuOnly,
                    ComputeUnits::CpuAndGpu => MLComputeUnits::CpuAndGpu,
                    ComputeUnits::CpuAndNeuralEngine => MLComputeUnits::CpuAndNeuralEngine,
                    ComputeUnits::All => MLComputeUnits::All,
                })
                .unwrap_or(MLComputeUnits::CpuAndNeuralEngine),
            allow_low_precision_accumulation_on_gpu: true,
        };

        let config_json = serde_json::to_string(&config)
            .map_err(|e| ANEError::Internal(format!("Failed to serialize config: {}", e)))?;
        
        // Log compute unit configuration for verification
        tracing::debug!(
            "Loading model with compute units: {:?} (config: {})",
            config.compute_units,
            config_json
        );
        
        let config_json_cstr = std::ffi::CString::new(config_json)
            .map_err(|e| ANEError::InvalidInput(format!("Invalid config JSON: {}", e)))?;

        let result = unsafe {
            agentbridge_model_create(
                model_path_cstr.as_ptr(),
                config_json_cstr.as_ptr(),
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
            return Err(ANEError::Internal(format!(
                "Failed to load Core ML model: {}",
                error_msg
            )));
        }

        // Register the handle in the thread-local registry
        let raw_ptr = model_ref as *mut std::ffi::c_void;
        let handle = CoreMlHandle::new(raw_ptr)
            .ok_or_else(|| ANEError::Internal("Null model handle".into()))?;
        let id = registry::register_model(handle);
        
        tracing::debug!(
            "Model loaded successfully with compute units: {:?}, model_ref: {}",
            config.compute_units,
            model_ref
        );
        
        Ok(id)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
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
    CpuAndNeuralEngine,
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

        let result = unsafe { agentbridge_dict_provider_create(&mut provider_ref, &mut error_ptr) };

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
            return Err(ANEError::Internal(format!(
                "Failed to create MLFeatureProvider: {}",
                error_msg
            )));
        }

        let ptr = NonNull::new(provider_ref as *mut std::ffi::c_void)
            .ok_or_else(|| ANEError::Internal("Failed to create MLFeatureProvider".to_string()))?;
        Ok(MLFeatureProvider::new(ptr))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
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

        let result = unsafe { agentbridge_dict_provider_destroy(_prediction.ptr() as u64) };

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
            return Err(ANEError::Internal(format!(
                "Failed to extract output tensor: {}",
                error_msg
            )));
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
        let data = output_data["data"]
            .as_array()
            .ok_or_else(|| ANEError::Internal("Invalid output data format".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        let _shape = output_data["shape"]
            .as_array()
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
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
    }
}

/// Run inference using a raw model handle pointer with multiple inputs (for use in spawn_blocking threads)
/// This function does not access the thread-local registry, making it safe to call
/// from any thread after extracting the handle pointer.
pub fn run_inference_with_handle_multi_input(
    model_handle_ptr: u64,
    input_features: &HashMap<String, MLFeatureValue>,
    _output_name: &str,
) -> Result<Tensor> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        // Create input provider with model handle for state features
        let input_provider =
            MLDictionaryFeatureProvider::from_dictionary(input_features, Some(model_handle_ptr)).map_err(
                |e| ANEError::Internal(format!("Failed to create input provider: {}", e)),
            )?;

        // Run inference using the handle pointer directly
        let mut output_provider_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let inference_result = unsafe {
            agentbridge_model_run_inference(
                model_handle_ptr,
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
            return Err(ANEError::Internal(
                "No output provider returned from inference".to_string(),
            ));
        }

        // Query model metadata for output feature names using the handle
        // We can't use query_model_outputs here because it needs ModelRef and registry access
        // Instead, try common output names for Mistral models
        let output_name = "logits".to_string(); // StatefulMistral typically uses "logits"
        
        let output_name_cstr = CString::new(output_name.clone())
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
                format!("Unknown error extracting output feature '{}'", output_name)
            };
            return Err(ANEError::Internal(error_msg));
        }

        if output_data_ptr.is_null() || output_data_length <= 0 {
            // Clean up the output provider
            unsafe { agentbridge_provider_destroy(output_provider_ref) };
            return Err(ANEError::Internal(
                "No output data returned from inference".to_string(),
            ));
        }

        // Create candle tensor from the output data
        let output_data =
            unsafe { std::slice::from_raw_parts(output_data_ptr, output_data_length as usize) };

        let output_tensor = Tensor::new(output_data, &Device::Cpu).map_err(|e| {
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
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
    }
}

/// Run inference using a raw model handle pointer (for use in spawn_blocking threads)
/// This is a convenience wrapper that creates a single-input feature dictionary.
pub fn run_inference_with_handle(
    model_handle_ptr: u64,
    input_name: &str,
    input_data: &[f32],
    input_shape: &[usize],
) -> Result<Tensor> {
    // Convert input shape to i32 array for Core ML
    let shape_i32: Vec<i32> = input_shape.iter().map(|&x| x as i32).collect();

    // Create input tensor
    let input_array = MLMultiArray::from_slice(input_data, &shape_i32)
        .map_err(|e| ANEError::Internal(format!("Failed to create input tensor: {}", e)))?;

    // Create input feature dictionary
    let mut input_features = HashMap::new();
    input_features.insert(
        input_name.to_string(),
        MLFeatureValue::MultiArray(input_array),
    );

    // Use default output name "logits" for single-input inference
    run_inference_with_handle_multi_input(model_handle_ptr, &input_features, "logits")
}

/// Run inference on a loaded model using opaque reference
/// This version uses ModelRef and accesses the thread-local registry
/// It extracts the handle and delegates to run_inference_with_handle
pub fn run_inference(
    model_ref: ModelRef,
    input_name: &str,
    input_data: &[f32],
    input_shape: &[usize],
) -> Result<Tensor> {
    // Extract model handle pointer from thread-local registry
    let model_handle_ptr: u64 = registry::with_model_handle(model_ref, |handle| {
        handle.as_ptr() as u64
    })
    .ok_or_else(|| ANEError::InvalidInput("Model not found in registry".to_string()))?;

    // Delegate to the handle-based version
    run_inference_with_handle(model_handle_ptr, input_name, input_data, input_shape)
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
pub fn query_model_inputs(model_ref: ModelRef) -> Result<Vec<ModelIOSpec>> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        // Query model info from the FFI layer
        let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let info_result = registry::with_model_handle(model_ref, |model_handle| unsafe {
            agentbridge_model_get_info(model_handle.as_ptr() as u64, &mut info_ptr, &mut error_ptr)
        })
        .ok_or_else(|| ANEError::InvalidInput("Model not found in registry".to_string()))?;

        if info_result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error getting model info".to_string()
            };
            return Err(ANEError::Internal(format!(
                "Failed to get model info: {}",
                error_msg
            )));
        }

        if info_ptr.is_null() {
            return Err(ANEError::Internal("No model info returned".to_string()));
        }

        // Extract and parse JSON string
        let info_json_str = unsafe {
            let cstr = std::ffi::CStr::from_ptr(info_ptr);
            let info_str = cstr.to_string_lossy().to_string();
            agentbridge_free_string(info_ptr);
            info_str
        };

        // Parse JSON to extract input descriptions
        let info_json: serde_json::Value = serde_json::from_str(&info_json_str)
            .map_err(|e| ANEError::Internal(format!("Failed to parse model info JSON: {}", e)))?;

        let input_descriptions = info_json["inputDescriptions"].as_array().ok_or_else(|| {
            ANEError::Internal("inputDescriptions not found or not an array".to_string())
        })?;

        // Check if this is a stateful model (Mistral, etc.)
        // State features like keyCache may not appear in inputDescriptions but are still required
        // Check multiple possible metadata locations
        let metadata = info_json["modelDescription"]["metadata"].as_object();
        let model_name = metadata
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();
        let model_desc = metadata
            .and_then(|m| m.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        // Also check the model path/identifier if available
        let model_identifier = info_json["modelDescription"]["metadata"]["identifier"]
            .as_str()
            .unwrap_or("")
            .to_lowercase();

        let is_stateful_model = model_name.contains("mistral")
            || model_name.contains("stateful")
            || model_desc.contains("mistral")
            || model_desc.contains("stateful")
            || model_identifier.contains("mistral")
            || model_identifier.contains("stateful");

        let mut input_specs = Vec::new();

        for input_desc in input_descriptions {
            let name = input_desc["name"]
                .as_str()
                .ok_or_else(|| {
                    ANEError::Internal("Input description missing 'name' field".to_string())
                })?
                .to_string();

            let feature_type = input_desc["type"].as_str().unwrap_or("unknown").to_string();

            // Check if this is a state feature (MLState)
            let is_state_feature = feature_type.to_lowercase().contains("state")
                || name.to_lowercase().contains("keycache")
                || name.to_lowercase().contains("valuecache");

            // Determine data type and shape
            let (dtype, shape, batch_capable) = if is_state_feature {
                // State feature - use "state" as dtype
                ("state".to_string(), vec![], false)
            } else if let Some(shape_array) = input_desc["shape"].as_array() {
                // MultiArray type
                let shape: Vec<i32> = shape_array
                    .iter()
                    .filter_map(|v| v.as_i64().map(|i| i as i32))
                    .collect();

                let data_type_str = input_desc["dataType"]
                    .as_str()
                    .unwrap_or("float32")
                    .to_string();

                // Determine if batch-capable (first dimension is variable or -1)
                let batch_capable = shape.is_empty() || shape[0] == -1 || shape[0] == 1;

                (data_type_str, shape, batch_capable)
            } else if input_desc["imageConstraint"].is_object() {
                // Image type
                let image_constraint =
                    input_desc["imageConstraint"].as_object().ok_or_else(|| {
                        ANEError::Internal("imageConstraint is not an object".to_string())
                    })?;

                let width = image_constraint["width"]
                    .as_i64()
                    .map(|w| w as i32)
                    .unwrap_or(-1);
                let height = image_constraint["height"]
                    .as_i64()
                    .map(|h| h as i32)
                    .unwrap_or(-1);

                // Image shape: [batch, channels, height, width] or [height, width, channels]
                let shape = if width > 0 && height > 0 {
                    vec![height, width, 3] // Default to RGB
                } else {
                    vec![-1, -1, 3] // Variable size
                };

                ("image".to_string(), shape, true)
            } else {
                // Unknown or unsupported type
                ("unknown".to_string(), vec![-1], false)
            };

            input_specs.push(ModelIOSpec {
                name,
                dtype,
                shape,
                batch_capable,
            });
        }

        // For stateful models (like Mistral), add keyCache state feature if not already present
        // Note: Core ML state features don't appear in inputDescriptions but are still required
        // Heuristic: If we have inputIds and causalMask but no keyCache, this is likely a Mistral model
        // that requires keyCache as a state feature
        let has_input_ids = input_specs
            .iter()
            .any(|spec| spec.name.to_lowercase() == "inputids");
        let has_causal_mask = input_specs
            .iter()
            .any(|spec| spec.name.to_lowercase() == "causalmask");
        let has_keycache = input_specs
            .iter()
            .any(|spec| spec.name.to_lowercase().contains("keycache"));

        // Mistral models typically have inputIds and causalMask but keyCache is required as state
        // Always add keyCache if we detect this pattern (safe heuristic for Mistral models)
        if has_input_ids && has_causal_mask && !has_keycache {
            input_specs.push(ModelIOSpec {
                name: "keyCache".to_string(),
                dtype: "state".to_string(),
                shape: vec![],
                batch_capable: false,
            });
        }

        // Also check metadata-based detection for other stateful models
        if is_stateful_model && !has_keycache {
            input_specs.push(ModelIOSpec {
                name: "keyCache".to_string(),
                dtype: "state".to_string(),
                shape: vec![],
                batch_capable: false,
            });
        }

        if input_specs.is_empty() {
            return Err(ANEError::Internal(
                "No input descriptions found in model info".to_string(),
            ));
        }

        Ok(input_specs)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
    }
}

/// Query model outputs
pub fn query_model_outputs(model_ref: ModelRef) -> Result<Vec<ModelIOSpec>> {
    if !coreml_runtime_available() {
        return Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        // Query model info from the FFI layer
        let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let info_result = registry::with_model_handle(model_ref, |model_handle| unsafe {
            agentbridge_model_get_info(model_handle.as_ptr() as u64, &mut info_ptr, &mut error_ptr)
        })
        .ok_or_else(|| ANEError::InvalidInput("Model not found in registry".to_string()))?;

        if info_result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error getting model info".to_string()
            };
            return Err(ANEError::Internal(format!(
                "Failed to get model info: {}",
                error_msg
            )));
        }

        if info_ptr.is_null() {
            return Err(ANEError::Internal("No model info returned".to_string()));
        }

        // Extract and parse JSON string
        let info_json_str = unsafe {
            let cstr = std::ffi::CStr::from_ptr(info_ptr);
            let info_str = cstr.to_string_lossy().to_string();
            agentbridge_free_string(info_ptr);
            info_str
        };

        // Parse JSON to extract output descriptions
        let info_json: serde_json::Value = serde_json::from_str(&info_json_str)
            .map_err(|e| ANEError::Internal(format!("Failed to parse model info JSON: {}", e)))?;

        let output_descriptions = info_json["outputDescriptions"].as_array().ok_or_else(|| {
            ANEError::Internal("outputDescriptions not found or not an array".to_string())
        })?;

        let mut output_specs = Vec::new();

        for output_desc in output_descriptions {
            let name = output_desc["name"]
                .as_str()
                .ok_or_else(|| {
                    ANEError::Internal("Output description missing 'name' field".to_string())
                })?
                .to_string();

            let _feature_type = output_desc["type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            // Determine data type and shape
            let (dtype, shape, batch_capable) =
                if let Some(shape_array) = output_desc["shape"].as_array() {
                    // MultiArray type
                    let shape: Vec<i32> = shape_array
                        .iter()
                        .filter_map(|v| v.as_i64().map(|i| i as i32))
                        .collect();

                    let data_type_str = output_desc["dataType"]
                        .as_str()
                        .unwrap_or("float32")
                        .to_string();

                    // Determine if batch-capable (first dimension is variable or -1)
                    let batch_capable = shape.is_empty() || shape[0] == -1 || shape[0] == 1;

                    (data_type_str, shape, batch_capable)
                } else if output_desc["imageConstraint"].is_object() {
                    // Image type
                    let image_constraint =
                        output_desc["imageConstraint"].as_object().ok_or_else(|| {
                            ANEError::Internal("imageConstraint is not an object".to_string())
                        })?;

                    let width = image_constraint["width"]
                        .as_i64()
                        .map(|w| w as i32)
                        .unwrap_or(-1);
                    let height = image_constraint["height"]
                        .as_i64()
                        .map(|h| h as i32)
                        .unwrap_or(-1);

                    let shape = if width > 0 && height > 0 {
                        vec![height, width, 3] // Default to RGB
                    } else {
                        vec![-1, -1, 3] // Variable size
                    };

                    ("image".to_string(), shape, true)
                } else {
                    // Unknown or unsupported type - default to float32
                    ("float32".to_string(), vec![-1], false)
                };

            output_specs.push(ModelIOSpec {
                name,
                dtype,
                shape,
                batch_capable,
            });
        }

        if output_specs.is_empty() {
            // Fallback: return default output spec if no outputs found
            return Ok(vec![ModelIOSpec {
                name: "output".to_string(),
                dtype: "float32".to_string(),
                shape: vec![-1, -1, -1],
                batch_capable: true,
            }]);
        }

        Ok(output_specs)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(ANEError::Internal(
            "Core ML not available on this platform".to_string(),
        ))
    }
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
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_is_cached(
        identifier: *const std::ffi::c_char,
        channel: *const std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_remove_cached(
        identifier: *const std::ffi::c_char,
        channel: *const std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_get_cache_stats(
        out_stats: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_clear_cache(out_error: *mut *mut std::ffi::c_char) -> i32;

    pub fn agentbridge_model_create(
        model_path: *const std::ffi::c_char,
        config_json: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_destroy(model_ref: u64) -> i32;

    pub fn agentbridge_model_get_info(
        model_ref: u64,
        out_info: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_mistral_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_mistral_generate(
        model_ref: u64,
        prompt: *const std::ffi::c_char,
        max_tokens: i32,
        temperature: f32,
        out_text: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_mistral_encode(
        text: *const std::ffi::c_char,
        out_tokens: *mut *mut i32,
        out_token_count: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_mistral_decode(
        tokens: *const i32,
        token_count: i32,
        out_text: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_mistral_free_tokens(tokens: *mut i32, count: i32);

    pub fn agentbridge_free_string(ptr: *mut std::ffi::c_char);

    pub fn agentbridge_array_create_float32(
        data: *const f32,
        data_length: i32,
        shape: *const i32,
        shape_length: i32,
        out_array_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
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
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_dict_provider_create(
        out_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_dict_provider_set_feature_float32(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        data: *const f32,
        shape: *const i32,
        shape_length: i32,
        out_error: *mut *mut std::ffi::c_char,
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
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    // KV cache state management functions
    pub fn agentbridge_kv_state_create(
        model_ref: u64,
        n_layers: i32,
        n_kv_heads: i32,
        head_dim: i32,
        max_seq_len: i32,
        out_state_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_kv_state_destroy(state_ref: u64) -> i32;

    pub fn agentbridge_model_run_inference_with_kv(
        model_ref: u64,
        input_provider_ref: u64,
        kv_state_ref: u64,
        out_output_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_kv_state_step(
        kv_state_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_kv_state_reset(
        kv_state_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_provider_destroy(provider_ref: u64) -> i32;

    pub fn agentbridge_provider_get_feature_float32(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        out_data: *mut *mut f32,
        out_shape: *mut *mut i32,
        out_shape_length: *mut i32,
        out_data_length: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_free_array_data(data: *mut f32) -> i32;

    pub fn agentbridge_audio_whisper_create(
        model_path: *const std::ffi::c_char,
        model_size: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_audio_whisper_transcribe(
        model_ref: u64,
        audio_path: *const std::ffi::c_char,
        language: *const std::ffi::c_char,
        out_text: *mut *mut std::ffi::c_char,
        out_segments_json: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_audio_speech_create(
        language: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_audio_speech_transcribe(
        model_ref: u64,
        audio_path: *const std::ffi::c_char,
        out_text: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_vision_yolo_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_vision_yolo_detect(
        model_ref: u64,
        image_data: *const u8,
        data_length: i32,
        confidence_threshold: f32,
        out_detections_json: *mut *mut std::ffi::c_char,
        out_detection_count: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_vision_ocr_create(
        language: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_vision_ocr_extract(
        model_ref: u64,
        image_data: *const u8,
        data_length: i32,
        out_text: *mut *mut std::ffi::c_char,
        out_confidence: *mut f32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_diffusion_create(
        model_path: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
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
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_text_diffusion_free_image(image_data: *mut u8);

    pub fn agentbridge_system_get_metrics(
        out_metrics: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_system_profile_start(
        session_name: *const std::ffi::c_char,
        out_session_id: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_system_profile_stop(
        session_id: u64,
        out_report: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;
}
