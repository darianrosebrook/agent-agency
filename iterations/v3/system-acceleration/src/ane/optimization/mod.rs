//! ANE Optimization Module
//!
//! Intelligent optimization for Apple Neural Engine performance.

pub mod ane_optimizer;

// Re-export main types
pub use ane_optimizer::{
    ANEMemoryOptimizer, ANEOptimizationParams, ANEOptimizationStrategy, ANEOptimizer,
    BatchOptimizer, ComputeUnitPreference, MemoryStrategy, PerformanceStats, PrecisionMode,
};
