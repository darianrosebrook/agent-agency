//! KV cache state management for efficient inference
//!
//! This module provides key-value cache management for stateful inference operations,
//! enabling efficient sequential processing in transformer models.

use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::compat::registry::ModelRef;
use crate::ane::compat::types::{KvStateHandle, MLFeatureProvider};

// FFI declarations for KV cache functions
#[cfg(target_os = "macos")]
extern "C" {
    pub fn agentbridge_kv_state_create(
        model_ref: u64,
        n_layers: i32,
        n_kv_heads: i32,
        head_dim: i32,
        max_seq_len: i32,
        out_kv_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

    pub fn agentbridge_kv_state_destroy(kv_ref: u64) -> i32;

    pub fn agentbridge_kv_state_step(kv_ref: u64, out_error: *mut *mut std::ffi::c_char) -> i32;

    pub fn agentbridge_kv_state_reset(kv_ref: u64, out_error: *mut *mut std::ffi::c_char) -> i32;

    pub fn agentbridge_model_run_inference_with_kv(
        model_ref: u64,
        kv_ref: u64,
        input_provider_ref: u64,
        out_output_provider_ref: *mut u64,
        out_error: *mut *mut std::ffi::c_char,
    ) -> i32;

}

// Import runtime functions from model module
use super::model::{coreml_runtime_available, coreml_unavailable_error};

// Import FFI functions from coreml_module
use super::coreml_module::agentbridge_free_string;

impl KvStateHandle {
    /// Create a new KV cache state for a model
    pub fn create(
        model_ref: &ModelRef,
        n_layers: usize,
        n_kv_heads: usize,
        head_dim: usize,
        max_seq_len: usize,
    ) -> Result<Self> {
        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        // Use scoped handle access to prevent ownership issues
        let mut state_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result =
            super::registry::registry::with_model_handle(*model_ref, |model_handle| unsafe {
                agentbridge_kv_state_create(
                    model_handle.as_ptr() as u64,
                    n_layers as i32,
                    n_kv_heads as i32,
                    head_dim as i32,
                    max_seq_len as i32,
                    &mut state_ref,
                    &mut error_ptr,
                )
            })
            .ok_or_else(|| ANEError::Internal("Model not found in registry".to_string()))?;

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error during KV state creation".to_string()
            };
            return Err(ANEError::Internal(error_msg));
        }

        Ok(KvStateHandle::new(state_ref))
    }

    /// Destroy the KV cache state
    pub fn destroy(self) -> Result<()> {
        if coreml_runtime_available() {
            let result = unsafe { agentbridge_kv_state_destroy(self.handle()) };
            if result != 0 {
                return Err(ANEError::Internal("Failed to destroy KV state".to_string()));
            }
        }
        Ok(())
    }

    /// Advance the KV cache state by one token
    pub fn step(&self) -> Result<()> {
        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe { agentbridge_kv_state_step(self.handle(), &mut error_ptr) };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error during KV state step".to_string()
            };
            return Err(ANEError::Internal(error_msg));
        }

        Ok(())
    }

    /// Reset the KV cache state
    pub fn reset(&self) -> Result<()> {
        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe { agentbridge_kv_state_reset(self.handle(), &mut error_ptr) };

        if result != 0 {
            let error_msg = if !error_ptr.is_null() {
                unsafe {
                    let cstr = std::ffi::CStr::from_ptr(error_ptr);
                    let msg = cstr.to_string_lossy().to_string();
                    agentbridge_free_string(error_ptr);
                    msg
                }
            } else {
                "Unknown error during KV state reset".to_string()
            };
            return Err(ANEError::Internal(error_msg));
        }

        Ok(())
    }
}

/// Extension to MLModel for KV-aware inference
impl super::types::MLModel {
    /// Run inference with KV cache state
    pub fn run_inference_with_kv(
        &self,
        input_provider: &super::types::MLFeatureProvider,
        kv_state: &KvStateHandle,
    ) -> Result<super::types::MLFeatureProvider> {
        if !coreml_runtime_available() {
            return Err(coreml_unavailable_error());
        }

        let mut output_provider_ref: u64 = 0;
        let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

        let result = unsafe {
            agentbridge_model_run_inference_with_kv(
                self.handle(),
                input_provider.ptr() as u64,
                kv_state.handle(),
                &mut output_provider_ref,
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
                "Unknown error during inference with KV cache".to_string()
            };
            return Err(ANEError::Internal(error_msg));
        }

        // Reconstruct the output provider from the reference
        // Create MLFeatureProvider from the reference
        let ptr = std::ptr::NonNull::new(output_provider_ref as *mut std::ffi::c_void)
            .ok_or_else(|| ANEError::Internal("Invalid output provider reference".to_string()))?;

        Ok(MLFeatureProvider::new(ptr))
    }
}
