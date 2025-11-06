//! Thread-local registry for Core ML model handle management
//!
//! This module provides thread-local storage and management for Core ML model handles,
//! ensuring safe access to model resources across different threads.
//!
//! ## Ownership Invariants
//!
//! 1. **Ownership**: The registry owns the MLModel; only `CoreMlHandle::drop` releases the model via `release_coreml_model`.
//! 2. **Threading**: A handle is confined to the registering thread; debug builds assert on cross-thread access.
//! 3. **Lifetimes**: User code only receives borrowed access via `with_*` scoped closures; no long-lived owned wrappers.
//! 4. **Unregister**: Only call when no concurrent `with_*` is active on the same thread. With `Rc`, this is enforced (drop occurs when the last `Rc` is released).

use std::collections::HashMap;
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::rc::Rc;

/// Types of Core ML objects that can be managed by the registry.
/// This determines which destroy function to call during cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreMlObjectType {
    /// MLModel compiled model
    Model,
    /// MLMultiArray data array
    Array,
    /// MLFeatureProvider base provider
    FeatureProvider,
    /// MLDictionaryFeatureProvider dictionary provider
    DictionaryProvider,
    /// KV cache state for stateful inference
    KvState,
}

/// Opaque newtype for Core ML object pointers with type information.
/// This wraps raw pointers from FFI calls and ensures they can only be released through proper channels.
#[derive(Debug)]
pub struct RawCoreMlObject {
    ptr: NonNull<std::ffi::c_void>,
    object_type: CoreMlObjectType,
}

impl RawCoreMlObject {
    /// Create from a NonNull pointer with type information (unsafe: caller must ensure pointer is valid)
    pub unsafe fn new(p: NonNull<std::ffi::c_void>, object_type: CoreMlObjectType) -> Self {
        Self {
            ptr: p,
            object_type,
        }
    }

    /// Create a model object (convenience method)
    pub unsafe fn model(p: NonNull<std::ffi::c_void>) -> Self {
        Self::new(p, CoreMlObjectType::Model)
    }

    /// Get the raw pointer for FFI calls
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr.as_ptr()
    }

    /// Get the object type
    pub fn object_type(&self) -> CoreMlObjectType {
        self.object_type
    }
}

/// Concrete FFI release function that owns the canonical destroy path.
/// Only `CoreMlHandle::drop` may call this to prevent double-free.
/// Calls the appropriate destroy function based on the object type.
unsafe fn release_coreml_object(object: &RawCoreMlObject) {
    // Declare all FFI destroy functions
    extern "C" {
        fn agentbridge_model_destroy(model_ref: u64) -> i32;
        fn agentbridge_array_destroy(array_ref: u64) -> i32;
        fn agentbridge_provider_destroy(provider_ref: u64) -> i32;
        fn agentbridge_dict_provider_destroy(provider_ref: u64) -> i32;
        fn agentbridge_kv_state_destroy(state_ref: u64) -> i32;
    }

    // Import runtime check from model module
    use super::model;

    if !model::coreml_runtime_available() {
        tracing::debug!("Core ML runtime not available, skipping object release");
        return;
    }

    let handle = object.as_ptr() as u64;
    let result = match object.object_type() {
        CoreMlObjectType::Model => {
            let result = agentbridge_model_destroy(handle);
            if result == 0 {
                tracing::debug!("Successfully released Core ML model handle {}", handle);
            }
            result
        }
        CoreMlObjectType::Array => {
            let result = agentbridge_array_destroy(handle);
            if result == 0 {
                tracing::debug!("Successfully released Core ML array handle {}", handle);
            }
            result
        }
        CoreMlObjectType::FeatureProvider => {
            let result = agentbridge_provider_destroy(handle);
            if result == 0 {
                tracing::debug!("Successfully released Core ML provider handle {}", handle);
            }
            result
        }
        CoreMlObjectType::DictionaryProvider => {
            let result = agentbridge_dict_provider_destroy(handle);
            if result == 0 {
                tracing::debug!("Successfully released Core ML dict provider handle {}", handle);
            }
            result
        }
        CoreMlObjectType::KvState => {
            let result = agentbridge_kv_state_destroy(handle);
            if result == 0 {
                tracing::debug!("Successfully released Core ML KV state handle {}", handle);
            }
            result
        }
    };

    if result != 0 {
        tracing::warn!("Failed to destroy Core ML {:?} handle {}", object.object_type(), handle);
    }
}

/// Thread-confined CoreML handle that cannot be sent or shared between threads.
/// This prevents Send/Sync violations when raw pointers are captured in async contexts.
#[derive(Debug)]
pub struct CoreMlHandle {
    object: RawCoreMlObject,
    owner: std::thread::ThreadId,
    // Ensures !Send + !Sync without unsafe impls
    _no_send_sync: PhantomData<*mut ()>,
}

impl CoreMlHandle {
    /// Create a new handle from a raw pointer for a model object.
    /// Returns None if the pointer is null.
    /// This is the primary constructor for backward compatibility.
    pub fn new(ptr: *mut std::ffi::c_void) -> Option<Self> {
        Self::with_type(ptr, CoreMlObjectType::Model)
    }

    /// Create a new handle from a raw pointer with explicit object type.
    /// Returns None if the pointer is null.
    pub fn with_type(ptr: *mut std::ffi::c_void, object_type: CoreMlObjectType) -> Option<Self> {
        NonNull::new(ptr).map(|nn| Self {
            object: unsafe { RawCoreMlObject::new(nn, object_type) },
            owner: std::thread::current().id(),
            _no_send_sync: PhantomData,
        })
    }

    /// Assert that we're being accessed on the owning thread.
    /// Panics in debug builds if accessed from wrong thread.
    #[inline]
    fn assert_owner(&self) {
        debug_assert_eq!(
            self.owner,
            std::thread::current().id(),
            "CoreMlHandle accessed on non-owning thread {:?} != {:?}",
            std::thread::current().id(),
            self.owner
        );
    }

    /// Get the raw pointer for FFI calls.
    /// This should only be called on the thread that owns the handle.
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.assert_owner();
        self.object.as_ptr()
    }

    /// Get the object type of this handle.
    pub fn object_type(&self) -> CoreMlObjectType {
        self.object.object_type()
    }
}

impl Drop for CoreMlHandle {
    fn drop(&mut self) {
        // Call the canonical release function - only this path may release the object
        unsafe {
            release_coreml_object(&self.object);
        }
        tracing::debug!("CoreMlHandle dropped and {:?} released", self.object.object_type());
    }
}

/// Opaque model reference that replaces raw pointers in public APIs.
/// This can be safely sent across threads and mapped back to raw handles
/// in thread-local registries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, schemars::JsonSchema)]
pub struct ModelRef(u64);

impl ModelRef {
    /// Create a new unique model reference
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the model ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl Default for ModelRef {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRef {
    /// Run a closure with access to the compiled MLModel
    /// This provides scoped access without creating owned wrappers
    pub fn with_compiled_model<F, T>(&self, f: F) -> crate::ane::ane_errors::Result<T>
    where
        F: FnOnce(std::ptr::NonNull<std::ffi::c_void>) -> T,
    {
        use crate::ane::ane_errors::ANEError;

        registry::with_model_handle(*self, f)
            .ok_or_else(|| ANEError::Internal("Model not found in registry".to_string()))
    }

    /// Save the compiled model to a file path using scoped access
    pub fn save_to_path(&self, path: &std::path::Path) -> crate::ane::ane_errors::Result<()> {
        self.with_compiled_model(|ptr| {
            // Use the existing save logic but with raw pointer
            save_model_to_path(ptr.as_ptr() as u64, path)
        })?
    }
}

/// Internal function to save a model using its raw handle
fn save_model_to_path(model_handle: u64, path: &std::path::Path) -> crate::ane::ane_errors::Result<()> {
    use std::ffi::CString;
    use crate::ane::ane_errors::ANEError;

    // Get model information from the FFI layer
    let mut info_ptr: *mut std::ffi::c_char = std::ptr::null_mut();
    let mut error_ptr: *mut std::ffi::c_char = std::ptr::null_mut();

    let info_result = unsafe {
        // We need to declare this locally since we can't import it
        extern "C" {
            fn agentbridge_model_get_info(
                model_ref: u64,
                out_info: *mut *mut std::ffi::c_char,
                out_error: *mut *mut std::ffi::c_char,
            ) -> i32;
        }
        agentbridge_model_get_info(
            model_handle,
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
        return Err(ANEError::Internal(format!("Failed to get model info: {}", error_msg)));
    }

    if info_ptr.is_null() {
        return Err(ANEError::Internal("No model info available".to_string()));
    }

    let model_info_json = unsafe {
        std::ffi::CStr::from_ptr(info_ptr)
            .to_string_lossy()
            .to_string()
    };

    // Free the info string
    unsafe {
        extern "C" {
            fn agentbridge_free_string(ptr: *mut std::ffi::c_char);
        }
        agentbridge_free_string(info_ptr);
    }

    // Parse the JSON to extract file path
    let model_info: serde_json::Value = serde_json::from_str(&model_info_json)
        .map_err(|e| ANEError::Internal(format!("Failed to parse model info: {}", e)))?;

    let source_path = model_info["path"].as_str()
        .ok_or_else(|| ANEError::Internal("Model info does not contain path".to_string()))?;

    // Copy the compiled model file to the destination
    let source_path = std::path::Path::new(source_path);
    if !source_path.exists() {
        return Err(ANEError::Internal(format!("Source model file does not exist: {:?}", source_path)));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ANEError::Internal(format!("Failed to create parent directory: {}", e)))?;
    }

    // Copy the file
    std::fs::copy(source_path, path)
        .map_err(|e| ANEError::Internal(format!("Failed to copy model file: {}", e)))?;

    // Verify the copy was successful
    if !path.exists() {
        return Err(ANEError::Internal("Model file copy verification failed".to_string()));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| ANEError::Internal(format!("Failed to verify copied file: {}", e)))?;

    if metadata.len() == 0 {
        return Err(ANEError::Internal("Copied model file is empty".to_string()));
    }

    Ok(())
}

/// Thread-local registry mapping ModelRef to CoreMlHandle
/// This should only be used on the thread that owns the CoreML handles.
/// Uses Rc<CoreMlHandle> to prevent premature drops during concurrent on-thread reads.
#[derive(Debug)]
pub struct ModelRegistry {
    models: HashMap<ModelRef, Rc<CoreMlHandle>>,
}

impl ModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a model handle and return an opaque reference
    pub fn register(&mut self, handle: CoreMlHandle) -> ModelRef {
        let id = ModelRef::new();
        self.models.insert(id, Rc::new(handle));
        id
    }

    /// Get a reference-counted handle for a model reference
    /// Returns None if the reference is not registered on this thread
    pub fn get_handle(&self, id: ModelRef) -> Option<Rc<CoreMlHandle>> {
        self.models.get(&id).cloned()
    }

    /// Remove a model from the registry (called during cleanup)
    /// Returns the handle so it can be properly dropped
    pub fn unregister(&mut self, id: ModelRef) -> Option<Rc<CoreMlHandle>> {
        self.models.remove(&id)
    }
}

// Thread-local storage for model registries
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

    /// Scoped access to a model handle for running a closure
    /// This ensures the handle cannot be taken ownership of and prevents lifetime issues
    pub fn with_model_handle<T, F: FnOnce(NonNull<std::ffi::c_void>) -> T>(id: ModelRef, f: F) -> Option<T> {
        MODEL_REGISTRY.with(|registry| {
            registry.borrow().get_handle(id).map(|rc_handle| {
                // Rc ensures the handle lives for the duration of the closure
                // SAFETY: rc_handle.as_ptr() is guaranteed to be non-null as it's from Rc
                f(unsafe { NonNull::new_unchecked(rc_handle.as_ptr()) })
            })
        })
    }

    /// Unregister a model (called during cleanup)
    /// Returns the handle for proper cleanup
    pub fn unregister_model(id: ModelRef) -> Option<Rc<CoreMlHandle>> {
        MODEL_REGISTRY.with(|registry| {
            registry.borrow_mut().unregister(id)
        })
    }

    /// DEPRECATED: Use with_model_handle instead to prevent ownership issues
    /// Get the raw handle for a model reference
    /// Returns None if called on wrong thread or reference doesn't exist
    #[deprecated(note = "Use with_model_handle for scoped access to prevent ownership issues")]
    pub fn get_model_handle(id: ModelRef) -> Option<NonNull<std::ffi::c_void>> {
        MODEL_REGISTRY.with(|registry| {
            registry.borrow().get_handle(id).map(|rc_handle| {
                // SAFETY: rc_handle.as_ptr() is guaranteed to be non-null as it's from Rc
                unsafe { NonNull::new_unchecked(rc_handle.as_ptr()) }
            })
        })
    }
}
