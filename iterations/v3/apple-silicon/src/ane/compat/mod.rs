//! Compatibility layers for ANE operations
//!
//! This module provides compatibility layers for Core ML and IOKit integration
//! with proper platform detection and fallback implementations.

pub mod coreml;
pub mod iokit;

// Re-export commonly used types
pub use coreml::{CoreMLModel, InferenceOptions, ModelMetadata, CoreMLCapabilities};
pub use iokit::{ThermalStatus, PowerStatus, DeviceInfo, ThermalCapabilities};
