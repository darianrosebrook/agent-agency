//! ONNX Runtime compatibility layer
//!
//! Handles API differences between ort versions and provides a stable interface.
//! This isolates ort API changes from the rest of the codebase.

use anyhow::Result;
use std::path::Path;

/// Create an ONNX Runtime session from a model file
///
/// This function abstracts away ort API differences and provides a stable interface.
/// Temporarily disabled due to ort API compatibility issues.
pub fn create_session_from_file(_model_path: impl AsRef<Path>) -> Result<ort::session::Session> {
    // TODO: Fix ort 2.0 RC API compatibility
    // The SessionBuilder API has changed and needs investigation
    Err(anyhow::anyhow!(
        "ONNX Runtime session creation temporarily disabled due to API compatibility issues"
    ))
}

/// Convert ort::Error to anyhow::Error
///
/// ort::Error doesn't implement StdError, so we need to wrap it manually.
pub fn ort_error_to_anyhow(err: ort::Error) -> anyhow::Error {
    anyhow::anyhow!("ONNX Runtime error: {:?}", err)
}

/// Convert ndarray Array2 to Vec for ort Value::from_array
///
/// ort's Value::from_array expects OwnedTensorArrayData, which Array2 doesn't implement.
/// We convert to Vec first.
pub fn array2_to_vec<T: Clone>(array: &ndarray::Array2<T>) -> Vec<T> {
    array.iter().cloned().collect()
}
