//! Compatibility layer for unstable external APIs
//!
//! This module isolates all interactions with external crates that have
//! unstable APIs (objc2, metal, MPSGraph, etc.) behind stable interfaces.
//!
//! Only this module calls external APIs directly. Everything else calls
//! the compat façade for stability.

pub mod mps;
pub mod numerics;
pub mod onnx;
pub mod types;
