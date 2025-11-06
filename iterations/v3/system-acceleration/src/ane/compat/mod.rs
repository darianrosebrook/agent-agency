//! Compatibility layers for ANE operations
//!
//! This module provides compatibility layers for Core ML and IOKit integration
//! with proper platform detection and fallback implementations.

pub mod coreml_direct;
pub mod coreml;
pub mod coreml_module;
pub mod hardening;
pub mod integration;
pub mod iokit;
pub mod kv_cache;
pub mod model;
pub mod registry;
pub mod safety;
pub mod testing;
pub mod tokenizer;
pub mod types;

// Note: CoreML safe API types are available in coreml module
pub use iokit::{ThermalStatus, PowerStatus, DeviceInfo, ThermalCapabilities};
