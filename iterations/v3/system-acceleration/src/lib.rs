//! System Acceleration Framework
//!
//! High-performance acceleration framework for machine learning inference,
//! providing unified access to hardware accelerators including Apple Neural Engine,
//! Metal GPU, and traditional CPU/GPU backends.
//!
//! ## Features
//!
//! - **Apple Neural Engine (ANE)**: Zero-overhead Core ML acceleration
//! - **Metal Performance Shaders**: GPU acceleration for compatible models
//! - **Unified Backend Interface**: Consistent API across all acceleration backends
//! - **Model Optimization**: Automatic quantization and performance tuning
//! - **Resource Management**: Intelligent backend selection and load balancing
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod ane;
// pub mod metal; // TODO: Implement Metal GPU acceleration
// pub mod coreml; // TODO: Implement Core ML acceleration
pub mod buffer_pool;
pub mod model_router;
pub mod quantization;
pub mod telemetry;

// Re-export main types
pub use ane::ANEManager;
pub use buffer_pool::BufferPool;
pub use model_router::ModelRouter;
pub use quantization::QuantizationConfig;

/// ANE capabilities structure
#[derive(Debug, Clone, Default, JsonSchema)]
pub struct ANECapabilities {
    /// Whether ANE is available
    pub is_available: bool,
    /// Number of compute units
    pub compute_units: u32,
    /// Maximum memory in MB
    pub max_memory_mb: Option<u64>,
    /// Supported precisions
    pub supported_precisions: Vec<String>,
    /// Maximum batch size supported
    pub max_batch_size: usize,
    /// Supported data types
    pub supported_types: Vec<String>,
    /// Memory limit in MB
    pub memory_limit_mb: u64,
    /// Performance tier
    pub performance_tier: u8,
}

/// Tensor specification for model inputs/outputs
#[derive(Debug, Clone, JsonSchema)]
pub struct TensorSpec {
    /// Tensor name
    pub name: String,
    /// Data type
    pub dtype: String,
    /// Shape dimensions
    pub shape: Vec<usize>,
    /// Whether this tensor supports batching
    pub batch_capable: bool,
}

/// Quantization method enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum QuantizationMethod {
    /// No quantization
    None,
    /// INT8 quantization
    Int8,
    /// INT4 quantization
    Int4,
    /// Dynamic quantization
    Dynamic,
}

/// Device identifier type
pub type DeviceId = String;
