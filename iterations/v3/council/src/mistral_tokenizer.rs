//! Mistral Tokenizer Bridge
//!
//! Provides Rust interface to Swift Mistral tokenizer implementation
//! Handles text tokenization and detokenization for Mistral-7B models

use std::ffi::{c_char, c_int, c_uint, CStr, CString};
use std::ptr;

/// Mistral tokenizer handle (opaque pointer from Swift)
pub type TokenizerHandle = *mut std::ffi::c_void;

/// Mistral tokenizer for text processing
#[derive(Debug)]
pub struct MistralTokenizer {
    handle: TokenizerHandle,
}

// Mark as Send + Sync (Swift handles thread safety)
unsafe impl Send for MistralTokenizer {}
unsafe impl Sync for MistralTokenizer {}

impl MistralTokenizer {
    /// Create a new Mistral tokenizer
    pub fn new() -> Result<Self, String> {
        unsafe {
            let handle = mistral_tokenizer_create();
            if handle.is_null() {
                return Err("Failed to create Mistral tokenizer".to_string());
            }
            Ok(Self { handle })
        }
    }

    /// Encode text into token IDs
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, String> {
        unsafe {
            let c_text = match CString::new(text) {
                Ok(s) => s,
                Err(_) => return Err("Text contains null bytes".to_string()),
            };

            let mut tokens_ptr: *mut u32 = ptr::null_mut();
            let mut token_count: c_uint = 0;
            let mut error_ptr: *mut c_char = ptr::null_mut();

            let result = mistral_tokenizer_encode(
                self.handle,
                c_text.as_ptr(),
                &mut tokens_ptr,
                &mut token_count,
                &mut error_ptr,
            );

            if result != 0 {
                let error_msg = if error_ptr.is_null() {
                    "Unknown encoding error".to_string()
                } else {
                    CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
                };
                mistral_tokenizer_free_string(error_ptr);
                return Err(error_msg);
            }

            if tokens_ptr.is_null() {
                return Err("Tokenizer returned null token array".to_string());
            }

            // Convert to Vec<u32>
            let tokens = std::slice::from_raw_parts(tokens_ptr, token_count as usize)
                .to_vec();

            // Free the allocated memory
            mistral_tokenizer_free_tokens(tokens_ptr);

            Ok(tokens)
        }
    }

    /// Decode token IDs back to text
    pub fn decode(&self, tokens: &[u32]) -> Result<String, String> {
        unsafe {
            let mut text_ptr: *mut c_char = ptr::null_mut();
            let mut error_ptr: *mut c_char = ptr::null_mut();

            let result = mistral_tokenizer_decode(
                self.handle,
                tokens.as_ptr(),
                tokens.len() as c_uint,
                &mut text_ptr,
                &mut error_ptr,
            );

            if result != 0 {
                let error_msg = if error_ptr.is_null() {
                    "Unknown decoding error".to_string()
                } else {
                    CStr::from_ptr(error_ptr).to_string_lossy().into_owned()
                };
                mistral_tokenizer_free_string(error_ptr);
                return Err(error_msg);
            }

            if text_ptr.is_null() {
                return Err("Tokenizer returned null text".to_string());
            }

            let text = CStr::from_ptr(text_ptr).to_string_lossy().into_owned();

            // Free the allocated memory
            mistral_tokenizer_free_string(text_ptr);

            Ok(text)
        }
    }

    /// Get tokenizer vocabulary information
    pub fn vocab_size(&self) -> usize {
        32000 // Mistral vocabulary size
    }

    /// Get maximum sequence length
    pub fn max_sequence_length(&self) -> usize {
        4096 // Mistral context window
    }
}

impl Drop for MistralTokenizer {
    fn drop(&mut self) {
        unsafe {
            mistral_tokenizer_destroy(self.handle);
        }
    }
}

impl Default for MistralTokenizer {
    fn default() -> Self {
        Self::new().expect("Failed to create default Mistral tokenizer")
    }
}

/// FFI declarations for Mistral tokenizer bridge
extern "C" {
    fn mistral_tokenizer_create() -> TokenizerHandle;
    fn mistral_tokenizer_destroy(handle: TokenizerHandle);

    fn mistral_tokenizer_encode(
        handle: TokenizerHandle,
        text: *const c_char,
        out_tokens: *mut *mut u32,
        out_token_count: *mut c_uint,
        out_error: *mut *mut c_char,
    ) -> c_int;

    fn mistral_tokenizer_decode(
        handle: TokenizerHandle,
        tokens: *const u32,
        token_count: c_uint,
        out_text: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> c_int;

    fn mistral_tokenizer_free_string(ptr: *mut c_char);
    fn mistral_tokenizer_free_tokens(tokens: *mut u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = MistralTokenizer::new();
        assert!(tokenizer.is_ok(), "Failed to create tokenizer");
    }

    #[test]
    fn test_simple_encode_decode() {
        let tokenizer = MistralTokenizer::new().unwrap();

        let test_text = "Hello world";
        let tokens = tokenizer.encode(test_text).unwrap();

        assert!(!tokens.is_empty(), "Should produce tokens");
        assert_eq!(tokens[0], 1, "Should start with BOS token");
        assert_eq!(tokens.last().unwrap(), &2, "Should end with EOS token");

        let decoded = tokenizer.decode(&tokens).unwrap();
        assert!(!decoded.is_empty(), "Should produce decoded text");
    }

    #[test]
    fn test_vocab_size() {
        let tokenizer = MistralTokenizer::new().unwrap();
        assert_eq!(tokenizer.vocab_size(), 32000);
    }

    #[test]
    fn test_max_sequence_length() {
        let tokenizer = MistralTokenizer::new().unwrap();
        assert_eq!(tokenizer.max_sequence_length(), 4096);
    }

    #[test]
    fn test_empty_text() {
        let tokenizer = MistralTokenizer::new().unwrap();
        let tokens = tokenizer.encode("").unwrap();
        assert_eq!(tokens.len(), 2, "Empty text should have BOS and EOS tokens");
    }
}
