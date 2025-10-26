//! Compatibility layers for ANE operations
//!
//! This module provides compatibility layers for Core ML and IOKit integration
//! with proper platform detection and fallback implementations.

pub mod coreml;
pub mod iokit;

// Note: CoreML safe API types are available in coreml module
pub use iokit::{ThermalStatus, PowerStatus, DeviceInfo, ThermalCapabilities};
