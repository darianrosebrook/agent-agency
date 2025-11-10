// ============================================================================
// Core ML Direct Implementation
// ============================================================================
// This module provides a direct Core ML implementation using the agentbridge
// FFI functions for Core ML model operations.

use schemars::JsonSchema;
use crate::ane::ane_errors::{ANEError, Result};
use std::path::Path;
use std::ffi::CString;
use std::collections::HashMap;

// Import runtime check and FFI functions from model module
use super::model::{coreml_runtime_available, coreml_unavailable_error};

// FFI declarations for agentbridge functions
#[cfg(target_os = "macos")]
extern "C" {
    fn agentbridge_model_create(
        model_path: *const std::ffi::c_char,
        config_json: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_model_destroy(model_ref: u64) -> i32;

    fn agentbridge_model_get_info(
        model_ref: u64,
        out_info: *mut *mut std::ffi::c_char,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_model_run_inference(
        model_ref: u64,
        input_provider_ref: u64,
        out_output_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_dict_provider_create(
        out_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_dict_provider_destroy(provider_ref: u64) -> i32;

    fn agentbridge_provider_destroy(provider_ref: u64) -> i32;

    fn agentbridge_dict_provider_set_feature_multiarray(
        provider_ref: u64,
        name: *const std::ffi::c_char,
        array_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_array_create_float32(
        data: *const f32,
        data_len: i32,
        shape: *const i32,
        shape_len: i32,
        out_array_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_array_destroy(array_ref: u64) -> i32;

    fn agentbridge_provider_get_feature_float32(
        provider_ref: u64,
        name: *const std::ffi::c_char,
        out_data: *mut *mut f32,
        out_shape: *mut *mut i32,
        out_shape_len: *mut i32,
        out_data_len: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    fn agentbridge_free_string(ptr: *mut std::ffi::c_char);

    fn agentbridge_free_array_data(data_ptr: *mut f32) -> i32;
}

// Check if we're on Apple Silicon
const TARGET_APPLE_SILICON: bool = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");

/// Core ML model wrapper with real Core ML integration
#[derive(Debug)]
pub struct CoreMLModel {
    /// Model path
    path: String,
    /// Model handle (loaded on first use)
    model_ref: Option<u64>,
}

impl CoreMLModel {
    pub fn from_path(path: &Path) -> Result<Self> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Unavailable);
        }

        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let path_str = path.to_str()
            .ok_or_else(|| ANEError::InvalidInput("Invalid path encoding".to_string()))?;

        Ok(CoreMLModel {
            path: path_str.to_string(),
            model_ref: None,
        })
    }

    /// Load the model if not already loaded
    fn ensure_loaded(&mut self) -> Result<()> {
        if self.model_ref.is_some() {
            return Ok(());
        }

        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        #[cfg(target_os = "macos")]
        {
            let path_cstr = CString::new(self.path.as_str())
                .map_err(|e| ANEError::InvalidInput(format!("Invalid path string: {}", e)))?;

            let mut model_ref: u64 = 0;
            let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

            let result = unsafe {
                agentbridge_model_create(
                    path_cstr.as_ptr(),
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
                        agentbridge_free_string(error_ptr);
                        msg
                    }
                } else {
                    "Unknown error loading Core ML model".to_string()
                };
                return Err(ANEError::Internal(format!("Failed to load Core ML model: {}", error_msg)));
            }

            if model_ref == 0 {
                return Err(ANEError::Internal("Failed to create model handle".to_string()));
            }

            self.model_ref = Some(model_ref);
            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(ANEError::Internal("Core ML not available on this platform".to_string()))
        }
    }

    /// Get the model reference, loading if necessary
    fn get_model_ref(&mut self) -> Result<u64> {
        self.ensure_loaded()?;
        self.model_ref.ok_or_else(|| ANEError::Internal("Model not loaded".to_string()))
    }

    pub fn prediction_from_features(&mut self, features: &MLFeatureProvider) -> Result<MLFeatureProvider> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::NotImplemented("Core ML prediction only supported on macOS".to_string()));
        }

        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let model_ref = self.get_model_ref()?;

        #[cfg(target_os = "macos")]
        {
            // Create input feature provider using agentbridge
            let mut input_provider_ref: u64 = 0;
            let mut create_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

            let create_result = unsafe {
                agentbridge_dict_provider_create(
                    &mut input_provider_ref,
                    &mut create_error_ptr,
                )
            };

            if create_result != 0 {
                let error_msg = if !create_error_ptr.is_null() {
                    unsafe {
                        let cstr = std::ffi::CStr::from_ptr(create_error_ptr);
                        let msg = cstr.to_string_lossy().to_string();
                        agentbridge_free_string(create_error_ptr);
                        msg
                    }
                } else {
                    "Unknown error creating input provider".to_string()
                };
                return Err(ANEError::Internal(format!("Failed to create input provider: {}", error_msg)));
            }

            // Set features in the input provider
            for (name, value) in &features.features {
                match value {
                    MLFeatureValue::MultiArray(array) => {
                        let name_cstr = CString::new(name.as_str())
                            .map_err(|e| ANEError::InvalidInput(format!("Invalid feature name: {}", e)))?;

                        // Create MLMultiArray through agentbridge
                        let mut array_ref: u64 = 0;
                        let mut array_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

                        let array_result = unsafe {
                            agentbridge_array_create_float32(
                                array.data.as_ptr(),
                                array.data.len() as i32,
                                array.shape.as_ptr(),
                                array.shape.len() as i32,
                                &mut array_ref,
                                &mut array_error_ptr,
                            )
                        };

                        if array_result != 0 {
                            unsafe { agentbridge_dict_provider_destroy(input_provider_ref) };
                            let error_msg = if !array_error_ptr.is_null() {
                                unsafe {
                                    let cstr = std::ffi::CStr::from_ptr(array_error_ptr);
                                    let msg = cstr.to_string_lossy().to_string();
                                    agentbridge_free_string(array_error_ptr);
                                    msg
                                }
                            } else {
                                "Unknown error creating array".to_string()
                            };
                            return Err(ANEError::Internal(format!("Failed to create array: {}", error_msg)));
                        }

                        // Set feature in provider
                        let mut feature_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
                        let feature_result = unsafe {
                            agentbridge_dict_provider_set_feature_multiarray(
                                input_provider_ref,
                                name_cstr.as_ptr(),
                                array_ref,
                                &mut feature_error_ptr,
                            )
                        };

                        if feature_result != 0 {
                            unsafe {
                                agentbridge_array_destroy(array_ref);
                                agentbridge_dict_provider_destroy(input_provider_ref);
                            }
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
                            return Err(ANEError::Internal(error_msg));
                        }
                    }
                    _ => {
                        // For now, only MultiArray is supported
                        unsafe { agentbridge_dict_provider_destroy(input_provider_ref) };
                        return Err(ANEError::NotImplemented(format!("Feature type not yet supported for feature '{}'", name)));
                    }
                }
            }

            // Run inference
            let mut output_provider_ref: u64 = 0;
            let mut inference_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

            let inference_result = unsafe {
                agentbridge_model_run_inference(
                    model_ref,
                    input_provider_ref,
                    &mut output_provider_ref,
                    &mut inference_error_ptr,
                )
            };

            // Clean up input provider
            unsafe { agentbridge_dict_provider_destroy(input_provider_ref) };

            if inference_result != 0 {
                let error_msg = if !inference_error_ptr.is_null() {
                    unsafe {
                        let cstr = std::ffi::CStr::from_ptr(inference_error_ptr);
                        let msg = cstr.to_string_lossy().to_string();
                        agentbridge_free_string(inference_error_ptr);
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

            // Extract output features (simplified: assume single output feature)
            // In a real implementation, we'd query the model metadata for output feature names
            let output_name = "output"; // Default output name
            let output_name_cstr = CString::new(output_name)
                .map_err(|e| ANEError::Internal(format!("Invalid output name: {}", e)))?;

            let mut output_data_ptr: *mut f32 = std::ptr::null_mut();
            let mut output_shape_ptr: *mut i32 = std::ptr::null_mut();
            let mut output_shape_len: i32 = 0;
            let mut output_data_len: i32 = 0;
            let mut extract_error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

            let extract_result = unsafe {
                agentbridge_provider_get_feature_float32(
                    output_provider_ref,
                    output_name_cstr.as_ptr(),
                    &mut output_data_ptr,
                    &mut output_shape_ptr,
                    &mut output_shape_len,
                    &mut output_data_len,
                    &mut extract_error_ptr,
                )
            };

            if extract_result != 0 {
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

            if output_data_ptr.is_null() || output_data_len <= 0 {
                unsafe { agentbridge_provider_destroy(output_provider_ref) };
                return Err(ANEError::Internal("No output data returned from inference".to_string()));
            }

            // Extract output data
            let output_data = unsafe {
                std::slice::from_raw_parts(output_data_ptr, output_data_len as usize).to_vec()
            };

            let output_shape = if !output_shape_ptr.is_null() && output_shape_len > 0 {
                unsafe {
                    std::slice::from_raw_parts(output_shape_ptr, output_shape_len as usize).to_vec()
                }
            } else {
                vec![output_data_len]
            };

            // Clean up FFI-allocated resources
            unsafe {
                agentbridge_provider_destroy(output_provider_ref);
                agentbridge_free_array_data(output_data_ptr);
                if !output_shape_ptr.is_null() {
                    agentbridge_free_array_data(output_shape_ptr as *mut f32);
                }
            }

            // Build output feature provider
            let mut output_features = HashMap::new();
            output_features.insert(
                output_name.to_string(),
                MLFeatureValue::MultiArray(MLMultiArray {
                    data: output_data,
                    shape: output_shape,
                }),
            );

            Ok(MLFeatureProvider {
                features: output_features,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(ANEError::Internal("Core ML not available on this platform".to_string()))
        }
    }

    pub fn model_info(&mut self) -> Result<String> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::NotImplemented("Core ML model info only supported on macOS".to_string()));
        }

        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let model_ref = self.get_model_ref()?;

        #[cfg(target_os = "macos")]
        {
            let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
            let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

            let result = unsafe {
                agentbridge_model_get_info(
                    model_ref,
                    &mut info_ptr,
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
                    "Unknown error getting model info".to_string()
                };
                return Err(ANEError::Internal(format!("Failed to get model info: {}", error_msg)));
            }

            if info_ptr.is_null() {
                return Err(ANEError::Internal("No model info returned".to_string()));
            }

            let info = unsafe {
                let cstr = std::ffi::CStr::from_ptr(info_ptr);
                let info_str = cstr.to_string_lossy().to_string();
                agentbridge_free_string(info_ptr);
                info_str
            };

            Ok(info)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(ANEError::Internal("Core ML not available on this platform".to_string()))
        }
    }
}

impl Drop for CoreMLModel {
    fn drop(&mut self) {
        if let Some(model_ref) = self.model_ref {
            if coreml_runtime_available() {
                unsafe {
                    let _ = agentbridge_model_destroy(model_ref);
                }
            }
        }
    }
}

// Simple MLFeatureProvider implementation
pub struct MLFeatureProvider {
    pub features: std::collections::HashMap<String, MLFeatureValue>,
}

impl MLFeatureProvider {
    pub fn from_dictionary(dict: &std::collections::HashMap<String, MLFeatureValue>) -> Result<Self> {
        Ok(MLFeatureProvider {
            features: dict.clone(),
        })
    }
}

// Simple MLFeatureValue implementation
#[derive(Debug, Clone, JsonSchema)]
pub enum MLFeatureValue {
    MultiArray(MLMultiArray),
    String(String),
    Int64(i64),
    Double(f64),
}

// Simple MLMultiArray implementation
#[derive(Debug, Clone, JsonSchema)]
pub struct MLMultiArray {
    pub data: Vec<f32>,
    pub shape: Vec<i32>,
}

impl MLMultiArray {
    pub fn from_slice(data: &[f32], shape: &[i32]) -> Result<Self> {
        if shape.is_empty() {
            return Err(ANEError::InvalidInput("Shape cannot be empty".to_string()));
        }

        let total_elements: usize = shape.iter().map(|&dim| dim as usize).product();
        if data.len() != total_elements {
            return Err(ANEError::InvalidInput(format!(
                "Data length {} doesn't match shape product {}",
                data.len(), total_elements
            )));
        }

        Ok(MLMultiArray {
            data: data.to_vec(),
            shape: shape.to_vec(),
        })
    }
}

// Simple MLModelConfiguration implementation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct MLModelConfiguration {
    pub compute_units: MLComputeUnits,
    pub allow_low_precision_accumulation_on_gpu: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum MLComputeUnits {
    All,
    CpuOnly,
    CpuAndGpu,
    CpuAndNeuralEngine,
}

impl Default for MLComputeUnits {
    fn default() -> Self {
        MLComputeUnits::All
    }
}

impl Default for MLModelConfiguration {
    fn default() -> Self {
        MLModelConfiguration {
            compute_units: MLComputeUnits::All,
            allow_low_precision_accumulation_on_gpu: true,
        }
    }
}

// Simple model reference wrapper
#[derive(Debug, Clone, JsonSchema)]
pub struct ModelRef (u64);

impl ModelRef {
    pub fn new(handle: u64) -> Self {
        ModelRef(handle)
    }

    pub fn handle(&self) -> u64 {
        self.0
    }
}

// Test function to verify Core ML integration
pub fn test_coreml_integration_inner() -> Result<()> {
    if !TARGET_APPLE_SILICON {
        return Err(ANEError::Unavailable);
    }

    // Test basic functionality
    let config = MLModelConfiguration::default();
    assert_eq!(config.compute_units, MLComputeUnits::All);
    assert!(config.allow_low_precision_accumulation_on_gpu);

    // Test MLMultiArray creation
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let shape = vec![2, 2];
    let array = MLMultiArray::from_slice(&data, &shape)?;
    assert_eq!(array.data, data);
    assert_eq!(array.shape, shape);

    // Test MLFeatureProvider
    let mut features = std::collections::HashMap::new();
    features.insert("input".to_string(), MLFeatureValue::MultiArray(array));
    let provider = MLFeatureProvider::from_dictionary(&features)?;
    assert_eq!(provider.features.len(), 1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coreml_model_creation() {
        let path = Path::new("/tmp/test.mlmodel");
        let result = CoreMLModel::from_path(path);
        
        if TARGET_APPLE_SILICON {
            assert!(result.is_ok());
            let model = result.unwrap();
            assert_eq!(model.path, "/tmp/test.mlmodel");
        } else {
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ANEError::Unavailable));
        }
    }

    #[test]
    fn test_mlmultiarray_creation() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let array = MLMultiArray::from_slice(&data, &shape).unwrap();
        assert_eq!(array.data, data);
        assert_eq!(array.shape, shape);
    }

    #[test]
    fn test_mlmultiarray_invalid_shape() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![];
        let result = MLMultiArray::from_slice(&data, &shape);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ANEError::InvalidInput(_)));
    }

    #[test]
    fn test_mlmultiarray_size_mismatch() {
        let data = vec![1.0, 2.0, 3.0]; // 3 elements
        let shape = vec![2, 2]; // expects 4 elements
        let result = MLMultiArray::from_slice(&data, &shape);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ANEError::InvalidInput(_)));
    }

    #[test]
    fn test_coreml_integration() {
        let result = test_coreml_integration_inner();
        
        if TARGET_APPLE_SILICON {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ANEError::Unavailable));
        }
    }
}
