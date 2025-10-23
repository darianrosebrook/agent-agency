//! Core ML compatibility layer for ANE operations
//!
//! This module provides a safe interface to Core ML framework functionality
//! for Apple Neural Engine operations, avoiding direct private framework usage.

use crate::ane::errors::{ANEError, Result};
use crate::inference::{DType, TensorSpec};
use std::path::Path;
use std::marker::PhantomData;
use std::ptr::NonNull;

// Removed unused import: objc2::rc::Retained
// TODO: Fix objc2 imports when Core ML integration is implemented
// #[cfg(target_os = "macos")]
// use objc2_core_ml::{MLModel, MLMultiArray, MLPredictionOptions};
// #[cfg(target_os = "macos")]
// use objc2_foundation::{NSDictionary, NSError, NSString, NSURL};
// Removed unused import: std::ffi::c_void

/// Target platform detection
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const TARGET_APPLE_SILICON: bool = true;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
const TARGET_APPLE_SILICON: bool = false;

/// Core ML framework interface
pub mod coreml {
    use super::*;

    /// Check if ANE is available on this system
    pub fn is_ane_available() -> bool {
        TARGET_APPLE_SILICON
    }

    /// Get Core ML driver version (if available)
    pub fn driver_version() -> Option<String> {
        None
    }

    /// Compile a .mlmodel file to .mlmodelc format
    pub fn compile_model(source_path: &Path) -> Result<std::path::PathBuf> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Internal("Core ML not available on this platform"));
        }
        
        // TODO: Implement actual Core ML compilation
        // This would require objc2 bindings to MLModel.compileModelAtURL:error:
        let compiled_path = source_path.with_extension("mlmodelc");
        Ok(compiled_path)
    }

    /// Load a compiled Core ML model and return an opaque reference
    /// The raw handle is stored in a thread-local registry for safety
    pub fn load_model(path: &str) -> Result<ModelRef> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Internal("Core ML not available on this platform"));
        }

        #[cfg(target_os = "macos")]
        {
            // TODO: Implement actual Core ML model loading
            // For now, create a placeholder handle and register it
            let raw_handle = Box::into_raw(Box::new(42u32)) as *mut std::ffi::c_void;

            // Wrap in thread-confined handle
            let handle = CoreMlHandle::new(raw_handle)
                .ok_or_else(|| ANEError::Internal("Failed to create model handle"))?;

            // Register and get opaque reference
            let model_ref = registry::register_model(handle);
            Ok(model_ref)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(ANEError::Internal("Core ML not available on this platform"))
        }
    }

    /// Thread-confined CoreML handle that cannot be sent or shared between threads.
    /// This prevents Send/Sync violations when raw pointers are captured in async contexts.
    pub struct CoreMlHandle {
        ptr: NonNull<std::ffi::c_void>,
        // Ensures !Send + !Sync without unsafe impls
        _no_send_sync: PhantomData<*mut ()>,
    }

    impl CoreMlHandle {
        /// Create a new handle from a raw pointer.
        /// Returns None if the pointer is null.
        pub fn new(ptr: *mut std::ffi::c_void) -> Option<Self> {
            NonNull::new(ptr).map(|nn| Self {
                ptr: nn,
                _no_send_sync: PhantomData,
            })
        }

        /// Get the raw pointer for FFI calls.
        /// This should only be called on the thread that owns the handle.
        pub fn as_ptr(&self) -> *mut std::ffi::c_void {
            self.ptr.as_ptr()
        }
    }

    impl Drop for CoreMlHandle {
        fn drop(&mut self) {
            // TODO: Call appropriate CoreML release function if needed
            // This would typically call a release function from the CoreML bridge
            tracing::debug!("Dropping CoreMlHandle");
        }
    }

    /// Opaque model reference that replaces raw pointers in public APIs.
    /// This can be safely sent across threads and mapped back to raw handles
    /// in thread-local registries.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModelRef(u64);

    impl ModelRef {
        /// Create a new unique model reference
        pub fn new() -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT_ID: AtomicU64 = AtomicU64::new(1);
            Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
        }
    }

    /// Thread-local registry mapping ModelRef to CoreMlHandle
    /// This should only be used on the thread that owns the CoreML handles.
    pub struct ModelRegistry {
        models: std::collections::HashMap<ModelRef, CoreMlHandle>,
    }

    impl ModelRegistry {
        /// Create a new empty registry
        pub fn new() -> Self {
            Self {
                models: std::collections::HashMap::new(),
            }
        }

        /// Register a model handle and return an opaque reference
        pub fn register(&mut self, handle: CoreMlHandle) -> ModelRef {
            let id = ModelRef::new();
            self.models.insert(id, handle);
            id
        }

        /// Get the raw handle for a model reference
        /// Returns None if the reference is not registered on this thread
        pub fn get_handle(&self, id: ModelRef) -> Option<&CoreMlHandle> {
            self.models.get(&id)
        }

        /// Remove a model from the registry (called during cleanup)
        pub fn unregister(&mut self, id: ModelRef) -> Option<CoreMlHandle> {
            self.models.remove(&id)
        }
    }

    /// Thread-local storage for model registries
    thread_local! {
        static MODEL_REGISTRY: std::cell::RefCell<ModelRegistry> = std::cell::RefCell::new(ModelRegistry::new());
    }

    /// Thread-safe operations on the thread-local registry
    pub mod registry {
        use super::*;

        /// Register a model handle and get an opaque reference
        /// This should only be called on the thread that owns the handle
        pub fn register_model(handle: CoreMlHandle) -> ModelRef {
            MODEL_REGISTRY.with(|registry| {
                registry.borrow_mut().register(handle)
            })
        }

        /// Get the raw handle for a model reference
        /// Returns None if called on wrong thread or reference doesn't exist
        pub fn get_model_handle(id: ModelRef) -> Option<std::ptr::NonNull<std::ffi::c_void>> {
            MODEL_REGISTRY.with(|registry| {
                registry.borrow().get_handle(id).map(|h| h.ptr)
            })
        }

        /// Unregister a model (called during cleanup)
        /// Returns the handle for proper cleanup
        pub fn unregister_model(id: ModelRef) -> Option<CoreMlHandle> {
            MODEL_REGISTRY.with(|registry| {
                registry.borrow_mut().unregister(id)
            })
        }
    }

    /// I/O safety validation helpers
    pub mod io_safety {
        use super::*;

        /// Convert FFI tensor data to owned Vec<f32>, validating shape and bounds
        pub fn into_owned_tensor(data: &[f32], shape: &[usize]) -> Result<Tensor> {
            // Validate shape is not empty and compute total size
            if shape.is_empty() {
                return Err(ANEError::InvalidInput("Tensor shape cannot be empty".to_string()));
            }

            let total_size: usize = shape.iter().product();
            if total_size == 0 {
                return Err(ANEError::InvalidInput("Tensor cannot have zero size".to_string()));
            }

            // Check data length matches shape
            if data.len() != total_size {
                return Err(ANEError::InvalidInput(
                    format!("Data length {} doesn't match shape product {}", data.len(), total_size)
                ));
            }

            // Reasonable size limits to prevent memory exhaustion
            const MAX_TENSOR_ELEMENTS: usize = 100 * 1024 * 1024; // 100M elements
            if total_size > MAX_TENSOR_ELEMENTS {
                return Err(ANEError::InvalidInput(
                    format!("Tensor too large: {} elements (max {})", total_size, MAX_TENSOR_ELEMENTS)
                ));
            }

            Ok(Tensor::new(data, shape)?)
        }

        /// Validate tensor schema matches expected I/O specification
        pub fn validate_io_schema(tensor: &Tensor, expected_spec: &TensorSpec) -> Result<()> {
            // Check data type (for now we only support f32)
            if expected_spec.dtype != DType::F32 {
                return Err(ANEError::InvalidInput(
                    format!("Unsupported dtype: {:?}, expected F32", expected_spec.dtype)
                ));
            }

            // Check shape compatibility
            if tensor.shape.len() != expected_spec.shape.len() {
                return Err(ANEError::InvalidInput(
                    format!("Shape dimension mismatch: got {}, expected {}",
                           tensor.shape.len(), expected_spec.shape.len())
                ));
            }

            // For batch-capable tensors, allow variable batch size
            if expected_spec.batch_capable && tensor.shape.len() > 0 {
                // Check non-batch dimensions match
                if &tensor.shape[1..] != &expected_spec.shape[1..] {
                    return Err(ANEError::InvalidInput(
                        format!("Non-batch dimensions don't match: got {:?}, expected {:?}",
                               &tensor.shape[1..], &expected_spec.shape[1..])
                    ));
                }
            } else {
                // Exact shape match required
                if tensor.shape != expected_spec.shape {
                    return Err(ANEError::InvalidInput(
                        format!("Shape mismatch: got {:?}, expected {:?}", tensor.shape, expected_spec.shape)
                    ));
                }
            }

            Ok(())
        }

        /// Safe conversion from raw FFI tensors to owned tensors
        /// This prevents buffer overflows and validates all inputs
        pub fn convert_ffi_tensors(raw_tensors: Vec<super::Tensor>) -> Result<Vec<Tensor>> {
            let mut owned_tensors = Vec::with_capacity(raw_tensors.len());

            for raw_tensor in raw_tensors {
                // Validate and convert each tensor
                let owned = into_owned_tensor(&raw_tensor.data, &raw_tensor.shape)?;
                owned_tensors.push(owned);
            }

            Ok(owned_tensors)
        }
    }

    /// Tensor type
    pub struct Tensor {
        pub data: Vec<f32>,
        pub shape: Vec<usize>,
    }

    impl Tensor {
        pub fn new(data: &[f32], shape: &[usize]) -> Result<Self> {
            Ok(Tensor {
                data: data.to_vec(),
                shape: shape.to_vec(),
            })
        }
    }

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
            ane_available: TARGET_APPLE_SILICON,
            supported_precisions: if TARGET_APPLE_SILICON {
                vec!["FP16".to_string(), "FP32".to_string()]
            } else {
                vec![]
            },
        }
    }

    /// Run inference on a loaded model using opaque reference
    pub fn run_inference(
        model_ref: ModelRef,
        _input_name: &str,
        input_data: &[f32],
        input_shape: &[i32],
    ) -> Result<Tensor> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Internal("Core ML not available on this platform"));
        }

        // Get the thread-confined handle from registry
        let handle = registry::get_model_handle(model_ref)
            .ok_or_else(|| ANEError::InvalidInput("Model reference not found or called from wrong thread".to_string()))?;

        #[cfg(target_os = "macos")]
        {
            // TODO: Implement actual Core ML inference using handle.as_ptr()
            // For now, return a placeholder tensor with the expected output shape
            let output_size = (input_shape.iter().product::<i32>() * 4) as usize; // Rough estimation
            let output_data = vec![0.0f32; output_size.max(1280 * 1500)]; // Placeholder

            Ok(Tensor::new(&output_data, &[1, 1280, 1500])?)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(ANEError::Internal("Core ML not available on this platform"))
        }
    }

    /// Unload a model and free resources using opaque reference
    pub fn unload_model(model_ref: ModelRef) {
        // Unregister from thread-local registry - this will drop the CoreMlHandle
        // and trigger proper cleanup
        let _handle = registry::unregister_model(model_ref);
        // Handle is dropped here, which calls the Drop impl for CoreMlHandle
    }

    // ============================================================================
    // FFI Declarations for BridgesFFI
    // ============================================================================

    #[cfg_attr(target_os = "macos", link(name = "BridgesFFI", kind = "framework"))]
    extern "C" {
        // Core functions
        pub fn agentbridge_init() -> i32;
        pub fn agentbridge_shutdown() -> i32;
        pub fn agentbridge_get_version(out_version: *mut *mut std::ffi::c_char) -> i32;
        pub fn agentbridge_free_string(ptr: *mut std::ffi::c_char);

        // Model management
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

        // Text processing - Mistral
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

        // Audio processing - Whisper
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

        // Audio processing - Speech Framework
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

        // Vision processing - YOLO
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

        // Vision processing - OCR
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

        // Text generation - Diffusion
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

        // System monitoring
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

    /// Mistral tokenizer functions (wrappers around FFI)
    pub fn mistral_tokenizer_create() -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }

    pub fn mistral_encode(_tokenizer: *mut std::ffi::c_void, _text: &str) -> Result<*mut i32> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Internal("Core ML not available on this platform"));
        }
        Ok(std::ptr::null_mut())
    }

    pub fn mistral_free_tokens(_tokens: *mut i32) {
        // No-op
    }

    pub fn mistral_decode(_tokenizer: *mut std::ffi::c_void, _tokens: &[i32]) -> Result<*mut std::ffi::c_char> {
        if !TARGET_APPLE_SILICON {
            return Err(ANEError::Internal("Core ML not available on this platform"));
        }
        Ok(std::ptr::null_mut())
    }

    pub fn mistral_free_string(_text: *mut std::ffi::c_char) {
        // No-op
    }

    pub fn mistral_get_vocab_size(_tokenizer: *mut std::ffi::c_void) -> usize {
        0
    }

    pub fn mistral_tokenizer_destroy(_tokenizer: *mut std::ffi::c_void) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        assert_eq!(TARGET_APPLE_SILICON, cfg!(all(target_os = "macos", target_arch = "aarch64")));
    }

    #[test]
    fn test_ane_availability() {
        let available = coreml::is_ane_available();
        assert_eq!(available, TARGET_APPLE_SILICON);
    }
}