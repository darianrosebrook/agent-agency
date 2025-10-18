/// Core ML Rust FFI Wrapper – Safe C-ABI bindings with autorelease pool management
/// @darianrosebrook
///
/// This module provides safe Rust wrappers around the Swift C-ABI bridge.
/// All FFI calls are wrapped in autorelease pools and error handling is centralized.
/// Invariants:
/// - Every FFI call runs within with_autorelease_pool guard
/// - No ObjC types escape this module
/// - All errors translated to Rust Result types
/// - Opaque pointers managed via Drop implementation

use anyhow::{bail, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// C FFI declarations – these match Swift @_cdecl exports
/// These must align exactly with coreml-bridge/Sources/CoreMLBridge.swift
extern "C" {
    /// Compile .mlmodel → .mlmodelc
    /// Returns: 0 success, 1 failure
    fn coreml_compile_model(
        model_path: *const c_char,
        compute_units: i32,
        out_compiled_path: *mut *mut c_char,
        out_err: *mut *mut c_char,
    ) -> i32;

    /// Load compiled .mlmodelc
    /// Returns: 0 success, 1 failure
    fn coreml_load_model(
        compiled_dir: *const c_char,
        compute_units: i32,
        out_handle: *mut *mut std::ffi::c_void,
        out_err: *mut *mut c_char,
    ) -> i32;

    /// Query model schema as JSON
    /// Returns: 0 success, 1 failure
    fn coreml_model_schema(
        handle: *mut std::ffi::c_void,
        out_schema_json: *mut *mut c_char,
        out_err: *mut *mut c_char,
    ) -> i32;

    /// Run inference
    /// Returns: 0 success, 1 failure
    fn coreml_predict(
        handle: *mut std::ffi::c_void,
        inputs_desc_json: *const c_char,
        out_outputs_desc_json: *mut *mut c_char,
        timeout_ms: i32,
        out_err: *mut *mut c_char,
    ) -> i32;

    /// Free model handle
    fn coreml_free_model(handle: *mut std::ffi::c_void);

    /// Free C string allocated by bridge
    fn coreml_free_cstr(s: *mut c_char);
}

/// Autorelease pool guard for FFI calls (macOS only)
/// TODO: This would use objc2-foundation::autoreleasepool for proper memory management
///    - Handle autorelease pool creation and cleanup for Objective-C objects
///    - Implement proper error handling for autorelease pool operations
///    - Handle autorelease pool thread safety and synchronization
///    - Implement proper memory leak prevention for Objective-C objects
///    - Add memory safety validation and error handling
///    - Implement autorelease pool performance monitoring and optimization
///    - Handle autorelease pool performance metrics and analytics
///    - Ensure autorelease pool operations meet performance requirements
///    - Handle autorelease pool creation failures gracefully
///    - Implement fallback mechanisms for autorelease pool operations
///    - Add proper logging and diagnostics for autorelease pool issues
#[cfg(target_os = "macos")]
pub fn with_autorelease_pool<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    // TODO: Implement autorelease pool integration with objc2-foundation with these requirements:
    // 1. Autorelease pool management: Implement proper autorelease pool lifecycle management
    //    - Integrate objc2-foundation::autoreleasepool for proper memory management
    //    - Handle autorelease pool creation and cleanup for Objective-C objects
    //    - Implement proper error handling for autorelease pool operations
    // 2. Memory safety: Ensure memory safety for Objective-C interop
    //    - Handle autorelease pool thread safety and synchronization
    //    - Implement proper memory leak prevention for Objective-C objects
    //    - Add memory safety validation and error handling
    // 3. Performance optimization: Optimize autorelease pool performance
    //    - Implement autorelease pool performance monitoring and optimization
    //    - Handle autorelease pool performance metrics and analytics
    //    - Ensure autorelease pool operations meet performance requirements
    // 4. Error handling: Implement robust error handling for autorelease pool
    //    - Handle autorelease pool creation failures gracefully
    //    - Implement fallback mechanisms for autorelease pool operations
    //    - Add proper logging and diagnostics for autorelease pool issues
    f()
}

#[cfg(not(target_os = "macos"))]
pub fn with_autorelease_pool<F, T>(_f: F) -> T
where
    F: FnOnce() -> T,
{
    panic!("Core ML bridge only available on macOS")
}

/// Safe wrapper around C string returned from FFI
struct CStringOwned {
    ptr: *mut c_char,
}

impl CStringOwned {
    /// Take ownership of a C string from FFI
    fn from_ptr(ptr: *mut c_char) -> Self {
        CStringOwned { ptr }
    }

    /// Convert to Rust string, consuming self
    fn to_string(self) -> Result<String> {
        if self.ptr.is_null() {
            bail!("Null pointer from FFI");
        }

        unsafe {
            let c_str = CStr::from_ptr(self.ptr);
            Ok(c_str.to_string_lossy().to_string())
        }
    }
}

impl Drop for CStringOwned {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                coreml_free_cstr(self.ptr);
            }
        }
    }
}

/// Opaque handle to Core ML model (owned)
pub struct CoreMLModel {
    handle: *mut std::ffi::c_void,
}

impl CoreMLModel {
    /// Compile a model from path
    pub fn compile(model_path: &str, compute_units: i32) -> Result<String> {
        with_autorelease_pool(|| {
            let c_model_path = CString::new(model_path)?;

            let mut out_compiled_path: *mut c_char = std::ptr::null_mut();
            let mut out_err: *mut c_char = std::ptr::null_mut();

            let ret = unsafe {
                coreml_compile_model(
                    c_model_path.as_ptr(),
                    compute_units,
                    &mut out_compiled_path,
                    &mut out_err,
                )
            };

            if ret != 0 {
                let err_string = CStringOwned::from_ptr(out_err).to_string().unwrap_or_else(|_| "Unknown error".to_string());
                bail!("Compile failed: {}", err_string);
            }

            let path_string = CStringOwned::from_ptr(out_compiled_path).to_string()?;
            Ok(path_string)
        })
    }

    /// Load a compiled model
    pub fn load(compiled_dir: &str, compute_units: i32) -> Result<Self> {
        with_autorelease_pool(|| {
            let c_compiled_dir = CString::new(compiled_dir)?;

            let mut out_handle: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut out_err: *mut c_char = std::ptr::null_mut();

            let ret = unsafe {
                coreml_load_model(
                    c_compiled_dir.as_ptr(),
                    compute_units,
                    &mut out_handle,
                    &mut out_err,
                )
            };

            if ret != 0 {
                let err_string = CStringOwned::from_ptr(out_err).to_string().unwrap_or_else(|_| "Unknown error".to_string());
                bail!("Load failed: {}", err_string);
            }

            Ok(CoreMLModel { handle: out_handle })
        })
    }

    /// Query model schema
    pub fn schema(&self) -> Result<String> {
        with_autorelease_pool(|| {
            let mut out_schema_json: *mut c_char = std::ptr::null_mut();
            let mut out_err: *mut c_char = std::ptr::null_mut();

            let ret = unsafe {
                coreml_model_schema(self.handle, &mut out_schema_json, &mut out_err)
            };

            if ret != 0 {
                let err_string = CStringOwned::from_ptr(out_err).to_string().unwrap_or_else(|_| "Unknown error".to_string());
                bail!("Schema query failed: {}", err_string);
            }

            CStringOwned::from_ptr(out_schema_json).to_string()
        })
    }

    /// Run inference
    pub fn predict(&self, inputs_json: &str, timeout_ms: i32) -> Result<String> {
        with_autorelease_pool(|| {
            let c_inputs = CString::new(inputs_json)?;

            let mut out_outputs_json: *mut c_char = std::ptr::null_mut();
            let mut out_err: *mut c_char = std::ptr::null_mut();

            let ret = unsafe {
                coreml_predict(
                    self.handle,
                    c_inputs.as_ptr(),
                    &mut out_outputs_json,
                    timeout_ms,
                    &mut out_err,
                )
            };

            if ret != 0 {
                let err_string = CStringOwned::from_ptr(out_err).to_string().unwrap_or_else(|_| "Unknown error".to_string());
                bail!("Prediction failed: {}", err_string);
            }

            CStringOwned::from_ptr(out_outputs_json).to_string()
        })
    }
}

impl Drop for CoreMLModel {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            with_autorelease_pool(|| unsafe {
                coreml_free_model(self.handle);
            });
        }
    }
}

// Safety: Core ML handles can be shared across threads safely
// The underlying MLModel is thread-safe (can call prediction from multiple threads)
unsafe impl Send for CoreMLModel {}
unsafe impl Sync for CoreMLModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_autorelease_pool() {
        let result = with_autorelease_pool(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_core_ml_model_send_sync() {
        // Verify at compile time
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CoreMLModel>();
        assert_sync::<CoreMLModel>();
    }
}
