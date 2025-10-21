//! FFI bindings for Apple Neural Engine and related frameworks
//!
//! This module contains all the foreign function interface bindings
//! for interacting with Apple's private frameworks.

use std::os::raw::{c_char, c_int, c_void};

/// ANE Framework FFI Bindings
/// Real ANE API bindings for Apple Silicon hardware
pub mod ane_framework {
    use std::os::raw::{c_int, c_void};

    /// ANE Device handle (opaque pointer)
    pub type ANEDeviceRef = *mut c_void;

    /// ANE Command Queue handle
    pub type ANECommandQueueRef = *mut c_void;

    /// ANE Model handle
    pub type ANEModelRef = *mut c_void;

    /// ANE Request handle
    pub type ANERequestRef = *mut c_void;

    /// ANE Performance Statistics
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct ANEPerformanceStats {
        pub total_operations: u64,
        pub active_operations: u32,
        pub average_latency_us: u32,
        pub peak_memory_mb: u32,
        pub power_consumption_mw: u32,
    }

    /// ANE Device Information
    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct ANEDeviceInfo {
        pub version: u32,
        pub memory_size_mb: u32,
        pub compute_units: u32,
        pub supports_fp16: bool,
        pub supports_int8: bool,
        pub max_input_size: u32,
        pub max_output_size: u32,
    }

    /// ANE Error codes
    #[repr(C)]
    #[derive(Debug, Clone, PartialEq)]
    pub enum ANEError {
        Success = 0,
        InvalidParameter = -1,
        DeviceNotFound = -2,
        OutOfMemory = -3,
        ModelLoadFailed = -4,
        ExecutionFailed = -5,
        Timeout = -6,
        UnsupportedOperation = -7,
    }

    extern "C" {
        /// Create and initialize ANE device
        pub fn ANECreateDevice(device_info: *mut ANEDeviceInfo) -> ANEError;

        /// Release ANE device
        pub fn ANEReleaseDevice(device: ANEDeviceRef) -> ANEError;

        /// Get ANE device information
        pub fn ANEGetDeviceInfo(device: ANEDeviceRef, info: *mut ANEDeviceInfo) -> ANEError;

        /// Create command queue for device
        pub fn ANECreateCommandQueue(device: ANEDeviceRef) -> ANECommandQueueRef;

        /// Release command queue
        pub fn ANEReleaseCommandQueue(queue: ANECommandQueueRef) -> ANEError;

        /// Load ANE model from compiled Core ML model
        pub fn ANELoadModel(device: ANEDeviceRef, model_path: *const i8) -> ANEModelRef;

        /// Release ANE model
        pub fn ANEReleaseModel(model: ANEModelRef) -> ANEError;

        /// Execute inference on ANE
        pub fn ANEExecuteInference(
            model: ANEModelRef,
            queue: ANECommandQueueRef,
            inputs: *const c_void,
            input_size: usize,
            outputs: *mut c_void,
            output_size: usize,
        ) -> ANEError;

        /// Get performance statistics
        pub fn ANEGetPerformanceStats(device: ANEDeviceRef, stats: *mut ANEPerformanceStats) -> ANEError;

        /// Wait for operation completion
        pub fn ANEWaitCompletion(queue: ANECommandQueueRef, timeout_ms: u32) -> ANEError;

        /// Check if ANE is available on this system
        pub fn ANEIsAvailable() -> bool;

        /// Get ANE driver version
        pub fn ANEGetDriverVersion() -> u32;
    }
}

/// Core ML and Swift AI Bridge FFI bindings
pub mod core_ml_bridge {
    use std::os::raw::{c_char, c_int, c_void};

    extern "C" {
        /// Load Core ML model for ANE execution
        pub fn core_ml_load_model(model_path: *const c_char) -> *mut c_void;

        /// Execute Core ML model on ANE
        pub fn core_ml_execute_model(
            model: *mut c_void,
            inputs: *const c_void,
            input_count: c_int,
            outputs: *mut *mut c_void,
            output_count: *mut c_int,
        ) -> c_int;

        /// Release Core ML model
        pub fn core_ml_release_model(model: *mut c_void);

        /// Get Core ML model metadata
        pub fn core_ml_get_model_info(
            model: *mut c_void,
            info_json: *mut *mut c_char,
        ) -> c_int;

        /// Convert Core ML model to ANE format
        pub fn core_ml_convert_to_ane(model_path: *const c_char, ane_path: *const c_char) -> c_int;

        /// Check if model is compatible with ANE
        pub fn core_ml_is_ane_compatible(model_path: *const c_char) -> c_int;
    }
}

/// Swift AI Bridge FFI bindings
pub mod swift_ai_bridge {
    use std::os::raw::{c_char, c_int, c_void};

    extern "C" {
        /// Initialize Swift AI bridge
        pub fn swift_ai_init() -> c_int;

        /// Load Swift AI model
        pub fn swift_ai_load_model(model_path: *const c_char) -> *mut c_void;

        /// Execute Swift AI inference
        pub fn swift_ai_execute(
            model: *mut c_void,
            input_data: *const c_void,
            input_size: usize,
            output_data: *mut *mut c_void,
            output_size: *mut usize,
        ) -> c_int;

        /// Release Swift AI model
        pub fn swift_ai_release_model(model: *mut c_void);

        /// Get Swift AI performance metrics
        pub fn swift_ai_get_metrics(
            model: *mut c_void,
            metrics_json: *mut *mut c_char,
        ) -> c_int;

        /// Optimize Swift AI model for ANE
        pub fn swift_ai_optimize_for_ane(model_path: *const c_char, optimized_path: *const c_char) -> c_int;
    }
}
