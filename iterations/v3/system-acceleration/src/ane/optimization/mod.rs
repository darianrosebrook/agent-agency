//! ANE Optimization Module
//!
//! Intelligent optimization for Apple Neural Engine performance.

pub mod ane_optimizer;

// Re-export main types
pub use ane_optimizer::{
    ANEOptimizer, ANEOptimizationStrategy, ANEOptimizationParams,
    PrecisionMode, MemoryStrategy, ComputeUnitPreference,
    ANEMemoryOptimizer, BatchOptimizer, PerformanceStats
};
