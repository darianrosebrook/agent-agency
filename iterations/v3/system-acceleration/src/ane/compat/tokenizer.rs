//! Mistral tokenizer implementation
//!
//! This module provides high-level text tokenization functions for Mistral models,
//! along with legacy FFI-style wrappers for backward compatibility.

use crate::ane::ane_errors::{ANEError, Result};
use std::ffi::CString;

// Import the coreml module for FFI access
use super::coreml_module as coreml;

/// Runtime capability check for Core ML availability
///
/// Core ML requires:
/// - macOS operating system
/// - ARM64 architecture (Apple Silicon)
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
    ANEError::Internal(
        "Core ML unavailable: this process is not the arm64 macOS slice or the OS lacks Core ML mlprogram support. \
         Fix: build/run `aarch64-apple-darwin` (or ship a Universal2 and ensure arm64 launches). \
         Example: `cargo build --target aarch64-apple-darwin`."
            .to_string(),
    )
}

// FFI declarations needed for tokenizer operations
extern "C" {
    /// Optional Swift/ObjC shim (returning true iff CoreML APIs are usable on this OS)
    #[allow(dead_code)] // Will be used in v4
    fn coreml_can_load_models() -> bool;
}

// Import the coreml module for FFI access
// coreml module is re-exported from the main module

/// Encode text to tokens using Mistral tokenizer
pub fn mistral_encode(text: &str) -> Result<Vec<i32>> {
    if !coreml_runtime_available() {
        return Err(coreml_unavailable_error());
    }

    let text_cstr = CString::new(text)
        .map_err(|e| ANEError::InvalidInput(format!("Invalid text encoding: {}", e)))?;

    let mut tokens_ptr: *mut i32 = std::ptr::null_mut();
    let mut token_count: i32 = 0;
    let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

    let result = unsafe {
        coreml::agentbridge_text_mistral_encode(
            text_cstr.as_ptr(),
            &mut tokens_ptr,
            &mut token_count,
            &mut error_ptr,
        )
    };

    if result != 0 {
        let error_msg = if !error_ptr.is_null() {
            unsafe {
                let cstr = std::ffi::CStr::from_ptr(error_ptr);
                let msg = cstr.to_string_lossy().to_string();
                coreml::agentbridge_free_string(error_ptr);
                msg
            }
        } else {
            "Unknown error during Mistral encoding".to_string()
        };
        return Err(ANEError::Internal(error_msg));
    }

    if tokens_ptr.is_null() || token_count <= 0 {
        return Err(ANEError::Internal(
            "No tokens returned from encoding".to_string(),
        ));
    }

    let tokens = unsafe {
        let slice = std::slice::from_raw_parts(tokens_ptr, token_count as usize);
        let vec = slice.to_vec();
        // Free the allocated memory
        coreml::agentbridge_text_mistral_free_tokens(tokens_ptr, token_count);
        vec
    };

    Ok(tokens)
}

/// Decode tokens to text using Mistral tokenizer
pub fn mistral_decode(tokens: &[i32]) -> Result<String> {
    if !coreml_runtime_available() {
        return Err(coreml_unavailable_error());
    }

    if tokens.is_empty() {
        return Err(ANEError::InvalidInput(
            "Cannot decode empty token sequence".to_string(),
        ));
    }

    let mut text_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

    let result = unsafe {
        coreml::agentbridge_text_mistral_decode(
            tokens.as_ptr(),
            tokens.len() as i32,
            &mut text_ptr,
            &mut error_ptr,
        )
    };

    if result != 0 {
        let error_msg = if !error_ptr.is_null() {
            unsafe {
                let cstr = std::ffi::CStr::from_ptr(error_ptr);
                let msg = cstr.to_string_lossy().to_string();
                coreml::agentbridge_free_string(error_ptr);
                msg
            }
        } else {
            "Unknown error during Mistral decoding".to_string()
        };
        return Err(ANEError::Internal(error_msg));
    }

    if text_ptr.is_null() {
        return Err(ANEError::Internal(
            "No text returned from decoding".to_string(),
        ));
    }

    let text = unsafe {
        let cstr = std::ffi::CStr::from_ptr(text_ptr);
        let text_str = cstr.to_string_lossy().to_string();
        coreml::agentbridge_free_string(text_ptr);
        text_str
    };

    Ok(text)
}

/// Free memory allocated by agentbridge functions
///
/// # Safety
/// The caller must ensure `ptr` is a valid pointer returned by agentbridge functions or is null.
pub unsafe fn mistral_free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        coreml::agentbridge_free_string(ptr);
    }
}

/// Legacy FFI-style function aliases for backward compatibility
/// These delegate to the new high-level wrapper functions

/// Create a tokenizer handle (legacy FFI compatibility)
pub fn mistral_tokenizer_create() -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

/// Encode text using legacy FFI interface
///
/// # Safety
/// The caller must ensure `text` is a valid null-terminated C string or is null.
pub unsafe fn mistral_tokenizer_encode(
    _tokenizer: *mut std::ffi::c_void,
    text: *const std::ffi::c_char,
    tokens_out: &mut *mut i32,
    token_count_out: &mut i32,
    error_out: &mut *mut std::ffi::c_char,
) -> i32 {
    if !coreml_runtime_available() {
        return -1; // Error
    }

    if text.is_null() {
        *error_out = std::ffi::CString::new("Null text pointer")
            .unwrap()
            .into_raw();
        return -1;
    }

    let cstr = std::ffi::CStr::from_ptr(text);
    let text_str = match cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            *error_out = std::ffi::CString::new("Invalid UTF-8 text")
                .unwrap()
                .into_raw();
            return -1;
        }
    };

    match mistral_encode(text_str) {
        Ok(tokens) => {
            let token_count = tokens.len() as i32;
            *tokens_out = Box::into_raw(tokens.into_boxed_slice()) as *mut i32;
            *token_count_out = token_count;
            *error_out = std::ptr::null_mut();
            0 // Success
        }
        Err(e) => {
            let error_msg = format!("Encoding failed: {}", e);
            *error_out = std::ffi::CString::new(error_msg).unwrap().into_raw();
            -1 // Error
        }
    }
}

/// Free tokens allocated by mistral_tokenizer_encode
///
/// # Safety
/// The caller must ensure `tokens` is a valid pointer returned by `mistral_tokenizer_encode` or is null.
pub unsafe fn mistral_tokenizer_free_tokens(tokens: *mut i32) {
    if !tokens.is_null() {
        let _ = Box::from_raw(tokens);
    }
}

/// Decode tokens using legacy FFI interface
///
/// # Safety
/// The caller must ensure:
/// - `tokens` is a valid pointer to an array of at least `token_count` elements, or is null
/// - `text_out` and `error_out` are valid pointers to mutable pointers
pub unsafe fn mistral_tokenizer_decode(
    _tokenizer: *mut std::ffi::c_void,
    tokens: *const i32,
    token_count: i32,
    text_out: *mut *mut std::ffi::c_char,
    error_out: *mut *mut std::ffi::c_char,
) -> i32 {
    if !coreml_runtime_available() {
        return -1; // Error
    }

    if tokens.is_null() || token_count <= 0 {
        *error_out = std::ffi::CString::new("Invalid tokens").unwrap().into_raw();
        return -1;
    }

    let token_slice = std::slice::from_raw_parts(tokens, token_count as usize);

    match mistral_decode(token_slice) {
        Ok(text) => {
            *text_out = std::ffi::CString::new(text).unwrap().into_raw();
            *error_out = std::ptr::null_mut();
            0 // Success
        }
        Err(e) => {
            let error_msg = format!("Decoding failed: {}", e);
            *error_out = std::ffi::CString::new(error_msg).unwrap().into_raw();
            -1 // Error
        }
    }
}

/// Free text allocated by mistral_tokenizer_decode
///
/// # Safety
/// The caller must ensure `text` is a valid pointer returned by `mistral_tokenizer_decode` or is null.
pub unsafe fn mistral_tokenizer_free_text(text: *mut std::ffi::c_char) {
    if !text.is_null() {
        let _ = std::ffi::CString::from_raw(text);
    }
}

/// Destroy tokenizer handle (legacy FFI compatibility)
pub fn mistral_tokenizer_destroy(_tokenizer: *mut std::ffi::c_void) {
    // No-op - tokenizers are managed differently now
}
