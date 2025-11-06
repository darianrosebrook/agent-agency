//! Core ML model operations
//!
//! This module contains the MLModel implementation and core model operations,
//! including loading, saving, prediction, and resource management.

use super::types::*;
use crate::ane::ane_errors::{ANEError, Result};
use schemars::JsonSchema;
use std::ffi::CString;
use std::path::Path;
use std::ptr::NonNull;

// Import the coreml_module for FFI access
use super::coreml_module as coreml;

// FFI declarations for agentbridge functions
#[cfg(target_os = "macos")]
extern "C" {
    pub fn agentbridge_array_create_float32(
        data: *const f32,
        data_len: i32,
        shape: *const i32,
        shape_len: i32,
        out_array_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_array_destroy(array_ref: u64) -> i32;

    pub fn agentbridge_dict_provider_create(
        out_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_dict_provider_destroy(provider_ref: u64) -> i32;

    pub fn agentbridge_provider_destroy(provider_ref: u64) -> i32;

    pub fn agentbridge_free_string(ptr: *mut std::ffi::c_char);

    pub fn agentbridge_dict_provider_set_feature_multiarray(
        provider_ref: u64,
        name: *const std::ffi::c_char,
        array_ref: u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_dict_provider_set_feature_float32(
        provider_ref: u64,
        feature_name: *const std::ffi::c_char,
        data: *const f32,
        shape: *const i32,
        shape_length: i32,
        out_error: *mut *mut std::ffi::c_char
    ) -> i32;

    pub fn agentbridge_model_create(
        model_path: *const std::ffi::c_char,
        config_json: *const std::ffi::c_char,
        out_model_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_model_create_from_data(
        model_data: *const u8,
        data_len: usize,
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

    pub fn agentbridge_provider_get_feature_float32(
        provider_ref: u64,
        name: *const std::ffi::c_char,
        out_data: *mut *mut f32,
        out_shape: *mut *mut i32,
        out_shape_len: *mut i32,
        out_data_len: *mut i32,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_free_array_data(data_ptr: *mut f32) -> i32;

}

// FFI declarations for runtime checks
#[cfg(target_os = "macos")]
extern "C" {
    /// Optional Swift/ObjC shim (returning true iff CoreML APIs are usable on this OS)
    fn coreml_can_load_models() -> bool;
}

/// Runtime capability check for Core ML availability
///
/// Core ML requires:
/// - macOS operating system
/// - ARM64 architecture (Apple Silicon)
///
/// In a Universal2 app each slice is compiled independently; `cfg!(target_arch)` already
/// reflects the active slice. The optional Swift shim provides the only genuinely runtime
/// dimension needed (macOS version/framework availability).
#[inline]
pub fn coreml_runtime_available() -> bool {
    // Strongest gates first (compile-time slice):
    if !cfg!(target_os = "macos") {
        return false;
    }
    if !cfg!(target_arch = "aarch64") {
        return false;
    }

    // Optional runtime probe to catch old macOS / missing symbols gracefully.
    // If you haven't wired the shim yet, feature-gate this call.
    #[cfg(feature = "coreml_probe")]
    unsafe {
        return coreml_can_load_models();
    }

    #[cfg(not(feature = "coreml_probe"))]
    {
        true
    }
}

/// Prescriptive error message for Core ML unavailability
pub fn coreml_unavailable_error() -> ANEError {
    // Keep this prescriptive and short.
    ANEError::Internal(
        "Core ML unavailable: this process is not the arm64 macOS slice or the OS lacks Core ML mlprogram support. \
         Fix: build/run `aarch64-apple-darwin` (or ship a Universal2 and ensure arm64 launches). \
         Example: `cargo build --target aarch64-apple-darwin`."
            .to_string(),
    )
}

// Runtime functions are now properly declared above

impl Drop for MLMultiArray {
    fn drop(&mut self) {
        if coreml_runtime_available() {
            let array_ref = self.ptr() as u64;
            let result = unsafe { agentbridge_array_destroy(array_ref) };
            if result != 0 {
                tracing::warn!("Failed to destroy Core ML array handle {}", array_ref);
            }
        }
        tracing::debug!("Dropping MLMultiArray");
    }
}

impl Drop for MLFeatureProvider {
    fn drop(&mut self) {
        if coreml_runtime_available() {
            let provider_ref = self.ptr() as u64;
            let result = unsafe { agentbridge_provider_destroy(provider_ref) };
            if result != 0 {
                tracing::warn!("Failed to destroy Core ML provider handle {}", provider_ref);
            }
        }
        tracing::debug!("Dropping MLFeatureProvider");
    }
}

impl Drop for MLDictionaryFeatureProvider {
    fn drop(&mut self) {
        if coreml_runtime_available() {
            let provider_ref = self.ptr() as u64;
            let result = unsafe { agentbridge_dict_provider_destroy(provider_ref) };
            if result != 0 {
                tracing::warn!("Failed to destroy Core ML dictionary provider handle {}", provider_ref);
            }
        }
        tracing::debug!("Dropping MLDictionaryFeatureProvider");
    }
}

impl MLModel {
    /// Load a Core ML model from a compiled .mlmodelc file
    pub fn from_path(path: &std::path::Path) -> std::result::Result<Self, String> {
        if !coreml_runtime_available() {
            return Err("Core ML not available on this platform".to_string());
        }

        let path_str = path.to_str()
            .ok_or_else(|| "Invalid path encoding".to_string())?;

        let path_cstr = CString::new(path_str)
            .map_err(|e| format!("Invalid path string: {}", e))?;

        let mut model_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_model_create(
                path_cstr.as_ptr(),
                std::ptr::null(), // No config for now
                &mut model_ref,
                &mut error_ptr
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
            return Err(error_msg);
        }

        if model_ref == 0 {
            return Err("Failed to create model handle".to_string());
        }

        Ok(MLModel::new(model_ref))
    }

    /// Compile a .mlmodel file to .mlmodelc format
    pub fn compile_model_at_url(url: &str, error: &mut Option<String>) -> std::result::Result<Self, String> {
        if !coreml_runtime_available() {
            return Err("Core ML not available on this platform".to_string());
        }

        let url_cstr = CString::new(url)
            .map_err(|e| format!("Invalid URL string: {}", e))?;

        let mut model_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_model_create(
                url_cstr.as_ptr(),
                std::ptr::null(), // No config for now
                &mut model_ref,
                &mut error_ptr
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
                "Unknown error compiling Core ML model".to_string()
            };
            *error = Some(error_msg.clone());
            return Err(error_msg);
        }

        if model_ref == 0 {
            let error_msg = "Failed to create compiled model handle".to_string();
            *error = Some(error_msg.clone());
            return Err(error_msg);
        }

        Ok(MLModel::new(model_ref))
    }

    /// Save a compiled model to a file path
    pub fn save_to_path(&self, path: &std::path::Path) -> std::result::Result<(), String> {
        // Model saving is only supported on macOS with Core ML
        if !coreml_runtime_available() {
            return Err("Core ML not available on this platform".to_string());
        }

        use std::ffi::CString;

        // Get model information from the FFI layer
        let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let info_result = unsafe {
            agentbridge_model_get_info(
                self.handle(),
                &mut info_ptr,
                &mut error_ptr,
            )
        };

        if info_result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    std::ffi::CStr::from_ptr(error_ptr)
                        .to_string_lossy()
                        .to_string()
                }
            } else {
                "Unknown error getting model info".to_string()
            };
            return Err(format!("Failed to get model info: {}", error_msg));
        }

        if info_ptr.is_null() {
            return Err("No model info available".to_string());
        }

        let model_info_json = unsafe {
            std::ffi::CStr::from_ptr(info_ptr)
                .to_string_lossy()
                .to_string()
        };

        // Free the info string
        unsafe {
            agentbridge_free_string(info_ptr);
        }

        // Parse the JSON to extract file path
        let model_info: serde_json::Value = serde_json::from_str(&model_info_json)
            .map_err(|e| format!("Failed to parse model info: {}", e))?;

        let source_path = model_info["path"].as_str()
            .ok_or_else(|| "Model info does not contain path".to_string())?;

        // Copy the compiled model file to the destination
        let source_path = std::path::Path::new(source_path);
        if !source_path.exists() {
            return Err(format!("Source model file does not exist: {:?}", source_path));
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        // Copy the file
        std::fs::copy(source_path, path)
            .map_err(|e| format!("Failed to copy model file: {}", e))?;

        // Verify the copy was successful
        if !path.exists() {
            return Err("Model file copy verification failed".to_string());
        }

        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to verify copied file: {}", e))?;

        if metadata.len() == 0 {
            return Err("Copied model file is empty".to_string());
        }

        Ok(())
    }

// MLMultiArray implementation
    /// Get model information
    pub fn model_info(&self) -> std::result::Result<String, String> {
        if !coreml_runtime_available() {
            return Err("Core ML not available on this platform".to_string());
        }

        let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_model_get_info(
                self.handle(),
                &mut info_ptr,
                &mut error_ptr
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
            return Err(error_msg);
        }

        if info_ptr.is_null() {
            return Err("No model info returned".to_string());
        }

        let info = unsafe {
            let cstr = std::ffi::CStr::from_ptr(info_ptr);
            let info_str = cstr.to_string_lossy().to_string();
            agentbridge_free_string(info_ptr);
            info_str
        };

        Ok(info)
    }
}

impl Drop for MLModel {
    fn drop(&mut self) {
        if coreml_runtime_available() && self.handle() != 0 {
            let result = unsafe { agentbridge_model_destroy(self.handle()) };
            if result != 0 {
                tracing::warn!("Failed to destroy Core ML model handle {}", self.handle());
            }
        }
    }
}
