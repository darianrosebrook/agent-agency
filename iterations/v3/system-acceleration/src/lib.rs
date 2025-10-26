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

pub mod ane;
pub mod metal;
pub mod coreml;
pub mod model_router;
pub mod buffer_pool;
pub mod quantization;
pub mod telemetry;

// Re-export main types
pub use ane::{ANEManager, ANEConfig};
pub use model_router::ModelRouter;
pub use buffer_pool::BufferPool;
pub use quantization::Quantizer;
