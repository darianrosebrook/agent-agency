//! # Agent Agency PyTorch Wrapper
//!
//! A safe abstraction layer around PyTorch/tch that provides:
//! - Optional linking (no PyTorch dependency by default)
//! - Stub implementations when PyTorch is not available
//! - Graceful degradation for Apple Silicon environments
//! - Async-safe tensor operations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use agent_agency_pytorch_wrapper::{PyTorchEngine, Tensor, Device};
//!
//! // Create engine (will use stubs if PyTorch not available)
//! let engine = PyTorchEngine::new()?;
//!
//! // Load model (safe - returns error if PyTorch not available)
//! let model = engine.load_model("path/to/model.pt").await?;
//!
//! // Perform inference
//! let input = Tensor::from_vec(vec![1.0, 2.0, 3.0], &[3])?;
//! let output = model.forward(input).await?;
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;

#[cfg(feature = "pytorch")]
mod pytorch_impl;
#[cfg(feature = "pytorch")]
pub use pytorch_impl::*;

#[cfg(not(feature = "pytorch"))]
mod stub_impl;
#[cfg(not(feature = "pytorch"))]
pub use stub_impl::*;

/// Errors that can occur in PyTorch operations
#[derive(Debug, thiserror::Error)]
pub enum PyTorchError {
    #[error("PyTorch is not available: {message}")]
    PyTorchUnavailable { message: String },

    #[error("Model loading failed: {message}")]
    ModelLoadError { message: String },

    #[error("Inference failed: {message}")]
    InferenceError { message: String },

    #[error("Invalid tensor shape: {message}")]
    InvalidShape { message: String },

    #[error("Device error: {message}")]
    DeviceError { message: String },
}

/// Represents a tensor (multidimensional array)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    /// Shape of the tensor
    pub shape: Vec<usize>,
    /// Data as flat vector
    pub data: Vec<f32>,
    /// Device where tensor resides
    pub device: Device,
}

/// Device types for tensor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Device {
    Cpu,
    Cuda { device_id: usize },
    Mps, // Apple Silicon
}

/// Configuration for PyTorch engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyTorchConfig {
    /// Device to use for operations
    pub device: Device,
    /// Whether to use mixed precision
    pub mixed_precision: bool,
    /// Memory limit in MB (0 = unlimited)
    pub memory_limit_mb: usize,
    /// Thread pool size for CPU operations
    pub thread_pool_size: usize,
}

impl Default for PyTorchConfig {
    fn default() -> Self {
        Self {
            device: Device::Cpu,
            mixed_precision: false,
            memory_limit_mb: 0,
            thread_pool_size: num_cpus::get(),
        }
    }
}

/// PyTorch model interface
#[async_trait]
pub trait Model: Send + Sync {
    /// Perform forward pass
    async fn forward(&self, input: Tensor) -> Result<Tensor, PyTorchError>;

    /// Get input shape requirements
    fn input_shape(&self) -> Vec<usize>;

    /// Get output shape
    fn output_shape(&self) -> Vec<usize>;

    /// Get model metadata
    fn metadata(&self) -> ModelMetadata;
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub framework: String,
    pub input_shapes: Vec<Vec<usize>>,
    pub output_shapes: Vec<Vec<usize>>,
    pub parameters: std::collections::HashMap<String, String>,
}

/// Main PyTorch engine interface
#[async_trait]
pub trait PyTorchEngineTrait: Send + Sync {
    /// Create new engine instance
    async fn new() -> Result<Self> where Self: Sized;

    /// Load model from file
    async fn load_model(&self, path: &str) -> Result<Box<dyn Model>>;

    /// Check if PyTorch is available
    fn is_available(&self) -> bool;

    /// Get engine version info
    fn version(&self) -> String;

    /// Get available devices
    fn available_devices(&self) -> Vec<Device>;
}

/// Type alias for the PyTorch engine
pub type PyTorchEngine = Box<dyn PyTorchEngineTrait>;

/// Global function to create PyTorch engine
pub async fn create_pytorch_engine() -> Result<PyTorchEngine> {
    #[cfg(feature = "pytorch")]
    {
        pytorch_impl::PyTorchEngineImpl::new().await.map(|e| Box::new(e) as PyTorchEngine)
    }

    #[cfg(not(feature = "pytorch"))]
    {
        stub_impl::StubPyTorchEngine::new().await.map(|e| Box::new(e) as PyTorchEngine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = create_pytorch_engine().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_availability() {
        let engine = create_pytorch_engine().await.unwrap();
        // Availability depends on feature flags, but creation should always work
        let _is_available = engine.is_available();
    }
}
